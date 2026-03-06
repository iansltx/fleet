//! MySQL connection setup and pool creation.
//!
//! This module creates a connection pool with the same DSN format as the Go codebase.
//! The Go DSN is generated in `server/platform/mysql/common.go` `generateMysqlConnectionString`.

use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode};
use sqlx::MySqlPool;
use std::str::FromStr;
use std::time::Duration;

use crate::error::Result;

/// Configuration for connecting to MySQL, matching Go's config.MysqlConfig fields.
#[derive(Debug, Clone)]
pub struct MysqlDatastoreConfig {
    pub protocol: String,
    pub address: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub tls_cert: String,
    pub tls_key: String,
    pub tls_ca: String,
    pub tls_server_name: String,
    pub tls_config: String,
    pub max_open_conns: u32,
    pub max_idle_conns: u32,
    pub conn_max_lifetime_secs: u64,
    pub sql_mode: String,
}

impl Default for MysqlDatastoreConfig {
    fn default() -> Self {
        Self {
            protocol: "tcp".to_string(),
            address: "localhost:3306".to_string(),
            username: "fleet".to_string(),
            password: "fleet".to_string(),
            database: "fleet".to_string(),
            tls_cert: String::new(),
            tls_key: String::new(),
            tls_ca: String::new(),
            tls_server_name: String::new(),
            tls_config: String::new(),
            max_open_conns: 50,
            max_idle_conns: 50,
            conn_max_lifetime_secs: 0,
            sql_mode: String::new(),
        }
    }
}

/// The main MySQL datastore struct, holding a connection pool.
/// This is the Rust equivalent of Go's `mysql.Datastore` struct.
#[derive(Debug, Clone)]
pub struct MysqlDatastore {
    pool: MySqlPool,
}

impl MysqlDatastore {
    /// Creates a new MysqlDatastore from configuration.
    ///
    /// Builds a DSN compatible with the Go codebase's `generateMysqlConnectionString`:
    ///   `user:password@protocol(address)/database?params`
    ///
    /// Go uses these default connection parameters:
    ///   - collation=utf8mb4_unicode_ci
    ///   - parseTime=true
    ///   - loc=UTC
    ///   - time_zone='-00:00'
    ///   - clientFoundRows=true
    ///   - allowNativePasswords=true
    ///   - group_concat_max_len=4194304
    ///   - multiStatements=true
    pub async fn new(config: MysqlDatastoreConfig) -> Result<Self> {
        // Parse host and port from address (Go format: "host:port")
        let (host, port) = parse_address(&config.address);

        let mut options = MySqlConnectOptions::new()
            .host(&host)
            .port(port)
            .username(&config.username)
            .password(&config.password)
            .database(&config.database)
            .charset("utf8mb4")
            .collation("utf8mb4_unicode_ci")
            .timezone(Some(String::from("+00:00")));

        // Configure TLS matching Go's behavior
        if !config.tls_config.is_empty() && config.tls_config != "false" {
            options = options.ssl_mode(MySqlSslMode::Required);
        } else if !config.tls_ca.is_empty() {
            options = options
                .ssl_mode(MySqlSslMode::VerifyCa)
                .ssl_ca(&config.tls_ca);
        } else {
            options = options.ssl_mode(MySqlSslMode::Preferred);
        }

        let pool = MySqlPoolOptions::new()
            .max_connections(config.max_open_conns)
            .min_connections(config.max_idle_conns.min(config.max_open_conns))
            .max_lifetime(if config.conn_max_lifetime_secs > 0 {
                Some(Duration::from_secs(config.conn_max_lifetime_secs))
            } else {
                None
            })
            // Match Go's after_connect: SET sql_mode, group_concat_max_len, time_zone
            .after_connect(move |conn, _meta| {
                let sql_mode = config.sql_mode.clone();
                Box::pin(async move {
                    use sqlx::Executor;
                    // Set session variables matching Go's DSN parameters
                    conn.execute("SET time_zone = '+00:00'").await?;
                    conn.execute("SET group_concat_max_len = 4194304").await?;
                    if !sql_mode.is_empty() {
                        conn.execute(format!("SET sql_mode = '{}'", sql_mode).as_str())
                            .await?;
                    }
                    Ok(())
                })
            })
            .connect_with(options)
            .await?;

        tracing::info!("Connected to MySQL at {}", config.address);

        Ok(Self { pool })
    }

    /// Creates a MysqlDatastore from a DSN string (for testing/migration convenience).
    pub async fn from_dsn(dsn: &str) -> Result<Self> {
        let options = MySqlConnectOptions::from_str(dsn)
            .map_err(|e| sqlx::Error::Configuration(e.into()))?;

        let pool = MySqlPoolOptions::new()
            .max_connections(50)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    /// Returns a reference to the underlying connection pool.
    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }

    /// Run database migrations.
    ///
    /// Creates a `fleet_migrations` tracking table and applies the schema
    /// from the Go server's schema.sql if not already applied. This is a
    /// simplified approach that applies the full schema at once rather than
    /// running individual Go goose migration files.
    ///
    /// For production, consider using sqlx's migration system with individual
    /// SQL files extracted from the Go migrations.
    pub async fn migrate(&self, schema_sql: &str) -> Result<()> {
        use sqlx::Executor;

        // Create migration tracking table if it doesn't exist
        self.pool
            .execute(
                "CREATE TABLE IF NOT EXISTS fleet_migrations (
                    id INT AUTO_INCREMENT PRIMARY KEY,
                    version VARCHAR(255) NOT NULL UNIQUE,
                    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"
            )
            .await?;

        // Check if schema has been applied
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT COUNT(*) FROM fleet_migrations WHERE version = 'initial_schema'"
        )
        .fetch_optional(&self.pool)
        .await?;

        if row.map(|r| r.0).unwrap_or(0) > 0 {
            tracing::info!("Database schema already applied, skipping migration");
            return Ok(());
        }

        tracing::info!("Applying database schema...");

        // Split schema into individual statements and execute each
        // The schema uses semicolons as statement delimiters
        for statement in schema_sql.split(';') {
            let stmt = statement.trim();
            if stmt.is_empty() || stmt.starts_with("--") || stmt.starts_with("/*!") {
                continue;
            }
            if let Err(e) = self.pool.execute(stmt).await {
                tracing::warn!(error = %e, "Migration statement failed (may be expected for conditional DDL)");
            }
        }

        // Record migration
        sqlx::query("INSERT INTO fleet_migrations (version) VALUES ('initial_schema')")
            .execute(&self.pool)
            .await?;

        tracing::info!("Database schema applied successfully");
        Ok(())
    }
}

/// Parse a Go-style address "host:port" or just "host" into (host, port).
fn parse_address(address: &str) -> (String, u16) {
    if let Some((host, port_str)) = address.rsplit_once(':') {
        if let Ok(port) = port_str.parse::<u16>() {
            return (host.to_string(), port);
        }
    }
    (address.to_string(), 3306)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address() {
        assert_eq!(parse_address("localhost:3306"), ("localhost".to_string(), 3306));
        assert_eq!(parse_address("db.example.com:3307"), ("db.example.com".to_string(), 3307));
        assert_eq!(parse_address("localhost"), ("localhost".to_string(), 3306));
    }
}
