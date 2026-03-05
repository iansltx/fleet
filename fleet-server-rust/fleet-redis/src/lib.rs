//! Fleet Redis crate - handles all Redis interactions for the Fleet server.
//!
//! This crate provides Redis-backed implementations for:
//! - Connection pool management (standalone and cluster modes)
//! - Live query target/result storage using bitfields
//! - Pub/sub for live query campaign results
//! - GCRA-based rate limiting
//! - IP ban tracking for brute-force protection
//!
//! All Redis key patterns are identical to the Go implementation to ensure
//! drop-in compatibility with existing Redis data.

pub mod pool;
pub mod live_query;
pub mod pubsub;
pub mod rate_limit;
pub mod ip_banner;

pub use pool::{RedisPool, RedisMode, RedisConfig};
pub use live_query::RedisLiveQuery;
pub use pubsub::RedisQueryResults;
pub use rate_limit::ThrottledStore;
pub use ip_banner::IpBanner;

/// Errors produced by fleet-redis operations.
#[derive(Debug, thiserror::Error)]
pub enum RedisError {
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("no hosts targeted")]
    NoHostsTargeted,

    #[error("no subscribers on channel {0}")]
    NoSubscribers(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, RedisError>;
