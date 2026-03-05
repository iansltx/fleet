//! Live query Redis operations.
//!
//! Mirrors the Go `server/live_query/redis_live_query.go` implementation.
//!
//! # Redis Key Patterns (identical to Go)
//!
//! - `livequery:{<ID>}` - bitfield indicating targeted hosts for campaign `<ID>`
//! - `sql:livequery:{<ID>}` - SQL text for campaign `<ID>`
//! - `livequery:active` - set containing all active campaign IDs
//! - `query_results_count:<ID>` - integer count of results for query `<ID>`
//!
//! The `{<ID>}` hash tag ensures that related keys hash to the same Redis
//! Cluster slot, matching the Go implementation exactly.

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::pool::RedisPool;
use crate::{RedisError, Result};

/// Key prefix for live query bitfield targets.
/// Go: `queryKeyPrefix = "livequery:"`
const QUERY_KEY_PREFIX: &str = "livequery:";

/// Key prefix for live query SQL.
/// Go: `sqlKeyPrefix = "sql:"`
const SQL_KEY_PREFIX: &str = "sql:";

/// Redis key for the set of active live query campaign IDs.
/// Go: `activeQueriesKey = "livequery:active"`
const ACTIVE_QUERIES_KEY: &str = "livequery:active";

/// TTL for live query keys (7 days).
/// Go: `queryExpiration = 7 * 24 * time.Hour`
const QUERY_EXPIRATION_SECS: u64 = 7 * 24 * 60 * 60;

/// Key prefix for query results counts.
/// Go: `queryResultsCountPrefix = "query_results_count:"`
const QUERY_RESULTS_COUNT_PREFIX: &str = "query_results_count:";

const BITS_IN_BYTE: u32 = 8;

/// Generate the target-bitfield key and sql key for a given campaign name.
///
/// Uses `{name}` hash tags so both keys land on the same cluster slot.
/// Go: `generateKeys(name)` returns `("livequery:{name}", "sql:livequery:{name}")`
fn generate_keys(name: &str) -> (String, String) {
    let key_tag = format!("{{{}}}", name);
    (
        format!("{}{}", QUERY_KEY_PREFIX, key_tag),
        format!("{}{}{}", SQL_KEY_PREFIX, QUERY_KEY_PREFIX, key_tag),
    )
}

/// Extract the base campaign name from a target key.
///
/// Go: `extractTargetKeyName(key)` strips "livequery:" prefix and surrounding braces.
fn extract_target_key_name(key: &str) -> &str {
    let name = key.strip_prefix(QUERY_KEY_PREFIX).unwrap_or(key);
    let name = name.strip_prefix('{').unwrap_or(name);
    let name = name.strip_suffix('}').unwrap_or(name);
    name
}

/// Generate the Redis key for a query results count.
/// Go: `queryResultsCountKey(queryID)` returns `"query_results_count:<ID>"`
fn query_results_count_key(query_id: u32) -> String {
    format!("{}{}", QUERY_RESULTS_COUNT_PREFIX, query_id)
}

/// Map host IDs into a bitfield compatible with Redis GETBIT/SETBIT.
///
/// Mirrors Go `mapBitfield(hostIDs)`. Input IDs must be sorted ascending.
/// Each host ID sets bit `(BITS_IN_BYTE - (id % BITS_IN_BYTE) - 1)` in
/// byte `id / BITS_IN_BYTE`.
fn map_bitfield(host_ids: &[u32]) -> Vec<u8> {
    if host_ids.is_empty() {
        return Vec::new();
    }
    let last_id = *host_ids.last().unwrap();
    let byte_len = (last_id / BITS_IN_BYTE + 1) as usize;
    let mut field = vec![0u8; byte_len];
    for &id in host_ids {
        let byte_index = (id / BITS_IN_BYTE) as usize;
        let bit_index = BITS_IN_BYTE - (id % BITS_IN_BYTE) - 1;
        field[byte_index] |= 1 << bit_index;
    }
    field
}

/// In-memory cache for active query names and their SQL, mirroring the Go
/// `memCache` struct.
struct MemCache {
    sql_cache: HashMap<String, String>,
    active_queries: Vec<String>,
    expires_at: Option<Instant>,
}

impl MemCache {
    fn new() -> Self {
        Self {
            sql_cache: HashMap::new(),
            active_queries: Vec::new(),
            expires_at: None,
        }
    }

    fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(exp) => Instant::now() >= exp,
            None => true,
        }
    }
}

/// Redis-backed live query store.
///
/// Mirrors Go's `redisLiveQuery` struct.
pub struct RedisLiveQuery {
    pool: RedisPool,
    cache: RwLock<MemCache>,
    cache_expiration: Duration,
}

impl RedisLiveQuery {
    /// Create a new `RedisLiveQuery` backed by the given pool.
    pub fn new(pool: RedisPool, cache_expiration: Duration) -> Self {
        Self {
            pool,
            cache: RwLock::new(MemCache::new()),
            cache_expiration,
        }
    }

    /// Store a live query (campaign) in Redis.
    ///
    /// `name` is the campaign ID as a string. `sql` is the query text.
    /// `host_ids` must be sorted in ascending order.
    ///
    /// Go: `RunQuery(name, sql, hostIDs)`
    pub async fn run_query(&self, name: &str, sql: &str, host_ids: &[u32]) -> Result<()> {
        if host_ids.is_empty() {
            return Err(RedisError::NoHostsTargeted);
        }

        self.store_query_info(name, sql, host_ids).await?;
        self.store_query_names(&[name]).await?;
        Ok(())
    }

    /// Remove a live query from Redis.
    ///
    /// Go: `StopQuery(name)`
    pub async fn stop_query(&self, name: &str) -> Result<()> {
        self.remove_query_info(name).await?;
        self.remove_query_names(&[name]).await?;
        Ok(())
    }

    /// Return the map of `campaign_id -> SQL` for all active queries that
    /// target the given host.
    ///
    /// Go: `QueriesForHost(hostID)`
    pub async fn queries_for_host(&self, host_id: u32) -> Result<HashMap<String, String>> {
        let names = self.load_active_query_names().await?;
        let key_names: Vec<String> = names
            .iter()
            .map(|n| {
                let (tkey, _) = generate_keys(n);
                tkey
            })
            .collect();

        let mut queries = HashMap::new();
        self.collect_queries_for_host(host_id, &key_names, &mut queries)
            .await?;
        Ok(queries)
    }

    /// Mark a host as having completed a query (clear its bit in the bitfield).
    ///
    /// Uses the same Lua script as Go to only SETBIT if the key EXISTS.
    ///
    /// Go: `QueryCompletedByHost(name, hostID)`
    pub async fn query_completed_by_host(&self, name: &str, host_id: u32) -> Result<()> {
        let (target_key, _) = generate_keys(name);
        let mut conn = self.pool.get_connection().await?;

        let script = redis::Script::new(
            r#"
            if redis.call('EXISTS', KEYS[1]) == 1 then
                return redis.call('SETBIT', KEYS[1], ARGV[1], ARGV[2])
            else
                return nil
            end
            "#,
        );
        conn.invoke_script::<Option<i64>, _>(
            |inv| { inv.key(&target_key).arg(host_id).arg(0); },
            &script,
        )
        .await?;
        Ok(())
    }

    /// Load the set of active query names (campaign IDs).
    ///
    /// Uses the in-memory cache if not expired.
    ///
    /// Go: `LoadActiveQueryNames()`
    pub async fn load_active_query_names(&self) -> Result<Vec<String>> {
        {
            let cache = self.cache.read().unwrap();
            if !cache.is_expired() {
                return Ok(cache.active_queries.clone());
            }
        }

        self.load_cache().await?;

        let cache = self.cache.read().unwrap();
        Ok(cache.active_queries.clone())
    }

    /// Clean up campaigns that are no longer active.
    ///
    /// Go: `CleanupInactiveQueries(ctx, inactiveCampaignIDs)`
    pub async fn cleanup_inactive_queries(&self, inactive_campaign_ids: &[u32]) -> Result<()> {
        if inactive_campaign_ids.is_empty() {
            return Ok(());
        }

        // Remove from the active set
        {
            let mut conn = self.pool.get_connection().await?;
            let mut cmd = redis::cmd("SREM");
            cmd.arg(ACTIVE_QUERIES_KEY);
            for id in inactive_campaign_ids {
                cmd.arg(id);
            }
            conn.query_async::<()>(&cmd).await?;
        }

        // Delete the target and sql keys
        let mut keys_to_del: Vec<String> = Vec::with_capacity(inactive_campaign_ids.len() * 2);
        for id in inactive_campaign_ids {
            let (target_key, sql_key) = generate_keys(&id.to_string());
            keys_to_del.push(target_key);
            keys_to_del.push(sql_key);
        }

        let mut conn = self.pool.get_connection().await?;
        let mut cmd = redis::cmd("DEL");
        for key in &keys_to_del {
            cmd.arg(key);
        }
        conn.query_async::<()>(&cmd).await?;

        Ok(())
    }

    /// Get query results counts for multiple query IDs.
    ///
    /// Returns a map of query_id -> count. Missing keys return 0.
    ///
    /// Go: `GetQueryResultsCounts(queryIDs)`
    pub async fn get_query_results_counts(
        &self,
        query_ids: &[u32],
    ) -> Result<HashMap<u32, i64>> {
        if query_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut conn = self.pool.get_connection().await?;
        let mut pipe = redis::pipe();
        for &qid in query_ids {
            pipe.cmd("GET").arg(query_results_count_key(qid));
        }

        let results: Vec<Option<i64>> = conn.pipe_query_async(&pipe).await?;

        let mut counts = HashMap::with_capacity(query_ids.len());
        for (i, &qid) in query_ids.iter().enumerate() {
            counts.insert(qid, results.get(i).copied().flatten().unwrap_or(0));
        }
        Ok(counts)
    }

    /// Increment query results counts by the given amounts.
    ///
    /// Go: `IncrQueryResultsCounts(queryIDsToAmounts)`
    pub async fn incr_query_results_counts(
        &self,
        query_ids_to_amounts: &HashMap<u32, i64>,
    ) -> Result<()> {
        if query_ids_to_amounts.is_empty() {
            return Ok(());
        }

        let mut conn = self.pool.get_connection().await?;
        let mut pipe = redis::pipe();
        for (&qid, &amount) in query_ids_to_amounts {
            pipe.cmd("INCRBY")
                .arg(query_results_count_key(qid))
                .arg(amount);
        }
        conn.pipe_query_async::<Vec<i64>>(&pipe).await?;
        Ok(())
    }

    /// Set the query results count for a specific query.
    ///
    /// Go: `SetQueryResultsCount(queryID, count)`
    pub async fn set_query_results_count(&self, query_id: u32, count: i64) -> Result<()> {
        let mut conn = self.pool.get_connection().await?;
        let key = query_results_count_key(query_id);
        let cmd = redis::cmd("SET").arg(&key).arg(count).clone();
        conn.query_async::<()>(&cmd).await?;
        Ok(())
    }

    /// Delete the query results count key for a query.
    ///
    /// Go: `DeleteQueryResultsCount(queryID)`
    pub async fn delete_query_results_count(&self, query_id: u32) -> Result<()> {
        let mut conn = self.pool.get_connection().await?;
        let key = query_results_count_key(query_id);
        let cmd = redis::cmd("DEL").arg(&key).clone();
        conn.query_async::<()>(&cmd).await?;
        Ok(())
    }

    // ---- internal helpers ----

    /// Store the SQL and bitfield targets for a query.
    async fn store_query_info(&self, name: &str, sql: &str, host_ids: &[u32]) -> Result<()> {
        let (target_key, sql_key) = generate_keys(name);
        let targets = map_bitfield(host_ids);

        let mut conn = self.pool.get_connection().await?;

        // Set SQL first (mirrors Go ordering to avoid race conditions).
        let cmd = redis::cmd("SET")
            .arg(&sql_key)
            .arg(sql)
            .arg("EX")
            .arg(QUERY_EXPIRATION_SECS)
            .clone();
        conn.query_async::<()>(&cmd).await?;

        let cmd = redis::cmd("SET")
            .arg(&target_key)
            .arg(targets)
            .arg("EX")
            .arg(QUERY_EXPIRATION_SECS)
            .clone();
        conn.query_async::<()>(&cmd).await?;

        Ok(())
    }

    /// Add campaign names to the active queries set.
    async fn store_query_names(&self, names: &[&str]) -> Result<()> {
        let mut conn = self.pool.get_connection().await?;
        let mut cmd = redis::cmd("SADD");
        cmd.arg(ACTIVE_QUERIES_KEY);
        for name in names {
            cmd.arg(*name);
        }
        conn.query_async::<()>(&cmd).await?;
        Ok(())
    }

    /// Remove the target and sql keys for a query.
    async fn remove_query_info(&self, name: &str) -> Result<()> {
        let (target_key, sql_key) = generate_keys(name);
        let mut conn = self.pool.get_connection().await?;
        let cmd = redis::cmd("DEL").arg(&target_key).arg(&sql_key).clone();
        conn.query_async::<()>(&cmd).await?;
        Ok(())
    }

    /// Remove campaign names from the active queries set.
    async fn remove_query_names(&self, names: &[&str]) -> Result<()> {
        let mut conn = self.pool.get_connection().await?;
        let mut cmd = redis::cmd("SREM");
        cmd.arg(ACTIVE_QUERIES_KEY);
        for name in names {
            cmd.arg(*name);
        }
        conn.query_async::<()>(&cmd).await?;
        Ok(())
    }

    /// Load the cache of active queries and their SQL from Redis.
    async fn load_cache(&self) -> Result<()> {
        let mut conn = self.pool.get_connection().await?;

        // SMEMBERS livequery:active
        let active_ids: Vec<String> = conn
            .query_async(redis::cmd("SMEMBERS").arg(ACTIVE_QUERIES_KEY))
            .await?;

        let mut sql_cache = HashMap::new();
        let mut expired_queries = Vec::new();

        for id in &active_ids {
            let (_, sql_key) = generate_keys(id);
            let sql: Option<String> =
                conn.query_async(redis::cmd("GET").arg(&sql_key)).await?;

            match sql {
                Some(s) => {
                    sql_cache.insert(id.clone(), s);
                }
                None => {
                    expired_queries.push(id.clone());
                }
            }
        }

        let active_ids: Vec<String> = if expired_queries.is_empty() {
            active_ids
        } else {
            active_ids
                .into_iter()
                .filter(|id| !expired_queries.contains(id))
                .collect()
        };

        {
            let mut cache = self.cache.write().unwrap();
            cache.sql_cache = sql_cache;
            cache.active_queries = active_ids;
            cache.expires_at = Some(Instant::now() + self.cache_expiration);
        }

        // Best-effort cleanup of expired queries (matching Go's probabilistic cleanup)
        if !expired_queries.is_empty() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as i64;
            if now % 10 == 0 {
                let pool = self.pool.clone();
                let expired = expired_queries;
                tokio::spawn(async move {
                    if let Ok(mut conn) = pool.get_connection().await {
                        let mut cmd = redis::cmd("SREM");
                        cmd.arg(ACTIVE_QUERIES_KEY);
                        for name in &expired {
                            cmd.arg(name);
                        }
                        let _ = conn.query_async::<()>(&cmd).await;
                    }
                });
            }
        }

        Ok(())
    }

    /// Check which of the given target keys have the host's bit set.
    async fn collect_queries_for_host(
        &self,
        host_id: u32,
        query_keys: &[String],
        queries: &mut HashMap<String, String>,
    ) -> Result<()> {
        if query_keys.is_empty() {
            return Ok(());
        }

        // Ensure cache is fresh
        {
            let cache = self.cache.read().unwrap();
            if cache.is_expired() {
                drop(cache);
                self.load_cache().await?;
            }
        }

        let mut conn = self.pool.get_connection().await?;

        // Pipeline GETBIT for each query key
        let mut pipe = redis::pipe();
        for key in query_keys {
            pipe.cmd("GETBIT").arg(key).arg(host_id);
        }

        let results: Vec<i64> = conn.pipe_query_async(&pipe).await?;

        let cache = self.cache.read().unwrap();
        for (i, key) in query_keys.iter().enumerate() {
            let targeted = results.get(i).copied().unwrap_or(0);
            if targeted == 1 {
                let name = extract_target_key_name(key);
                if let Some(sql) = cache.sql_cache.get(name) {
                    queries.insert(name.to_string(), sql.clone());
                } else {
                    tracing::warn!(name = name, "live query not found in cache");
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keys() {
        let (target, sql) = generate_keys("42");
        assert_eq!(target, "livequery:{42}");
        assert_eq!(sql, "sql:livequery:{42}");
    }

    #[test]
    fn test_extract_target_key_name() {
        assert_eq!(extract_target_key_name("livequery:{42}"), "42");
        assert_eq!(extract_target_key_name("livequery:{abc}"), "abc");
    }

    #[test]
    fn test_map_bitfield_empty() {
        assert_eq!(map_bitfield(&[]), Vec::<u8>::new());
    }

    #[test]
    fn test_map_bitfield_single() {
        let field = map_bitfield(&[0]);
        assert_eq!(field, vec![0b10000000]);
    }

    #[test]
    fn test_map_bitfield_multiple() {
        let field = map_bitfield(&[0, 1, 7]);
        assert_eq!(field, vec![0b11000001]);
    }

    #[test]
    fn test_map_bitfield_cross_byte() {
        let field = map_bitfield(&[0, 8]);
        assert_eq!(field, vec![0b10000000, 0b10000000]);
    }

    #[test]
    fn test_query_results_count_key() {
        assert_eq!(query_results_count_key(42), "query_results_count:42");
    }
}
