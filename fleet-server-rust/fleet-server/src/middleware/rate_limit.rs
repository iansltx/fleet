//! Rate limiting middleware.
//!
//! Provides rate limiting for sensitive endpoints such as login,
//! forgot_password, and MDM SSO. Uses the GCRA algorithm
//! backed by Redis (matching Go's `throttled` library behavior).

use std::sync::Arc;

/// Rate limit configuration matching Go's `throttled.Rate` and `throttled.RateQuota`.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per the specified period.
    pub max_rate_per_period: u64,
    /// Period in seconds (e.g., 60 for per-minute).
    pub period_secs: u64,
    /// Maximum burst above the steady rate.
    pub max_burst: u64,
}

impl RateLimitConfig {
    /// 10 requests per minute with burst of 9 (login default from Go).
    pub fn login_default() -> Self {
        Self {
            max_rate_per_period: 10,
            period_secs: 60,
            max_burst: 9,
        }
    }

    /// 10 requests per hour with burst of 9 (forgot password from Go).
    pub fn forgot_password_default() -> Self {
        Self {
            max_rate_per_period: 10,
            period_secs: 3600,
            max_burst: 9,
        }
    }
}

/// State for the rate limiter.
///
/// Uses a GCRA (Generic Cell Rate Algorithm) backed by Redis via
/// `fleet_redis::ThrottledStore`. When no store is configured, all
/// requests are allowed (graceful degradation).
pub struct RateLimiter {
    pub config: RateLimitConfig,
    pub key_prefix: String,
    store: Option<Arc<fleet_redis::ThrottledStore>>,
}

impl RateLimiter {
    /// Create a new `RateLimiter` without a Redis store (allows all requests).
    pub fn new(key_prefix: &str, config: RateLimitConfig) -> Self {
        Self {
            config,
            key_prefix: key_prefix.to_string(),
            store: None,
        }
    }

    /// Create a new `RateLimiter` backed by a Redis `ThrottledStore`.
    pub fn with_store(
        key_prefix: &str,
        config: RateLimitConfig,
        store: Arc<fleet_redis::ThrottledStore>,
    ) -> Self {
        Self {
            config,
            key_prefix: key_prefix.to_string(),
            store: Some(store),
        }
    }

    /// Check if the request is allowed under the rate limit.
    ///
    /// Uses the GCRA algorithm:
    /// - Calculate emission interval = period / max_rate
    /// - Calculate delay tolerance (max burst window) = emission_interval * max_burst
    /// - Use ThrottledStore to get current TAT (Theoretical Arrival Time)
    /// - If no TAT exists, set initial TAT and allow
    /// - If request arrives after TAT, allow and update TAT
    /// - If request arrives before TAT but within burst window, allow
    /// - If request would exceed burst window, deny
    ///
    /// Returns `true` if the request should be allowed.
    pub async fn allow(&self, client_ip: &str) -> bool {
        let store = match &self.store {
            Some(s) => s,
            None => return true, // No Redis = allow all
        };

        let key = format!("{}:{}", self.key_prefix, client_ip);

        // GCRA parameters in nanoseconds
        let emission_interval_ns =
            (self.config.period_secs * 1_000_000_000) / self.config.max_rate_per_period;
        let delay_tolerance_ns = emission_interval_ns * self.config.max_burst;
        let increment_ns = emission_interval_ns;

        match store.get_with_time(&key).await {
            Ok((tat_ns, now)) => {
                let now_ns = now
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as i64;

                if tat_ns == -1 {
                    // Key doesn't exist, set initial TAT
                    let new_tat = now_ns + increment_ns as i64;
                    let ttl = std::time::Duration::from_nanos(
                        (increment_ns + delay_tolerance_ns) as u64,
                    );
                    let _ = store.set_if_not_exists_with_ttl(&key, new_tat, ttl).await;
                    true
                } else {
                    let new_tat = std::cmp::max(tat_ns, now_ns) + increment_ns as i64;
                    let allow_at = new_tat - delay_tolerance_ns as i64;

                    if now_ns < allow_at {
                        false // Rate limited
                    } else {
                        let ttl = std::time::Duration::from_nanos(
                            (increment_ns + delay_tolerance_ns) as u64,
                        );
                        let _ = store
                            .compare_and_swap_with_ttl(&key, tat_ns, new_tat, ttl)
                            .await;
                        true
                    }
                }
            }
            Err(_) => true, // Redis error = allow (graceful degradation)
        }
    }
}

/// IP-based error rate limiter for device endpoints.
///
/// Matches Go's `redis.NewIPBanner` behavior: tracks consecutive
/// failing requests per IP and temporarily bans IPs that exceed
/// the threshold.
///
/// When no `IpBanner` is configured, no banning occurs (graceful degradation).
pub struct IPErrorRateLimiter {
    /// Maximum consecutive failing requests before banning.
    pub max_consecutive_failures: u64,
    /// Time window for counting consecutive failures (seconds).
    pub failure_window_secs: u64,
    /// Duration of the ban (seconds).
    pub ban_duration_secs: u64,
    /// Redis key prefix.
    pub key_prefix: String,
    /// Redis-backed IP banner. When None, no banning occurs.
    banner: Option<Arc<fleet_redis::IpBanner>>,
}

impl IPErrorRateLimiter {
    /// Default configuration from Go:
    /// - 1000 consecutive failures per minute
    /// - 1 minute ban
    pub fn device_default() -> Self {
        Self {
            max_consecutive_failures: 1000,
            failure_window_secs: 60,
            ban_duration_secs: 60,
            key_prefix: "ipbanner::".to_string(),
            banner: None,
        }
    }

    /// Create an `IPErrorRateLimiter` backed by a Redis `IpBanner`.
    pub fn with_banner(
        max_consecutive_failures: u64,
        failure_window_secs: u64,
        ban_duration_secs: u64,
        key_prefix: &str,
        banner: Arc<fleet_redis::IpBanner>,
    ) -> Self {
        Self {
            max_consecutive_failures,
            failure_window_secs,
            ban_duration_secs,
            key_prefix: key_prefix.to_string(),
            banner: Some(banner),
        }
    }

    /// Check if the IP is currently banned.
    pub async fn is_banned(&self, client_ip: &str) -> bool {
        match &self.banner {
            Some(b) => b.check_banned(client_ip).await.unwrap_or(false),
            None => false, // No Redis = never banned
        }
    }

    /// Record a failure for the IP.
    pub async fn record_failure(&self, client_ip: &str) {
        if let Some(b) = &self.banner {
            let _ = b.run_request(client_ip, false).await;
        }
    }

    /// Reset the failure counter for the IP (on success).
    pub async fn record_success(&self, client_ip: &str) {
        if let Some(b) = &self.banner {
            let _ = b.run_request(client_ip, true).await;
        }
    }
}
