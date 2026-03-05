//! Redis pub/sub wrapper for live query campaign results.
//!
//! Mirrors the Go `server/pubsub/redis_query_results.go` implementation.
//!
//! # Redis Channel Patterns (identical to Go)
//!
//! - `results_<campaign_id>` - channel for distributing query results for a campaign
//! - `LQDuplicate` - channel for duplicating live query results (when enabled)

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::pool::RedisPool;
use crate::{RedisError, Result};

/// Channel name for a live query campaign's results.
///
/// Go: `pubSubForID(id)` returns `"results_<id>"`
fn pub_sub_channel_for_id(campaign_id: u32) -> String {
    format!("results_{}", campaign_id)
}

/// Channel name for duplicating live query results.
///
/// Go: `"LQDuplicate"` constant used in `WriteResult`.
const LQ_DUPLICATE_CHANNEL: &str = "LQDuplicate";

/// A distributed query result, matching the Go JSON format exactly.
///
/// Go: `fleet.DistributedQueryResult` serialized as JSON over pub/sub.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedQueryResult {
    /// Campaign ID.
    #[serde(rename = "distributed_query_execution_id")]
    pub distributed_query_campaign_id: u32,
    /// Host data.
    pub host: ResultHostData,
    /// Result rows.
    pub rows: Vec<std::collections::HashMap<String, String>>,
    /// Query stats.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<QueryStats>,
    /// Error reported by osquery.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Host data included in a distributed query result.
///
/// Go: `fleet.ResultHostData`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultHostData {
    pub id: u32,
    pub hostname: String,
    pub display_name: String,
}

/// Query execution stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wall_time_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_time: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_time: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<u64>,
}

/// Redis-backed pub/sub for distributing live query results.
///
/// Mirrors Go's `redisQueryResults` struct.
///
/// For PUBLISH operations, the pool's connection is used directly (works for
/// both standalone and cluster). For SUBSCRIBE operations, a dedicated
/// standalone `redis::Client` is needed because Redis pub/sub requires a
/// persistent, non-multiplexed connection. In cluster mode, the Go code uses
/// `ReadOnlyConn` which connects to any node - pub/sub works on any node
/// since messages are broadcast cluster-wide.
pub struct RedisQueryResults {
    pool: RedisPool,
    /// A standalone client used for pub/sub subscriptions. In standalone mode,
    /// this is the same client as in the pool. In cluster mode, this should
    /// point to any node in the cluster.
    pubsub_client: redis::Client,
    /// When true, results are also published to the `LQDuplicate` channel.
    duplicate_results: bool,
}

impl RedisQueryResults {
    /// Create a new `RedisQueryResults`.
    ///
    /// `pubsub_client` is a standalone `redis::Client` used for SUBSCRIBE.
    /// In standalone mode, pass the same client from the pool.
    /// In cluster mode, pass a client pointing to any cluster node.
    pub fn new(
        pool: RedisPool,
        pubsub_client: redis::Client,
        duplicate_results: bool,
    ) -> Self {
        Self {
            pool,
            pubsub_client,
            duplicate_results,
        }
    }

    /// Convenience constructor for standalone mode that extracts the client.
    pub fn new_standalone(pool: RedisPool, duplicate_results: bool) -> Result<Self> {
        let pubsub_client = match &pool {
            RedisPool::Standalone(client) => client.clone(),
            RedisPool::Cluster(_) => return Err(RedisError::Other(
                "use RedisQueryResults::new() for cluster mode with a dedicated pubsub client"
                    .to_string(),
            )),
        };
        Ok(Self {
            pool,
            pubsub_client,
            duplicate_results,
        })
    }

    /// Publish a query result to the campaign's channel.
    ///
    /// Returns `Err(NoSubscribers)` if no one is listening.
    ///
    /// Go: `WriteResult(result)` serializes the result as JSON and PUBLISHes
    /// to `results_<campaign_id>`. If `duplicateResults` is set and there are
    /// subscribers, also publishes to `LQDuplicate`.
    pub async fn write_result(&self, result: &DistributedQueryResult) -> Result<()> {
        let channel_name = pub_sub_channel_for_id(result.distributed_query_campaign_id);
        let json_val = serde_json::to_string(result)?;

        let mut conn = self.pool.get_connection().await?;

        // PUBLISH returns the number of subscribers that received the message
        let cmd = redis::cmd("PUBLISH")
            .arg(&channel_name)
            .arg(&json_val)
            .clone();
        let num_subscribers: i64 = conn.query_async(&cmd).await?;

        if num_subscribers > 0 && self.duplicate_results {
            // Best-effort duplicate publishing, ignore errors (matches Go behavior)
            let dup_cmd = redis::cmd("PUBLISH")
                .arg(LQ_DUPLICATE_CHANNEL)
                .arg(&json_val)
                .clone();
            let _ = conn.query_async::<i64>(&dup_cmd).await;
        }

        if num_subscribers == 0 {
            return Err(RedisError::NoSubscribers(channel_name));
        }

        Ok(())
    }

    /// Subscribe to a live query campaign's results channel.
    ///
    /// Returns a channel receiver that yields `DistributedQueryResult` messages.
    /// The subscription is active until the returned receiver is dropped.
    ///
    /// Go: `ReadChannel(ctx, query)` subscribes to `results_<campaign_id>` and
    /// returns `<-chan interface{}`.
    pub async fn read_channel(
        &self,
        campaign_id: u32,
    ) -> Result<mpsc::Receiver<std::result::Result<DistributedQueryResult, RedisError>>> {
        let channel_name = pub_sub_channel_for_id(campaign_id);
        let (tx, rx) = mpsc::channel(64);

        let mut pubsub = self
            .pubsub_client
            .get_async_pubsub()
            .await
            .map_err(RedisError::Redis)?;
        pubsub
            .subscribe(&channel_name)
            .await
            .map_err(RedisError::Redis)?;

        tokio::spawn(async move {
            Self::run_subscriber(pubsub, tx, campaign_id).await;
        });

        Ok(rx)
    }

    /// Background task that reads messages from a PubSub connection and
    /// forwards them as deserialized `DistributedQueryResult`s.
    async fn run_subscriber(
        mut pubsub: redis::aio::PubSub,
        tx: mpsc::Sender<std::result::Result<DistributedQueryResult, RedisError>>,
        campaign_id: u32,
    ) {
        use futures::StreamExt;

        let mut msg_stream = pubsub.on_message();

        while let Some(msg) = msg_stream.next().await {
            let payload: std::result::Result<Vec<u8>, _> = msg.get_payload();
            let result = match payload {
                Ok(bytes) => serde_json::from_slice::<DistributedQueryResult>(&bytes)
                    .map_err(RedisError::from),
                Err(e) => Err(RedisError::Redis(e)),
            };

            if tx.send(result).await.is_err() {
                // Receiver dropped
                break;
            }
        }

        // Stream ended unexpectedly (connection closed).
        // Match Go error message for UI compatibility:
        // "unexpected exit in receiveMessages, campaignID=<id>"
        let _ = tx
            .send(Err(RedisError::Other(format!(
                "unexpected exit in receiveMessages, campaignID={}",
                campaign_id
            ))))
            .await;
    }

    /// Health check: send PING to verify Redis is reachable.
    ///
    /// Go: `HealthCheck()`
    pub async fn health_check(&self) -> Result<()> {
        self.pool.health_check().await
    }
}
