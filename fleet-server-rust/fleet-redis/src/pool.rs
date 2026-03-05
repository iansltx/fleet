//! Redis connection pool setup supporting standalone and cluster modes.
//!
//! This mirrors the Go `server/datastore/redis/redis.go` pool configuration,
//! supporting both standalone Redis and Redis Cluster with identical config options.

use std::time::Duration;

use redis::aio::MultiplexedConnection;
use redis::cluster::ClusterClient;
use redis::cluster_async::ClusterConnection;
use redis::{Client, ConnectionAddr, ConnectionInfo, RedisConnectionInfo};

use crate::{RedisError, Result};

/// The mode in which Redis is running. Matches Go's `fleet.RedisMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedisMode {
    Standalone,
    Cluster,
}

impl std::fmt::Display for RedisMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RedisMode::Standalone => write!(f, "standalone"),
            RedisMode::Cluster => write!(f, "cluster"),
        }
    }
}

/// Configuration for the Redis connection pool.
/// Mirrors the Go `PoolConfig` struct in `server/datastore/redis/redis.go`.
#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// Redis server address in "host:port" format.
    pub server: String,
    /// Optional username for AUTH.
    pub username: Option<String>,
    /// Optional password for AUTH.
    pub password: Option<String>,
    /// Redis database number (standalone only).
    pub database: i64,
    /// Whether to use TLS.
    pub use_tls: bool,
    /// Connection timeout.
    pub connect_timeout: Duration,
    /// Whether to follow cluster redirections (MOVED/ASK).
    pub cluster_follow_redirections: bool,
    /// Whether to read from replica nodes in cluster mode.
    pub cluster_read_from_replica: bool,
    /// Maximum number of retry attempts when connecting.
    pub connect_retry_attempts: u32,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            server: "localhost:6379".to_string(),
            username: None,
            password: None,
            database: 0,
            use_tls: false,
            connect_timeout: Duration::from_secs(5),
            cluster_follow_redirections: true,
            cluster_read_from_replica: false,
            connect_retry_attempts: 0,
        }
    }
}

/// A Redis connection pool that abstracts over standalone and cluster modes.
///
/// Mirrors the Go `fleet.RedisPool` interface.
#[derive(Clone)]
pub enum RedisPool {
    Standalone(Client),
    Cluster(ClusterClient),
}

impl RedisPool {
    /// Create a new Redis pool from the provided configuration.
    ///
    /// The function first attempts to connect as a cluster. If the server does
    /// not support cluster mode, it falls back to standalone mode.
    pub async fn new(config: &RedisConfig) -> Result<Self> {
        match Self::try_cluster(config).await {
            Ok(pool) => Ok(pool),
            Err(_) => {
                tracing::info!("cluster mode not available, using standalone Redis");
                Self::try_standalone(config)
            }
        }
    }

    /// Create a standalone Redis pool directly (skip cluster detection).
    pub fn new_standalone(config: &RedisConfig) -> Result<Self> {
        Self::try_standalone(config)
    }

    /// Create a cluster Redis pool directly.
    pub async fn new_cluster(config: &RedisConfig) -> Result<Self> {
        Self::try_cluster(config).await
    }

    fn try_standalone(config: &RedisConfig) -> Result<Self> {
        let scheme = if config.use_tls {
            ConnectionAddr::TcpTls {
                host: parse_host(&config.server),
                port: parse_port(&config.server),
                insecure: false,
                tls_params: None,
            }
        } else {
            ConnectionAddr::Tcp(parse_host(&config.server), parse_port(&config.server))
        };

        let info = ConnectionInfo {
            addr: scheme,
            redis: RedisConnectionInfo {
                db: config.database,
                username: config.username.clone(),
                password: config.password.clone(),
                ..Default::default()
            },
        };

        let client = Client::open(info).map_err(RedisError::Redis)?;
        Ok(RedisPool::Standalone(client))
    }

    async fn try_cluster(config: &RedisConfig) -> Result<Self> {
        let node_url = if config.use_tls {
            format!("rediss://{}", config.server)
        } else {
            format!("redis://{}", config.server)
        };

        let mut builder = ClusterClient::builder(vec![node_url]);
        if let Some(ref username) = config.username {
            builder = builder.username(username.clone());
        }
        if let Some(ref password) = config.password {
            builder = builder.password(password.clone());
        }

        let client = builder.build().map_err(RedisError::Redis)?;

        // Verify cluster is reachable
        let _conn = client
            .get_async_connection()
            .await
            .map_err(RedisError::Redis)?;

        Ok(RedisPool::Cluster(client))
    }

    /// Returns the mode of this pool.
    pub fn mode(&self) -> RedisMode {
        match self {
            RedisPool::Standalone(_) => RedisMode::Standalone,
            RedisPool::Cluster(_) => RedisMode::Cluster,
        }
    }

    /// Get an async multiplexed connection (standalone mode).
    pub async fn get_multiplexed_connection(&self) -> Result<MultiplexedConnection> {
        match self {
            RedisPool::Standalone(client) => client
                .get_multiplexed_async_connection()
                .await
                .map_err(RedisError::Redis),
            RedisPool::Cluster(_) => Err(RedisError::Other(
                "use get_cluster_connection for cluster mode".to_string(),
            )),
        }
    }

    /// Get an async cluster connection (cluster mode).
    pub async fn get_cluster_connection(&self) -> Result<ClusterConnection> {
        match self {
            RedisPool::Cluster(client) => client
                .get_async_connection()
                .await
                .map_err(RedisError::Redis),
            RedisPool::Standalone(_) => Err(RedisError::Other(
                "use get_multiplexed_connection for standalone mode".to_string(),
            )),
        }
    }

    /// Get a unified connection that works in both modes.
    pub async fn get_connection(&self) -> Result<RedisConn> {
        match self {
            RedisPool::Standalone(client) => {
                let conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(RedisError::Redis)?;
                Ok(RedisConn::Standalone(conn))
            }
            RedisPool::Cluster(client) => {
                let conn = client
                    .get_async_connection()
                    .await
                    .map_err(RedisError::Redis)?;
                Ok(RedisConn::Cluster(conn))
            }
        }
    }

    /// Health check: send PING and verify PONG response.
    pub async fn health_check(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;
        conn.query_async::<String>(&redis::cmd("PING").clone()).await?;
        Ok(())
    }
}

/// Unified async connection that works in both standalone and cluster modes.
///
/// All Redis commands should go through the methods on this enum to avoid the
/// `Sized` constraint issues with trait objects in the redis crate.
pub enum RedisConn {
    Standalone(MultiplexedConnection),
    Cluster(ClusterConnection),
}

impl RedisConn {
    /// Execute a `redis::Cmd` and deserialize the result.
    pub async fn query_async<T: redis::FromRedisValue>(
        &mut self,
        cmd: &redis::Cmd,
    ) -> Result<T> {
        match self {
            RedisConn::Standalone(c) => cmd.query_async(c).await.map_err(RedisError::Redis),
            RedisConn::Cluster(c) => cmd.query_async(c).await.map_err(RedisError::Redis),
        }
    }

    /// Execute a `redis::Pipeline` and deserialize the result.
    pub async fn pipe_query_async<T: redis::FromRedisValue>(
        &mut self,
        pipe: &redis::Pipeline,
    ) -> Result<T> {
        match self {
            RedisConn::Standalone(c) => pipe.query_async(c).await.map_err(RedisError::Redis),
            RedisConn::Cluster(c) => pipe.query_async(c).await.map_err(RedisError::Redis),
        }
    }

    /// Invoke a Redis Lua script. Use the builder pattern on `redis::Script`
    /// to set keys and args, then call this to execute.
    ///
    /// Example:
    /// ```ignore
    /// let script = redis::Script::new("return 1");
    /// let result: i64 = conn.invoke_script(|s| s.key("mykey").arg(42), &script).await?;
    /// ```
    pub async fn invoke_script<T, F>(&mut self, f: F, script: &redis::Script) -> Result<T>
    where
        T: redis::FromRedisValue,
        F: FnOnce(&mut redis::ScriptInvocation<'_>),
    {
        let mut invocation = script.prepare_invoke();
        f(&mut invocation);
        match self {
            RedisConn::Standalone(c) => {
                invocation.invoke_async(c).await.map_err(RedisError::Redis)
            }
            RedisConn::Cluster(c) => {
                invocation.invoke_async(c).await.map_err(RedisError::Redis)
            }
        }
    }
}

fn parse_host(server: &str) -> String {
    server
        .rsplit_once(':')
        .map(|(h, _)| h.to_string())
        .unwrap_or_else(|| server.to_string())
}

fn parse_port(server: &str) -> u16 {
    server
        .rsplit_once(':')
        .and_then(|(_, p)| p.parse().ok())
        .unwrap_or(6379)
}
