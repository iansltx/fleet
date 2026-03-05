//! Rate limiting middleware.
//!
//! Provides rate limiting for sensitive endpoints such as login,
//! forgot_password, and MDM SSO. Uses a token-bucket algorithm
//! backed by Redis (matching Go's `throttled` library behavior).

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
/// In the Go server, this uses `throttled.GCRAStore` backed by Redis.
/// The Rust implementation will similarly use Redis for distributed rate limiting.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    pub config: RateLimitConfig,
    pub key_prefix: String,
}

impl RateLimiter {
    pub fn new(key_prefix: &str, config: RateLimitConfig) -> Self {
        Self {
            config,
            key_prefix: key_prefix.to_string(),
        }
    }

    /// Check if the request is allowed under the rate limit.
    ///
    /// Returns `true` if the request should be allowed.
    pub async fn allow(&self, _client_ip: &str) -> bool {
        // TODO: implement GCRA algorithm against Redis
        // For now, allow all requests.
        true
    }
}

/// IP-based error rate limiter for device endpoints.
///
/// Matches Go's `redis.NewIPBanner` behavior: tracks consecutive
/// failing requests per IP and temporarily bans IPs that exceed
/// the threshold.
#[derive(Debug, Clone)]
pub struct IPErrorRateLimiter {
    /// Maximum consecutive failing requests before banning.
    pub max_consecutive_failures: u64,
    /// Time window for counting consecutive failures (seconds).
    pub failure_window_secs: u64,
    /// Duration of the ban (seconds).
    pub ban_duration_secs: u64,
    /// Redis key prefix.
    pub key_prefix: String,
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
        }
    }

    /// Check if the IP is currently banned.
    pub async fn is_banned(&self, _client_ip: &str) -> bool {
        // TODO: check Redis
        false
    }

    /// Record a failure for the IP.
    pub async fn record_failure(&self, _client_ip: &str) {
        // TODO: increment failure counter in Redis
    }

    /// Reset the failure counter for the IP (on success).
    pub async fn record_success(&self, _client_ip: &str) {
        // TODO: reset counter in Redis
    }
}
