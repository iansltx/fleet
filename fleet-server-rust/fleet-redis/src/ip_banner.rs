//! IP ban tracking for brute-force protection.
//!
//! Mirrors the Go `server/datastore/redis/ip_banner.go` implementation.
//!
//! # Redis Key Patterns (identical to Go)
//!
//! - `<key_prefix>{<ip>}::count` - integer counter of consecutive failures
//! - `<key_prefix>{<ip>}::banned` - boolean flag indicating the IP is banned
//!
//! The `{<ip>}` hash tag ensures both keys for the same IP land on the same
//! Redis Cluster slot, matching the Go implementation.
//!
//! # Lua Script
//!
//! The `updateCountScript` is executed atomically and is identical to the Go
//! version. It handles:
//! - On failure: increment counter, set ban if threshold reached
//! - On success: reset counter

use std::time::Duration;

use crate::pool::RedisPool;
use crate::Result;

/// Lua script executed on every request to update IP ban state.
///
/// KEYS[1]: `<prefix>{<ip>}::count` (integer value)
/// KEYS[2]: `<prefix>{<ip>}::banned` (boolean value)
/// ARGV[1]: "0" for failure, "1" for success
/// ARGV[2]: threshold of consecutive failures
/// ARGV[3]: counter TTL in seconds (window for consecutive failures)
/// ARGV[4]: ban duration in seconds
///
/// This is the exact same Lua script as in Go's `updateCountScript`.
const UPDATE_COUNT_SCRIPT: &str = r#"
local threshold = tonumber(ARGV[2])
local counter_ttl = tonumber(ARGV[3])
local ban_ttl = tonumber(ARGV[4])

if ARGV[1] == "0" then
  -- failure: increment consecutive-failure counter
  local current = redis.call("INCR", KEYS[1])
  if current == 1 then
    redis.call("EXPIRE", KEYS[1], counter_ttl)
  elseif current >= threshold then
    -- set ban key with expiry
    redis.call("SET", KEYS[2], 1, "EX", ban_ttl)
    -- reset counter (delete it)
    redis.call("DEL", KEYS[1])
  end
else
  -- success: reset consecutive-failure counter
  redis.call("DEL", KEYS[1])
end
"#;

/// IP banning mechanism backed by Redis.
///
/// Allows a configurable number of consecutive failures within a time window.
/// After hitting the threshold, the IP is banned for a configurable duration.
///
/// Mirrors Go's `IPBanner` struct.
pub struct IpBanner {
    pool: RedisPool,
    /// Key prefix for all ban-related keys.
    key_prefix: String,
    /// Number of consecutive failures allowed before banning.
    allowed_consecutive_failures_count: i64,
    /// Time window within which failures are counted.
    allowed_consecutive_failures_time_window: Duration,
    /// How long an IP stays banned after hitting the threshold.
    ban_duration: Duration,
}

impl IpBanner {
    /// Create a new `IpBanner`.
    ///
    /// Go: `NewIPBanner(pool, keyPrefix, count, window, banDuration)`
    pub fn new(
        pool: RedisPool,
        key_prefix: String,
        allowed_consecutive_failures_count: i64,
        allowed_consecutive_failures_time_window: Duration,
        ban_duration: Duration,
    ) -> Self {
        Self {
            pool,
            key_prefix,
            allowed_consecutive_failures_count,
            allowed_consecutive_failures_time_window,
            ban_duration,
        }
    }

    /// Check if an IP is currently banned.
    ///
    /// Returns `true` if the IP is banned.
    ///
    /// Go: `CheckBanned(ip)` does `GET <prefix>{<ip>}::banned`
    pub async fn check_banned(&self, ip: &str) -> Result<bool> {
        let ip = set_null_if_empty_ip(ip);
        // Key format matches Go: `s.keyPrefix + "{" + ip + "}::banned"`
        let key = format!("{}{{{}}}::banned", self.key_prefix, ip);

        let mut conn = self.pool.get_connection().await?;
        let result: Option<String> =
            conn.query_async(redis::cmd("GET").arg(&key)).await?;

        Ok(result.is_some())
    }

    /// Update the ban state for an IP based on request success/failure.
    ///
    /// Go: `RunRequest(ip, success)` executes the `updateCountScript` Lua script.
    pub async fn run_request(&self, ip: &str, success: bool) -> Result<()> {
        let ip = set_null_if_empty_ip(ip);

        // Key format matches Go:
        // `s.keyPrefix + "{" + ip + "}::count"`
        // `s.keyPrefix + "{" + ip + "}::banned"`
        let ip_count_key = format!("{}{{{}}}::count", self.key_prefix, ip);
        let ip_banned_key = format!("{}{{{}}}::banned", self.key_prefix, ip);

        let action = if success { "1" } else { "0" };

        // Minimum 1 second for EX values (matches Go: `max(int(...), 1)`)
        let window_ttl_seconds = std::cmp::max(
            self.allowed_consecutive_failures_time_window.as_secs() as i64,
            1,
        );
        let ban_ttl_seconds = std::cmp::max(self.ban_duration.as_secs() as i64, 1);

        let mut conn = self.pool.get_connection().await?;

        let script = redis::Script::new(UPDATE_COUNT_SCRIPT);
        let allowed = self.allowed_consecutive_failures_count;
        conn.invoke_script::<(), _>(
            |inv| {
                inv.key(&ip_count_key)
                    .key(&ip_banned_key)
                    .arg(action)
                    .arg(allowed)
                    .arg(window_ttl_seconds)
                    .arg(ban_ttl_seconds);
            },
            &script,
        )
        .await?;

        Ok(())
    }
}

/// Replace empty IP strings with "null" to avoid empty Redis key components.
///
/// Go: `SetNullIfEmptyIP(ip)` returns `"null"` when ip is `""`.
fn set_null_if_empty_ip(ip: &str) -> &str {
    if ip.is_empty() {
        "null"
    } else {
        ip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_null_if_empty_ip() {
        assert_eq!(set_null_if_empty_ip(""), "null");
        assert_eq!(set_null_if_empty_ip("192.168.1.1"), "192.168.1.1");
    }

    #[test]
    fn test_key_format_count() {
        let prefix = "login:";
        let ip = "10.0.0.1";
        let key = format!("{}{{{}}}::count", prefix, ip);
        assert_eq!(key, "login:{10.0.0.1}::count");
    }

    #[test]
    fn test_key_format_banned() {
        let prefix = "login:";
        let ip = "10.0.0.1";
        let key = format!("{}{{{}}}::banned", prefix, ip);
        assert_eq!(key, "login:{10.0.0.1}::banned");
    }

    #[test]
    fn test_key_format_null_ip() {
        let prefix = "login:";
        let ip = set_null_if_empty_ip("");
        let count_key = format!("{}{{{}}}::count", prefix, ip);
        let banned_key = format!("{}{{{}}}::banned", prefix, ip);
        assert_eq!(count_key, "login:{null}::count");
        assert_eq!(banned_key, "login:{null}::banned");
    }
}
