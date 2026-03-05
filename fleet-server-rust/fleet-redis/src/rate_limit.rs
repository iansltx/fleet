//! GCRA-based rate limiting store backed by Redis.
//!
//! Mirrors the Go `server/datastore/redis/ratelimit_store.go` implementation.
//!
//! # Redis Key Patterns (identical to Go)
//!
//! Keys are constructed as `<key_prefix><key>` where `key_prefix` is configurable
//! and `key` is provided by the rate limiter (typically based on the route and
//! client IP or API token).
//!
//! The GCRA algorithm uses three atomic Redis operations:
//! - `GetWithTime` - GET the current TAT along with the server TIME
//! - `SetIfNotExistsWithTTL` - SET NX EX to initialize a new key
//! - `CompareAndSwapWithTTL` - Lua script to atomically CAS the TAT value
//!
//! These Lua scripts are identical to the Go implementation.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::pool::RedisPool;
use crate::{RedisError, Result};

/// Lua script that atomically GETs a key's value and the Redis server TIME.
///
/// Returns `[seconds, microseconds, value]` where value may be nil.
///
/// Go: `getWithTimeScript`
const GET_WITH_TIME_SCRIPT: &str = r#"
local tbl = redis.call('TIME')
local val = redis.call('GET', KEYS[1])
table.insert(tbl, val)
return tbl
"#;

/// Lua script that atomically compares and swaps a key's value with a TTL.
///
/// Returns 1 if swapped, 0 if old value didn't match, or error if key doesn't exist.
///
/// Go: `compareAndSwapWithTTLScript`
const COMPARE_AND_SWAP_WITH_TTL_SCRIPT: &str = r#"
local v = redis.call('get', KEYS[1])
if v == false then
  return redis.error_reply("key does not exist")
end
if v ~= ARGV[1] then
  return 0
end
redis.call('SET', KEYS[1], ARGV[2], 'EX', ARGV[3])
return 1
"#;

/// Error string returned by the CAS script when the key does not exist.
///
/// Go: `compareAndSwapNoKeyError = "key does not exist"`
const COMPARE_AND_SWAP_NO_KEY_ERROR: &str = "key does not exist";

/// Redis-backed store for GCRA rate limiting.
///
/// Mirrors Go's `ThrottledStore` struct.
pub struct ThrottledStore {
    pool: RedisPool,
    /// Key prefix prepended to all rate limit keys.
    key_prefix: String,
}

impl ThrottledStore {
    /// Create a new `ThrottledStore` with the given pool and key prefix.
    pub fn new(pool: RedisPool, key_prefix: String) -> Self {
        Self { pool, key_prefix }
    }

    /// Get the current value and server time for a rate limit key.
    ///
    /// Returns `(value, server_time)` where value is -1 if the key does not
    /// exist (matching Go convention).
    ///
    /// Go: `GetWithTime(key)` returns `(int64, time.Time, error)`
    pub async fn get_with_time(&self, key: &str) -> Result<(i64, SystemTime)> {
        let full_key = format!("{}{}", self.key_prefix, key);
        let mut conn = self.pool.get_connection().await?;

        let script = redis::Script::new(GET_WITH_TIME_SCRIPT);
        let result: Vec<redis::Value> = conn
            .invoke_script(|inv| { inv.key(&full_key); }, &script)
            .await?;

        // Parse [seconds, microseconds, value_or_nil]
        let secs = redis_value_to_i64(&result[0]).unwrap_or(0);
        let us = redis_value_to_i64(&result[1]).unwrap_or(0);
        let val = if result.len() > 2 {
            redis_value_to_i64(&result[2]).unwrap_or(-1)
        } else {
            -1
        };

        let time =
            UNIX_EPOCH + Duration::from_secs(secs as u64) + Duration::from_micros(us as u64);

        Ok((val, time))
    }

    /// Set a rate limit key only if it does not already exist, with a TTL.
    ///
    /// Returns `true` if the key was set (i.e., it did not exist).
    ///
    /// Go: `SetIfNotExistsWithTTL(key, value, ttl)` -> `(bool, error)`
    pub async fn set_if_not_exists_with_ttl(
        &self,
        key: &str,
        value: i64,
        ttl: Duration,
    ) -> Result<bool> {
        let full_key = format!("{}{}", self.key_prefix, key);
        let mut conn = self.pool.get_connection().await?;

        // Minimum 1 second TTL (matches Go: `if ttlSeconds < 1 { ttlSeconds = 1 }`)
        let ttl_seconds = std::cmp::max(ttl.as_secs(), 1);

        // SET key value EX ttl NX
        let cmd = redis::cmd("SET")
            .arg(&full_key)
            .arg(value)
            .arg("EX")
            .arg(ttl_seconds)
            .arg("NX")
            .clone();
        let result: Option<String> = conn.query_async(&cmd).await?;

        // SET ... NX returns "OK" if set, nil if not set
        Ok(result.is_some())
    }

    /// Atomically compare-and-swap a rate limit key's value with a new TTL.
    ///
    /// Returns `true` if the swap succeeded, `false` if the current value
    /// didn't match `old` or the key doesn't exist.
    ///
    /// Go: `CompareAndSwapWithTTL(key, old, new, ttl)` -> `(bool, error)`
    pub async fn compare_and_swap_with_ttl(
        &self,
        key: &str,
        old: i64,
        new: i64,
        ttl: Duration,
    ) -> Result<bool> {
        let full_key = format!("{}{}", self.key_prefix, key);
        let mut conn = self.pool.get_connection().await?;

        // Minimum 1 second TTL (matches Go)
        let ttl_seconds = std::cmp::max(ttl.as_secs(), 1);

        let script = redis::Script::new(COMPARE_AND_SWAP_WITH_TTL_SCRIPT);
        let result: std::result::Result<i64, RedisError> = conn
            .invoke_script(
                |inv| { inv.key(&full_key).arg(old).arg(new).arg(ttl_seconds); },
                &script,
            )
            .await;

        match result {
            Ok(val) => Ok(val == 1),
            Err(RedisError::Redis(e)) => {
                // "key does not exist" is not a real error - it means the key
                // expired between GetWithTime and CAS (matches Go behavior)
                if e.to_string().contains(COMPARE_AND_SWAP_NO_KEY_ERROR) {
                    Ok(false)
                } else {
                    Err(RedisError::Redis(e))
                }
            }
            Err(e) => Err(e),
        }
    }
}

/// Helper to extract an i64 from a redis::Value.
fn redis_value_to_i64(val: &redis::Value) -> Option<i64> {
    match val {
        redis::Value::Int(n) => Some(*n),
        redis::Value::BulkString(bytes) => std::str::from_utf8(bytes).ok()?.parse::<i64>().ok(),
        redis::Value::Nil => None,
        _ => None,
    }
}
