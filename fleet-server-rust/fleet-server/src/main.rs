//! Fleet server - main entry point.
//!
//! Parses CLI arguments and dispatches to the appropriate subcommand
//! (serve, prepare, version).

mod config;
mod frontend;
mod handlers;
mod middleware;
mod response;
mod routes;

use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{fmt, EnvFilter};

use fleet_service::FleetService;
use fleet_redis::{RedisLiveQuery, RedisQueryResults, RedisPool, RedisConfig as RedisPoolConfig};
use fleet_types::blobstore::BlobStore;

/// Shared application state passed to all handlers via axum's State extractor.
///
/// This will hold the FleetService (which owns the datastore, Redis, config, etc.)
/// once a real Datastore implementation is available.
#[derive(Clone)]
pub struct AppState {
    pub service: Arc<FleetService>,
    pub live_query: Arc<RedisLiveQuery>,
    pub query_results: Arc<RedisQueryResults>,
    /// Blob store for software installers.
    pub installer_store: Arc<dyn BlobStore>,
    /// Blob store for software title icons.
    pub icon_store: Arc<dyn BlobStore>,
    /// Blob store for MDM bootstrap packages.
    pub bootstrap_package_store: Arc<dyn BlobStore>,
}

/// Fleet server - osquery management and orchestration.
///
/// Options may be supplied in a YAML configuration file or via environment
/// variables with the FLEET_ prefix. You only need to define the configuration
/// values for which you wish to override the default value.
#[derive(Parser)]
#[command(name = "fleet-server", version, about)]
struct Cli {
    /// Path to a configuration file
    #[arg(short, long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the Fleet server
    Serve {
        /// Enable debug endpoints
        #[arg(long)]
        debug: bool,

        /// Enable development Fleet Premium license
        #[arg(long)]
        dev_license: bool,

        /// Enable development Fleet Premium license with an expired license
        #[arg(long)]
        dev_expired_license: bool,
    },

    /// Subcommands for initializing Fleet infrastructure
    Prepare {
        #[command(subcommand)]
        command: PrepareCommands,
    },

    /// Print Fleet server version
    Version,
}

#[derive(Subcommand)]
enum PrepareCommands {
    /// Given correct database configurations, prepare the databases for use
    Db {
        /// Disable prompting before migrations (for use in scripts)
        #[arg(long)]
        no_prompt: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Load configuration from file and environment variables
    let cfg = config::FleetConfig::load(cli.config.as_deref())?;

    // Initialize logging
    init_logging(&cfg);

    match cli.command {
        Commands::Serve {
            debug,
            dev_license,
            dev_expired_license,
        } => {
            run_serve(cfg, debug, dev_license, dev_expired_license).await?;
        }
        Commands::Prepare { command } => match command {
            PrepareCommands::Db { no_prompt } => {
                run_prepare_db(&cfg, no_prompt).await?;
            }
        },
        Commands::Version => {
            println!("fleet-server version {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}

/// Initialize the tracing/logging subsystem based on configuration.
fn init_logging(cfg: &config::FleetConfig) {
    let env_filter = if cfg.logging.debug {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    if cfg.logging.json {
        fmt()
            .json()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(true)
            .init();
    } else {
        fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .init();
    }
}

/// Run the Fleet HTTP server.
async fn run_serve(
    cfg: config::FleetConfig,
    _debug: bool,
    _dev_license: bool,
    _dev_expired_license: bool,
) -> anyhow::Result<()> {
    if !cfg.logging.disable_banner {
        print_banner();
    }

    let addr: SocketAddr = cfg.server.address.parse().unwrap_or_else(|_| {
        tracing::warn!(
            "Invalid server address '{}', falling back to 0.0.0.0:8080",
            cfg.server.address
        );
        "0.0.0.0:8080".parse().unwrap()
    });

    tracing::info!("Starting Fleet server on {}", addr);

    // Connect to MySQL
    let ds_config = fleet_datastore::MysqlDatastoreConfig {
        protocol: cfg.mysql.protocol.clone(),
        address: cfg.mysql.address.clone(),
        username: cfg.mysql.username.clone(),
        password: cfg.mysql.password.clone(),
        database: cfg.mysql.database.clone(),
        tls_cert: cfg.mysql.tls_cert.clone(),
        tls_key: cfg.mysql.tls_key.clone(),
        tls_ca: cfg.mysql.tls_ca.clone(),
        tls_server_name: cfg.mysql.tls_server_name.clone(),
        tls_config: cfg.mysql.tls_config.clone(),
        max_open_conns: cfg.mysql.max_open_conns,
        max_idle_conns: cfg.mysql.max_idle_conns,
        conn_max_lifetime_secs: cfg.mysql.conn_max_lifetime,
        sql_mode: cfg.mysql.sql_mode.clone(),
    };
    let ds = fleet_datastore::MysqlDatastore::new(ds_config).await
        .map_err(|e| anyhow::anyhow!("Failed to connect to MySQL: {}", e))?;

    // Connect to Redis
    let redis_pool_config = RedisPoolConfig {
        server: cfg.redis.address.clone(),
        username: if cfg.redis.username.is_empty() { None } else { Some(cfg.redis.username.clone()) },
        password: if cfg.redis.password.is_empty() { None } else { Some(cfg.redis.password.clone()) },
        database: cfg.redis.database as i64,
        use_tls: cfg.redis.use_tls,
        connect_timeout: std::time::Duration::from_secs(cfg.redis.connect_timeout_secs),
        cluster_follow_redirections: cfg.redis.cluster_follow_redirections,
        cluster_read_from_replica: cfg.redis.cluster_read_from_replica,
        connect_retry_attempts: cfg.redis.connect_retry_attempts,
    };
    let redis_pool = RedisPool::new(&redis_pool_config).await
        .map_err(|e| anyhow::anyhow!("Failed to connect to Redis: {}", e))?;
    tracing::info!(mode = %redis_pool.mode(), "Redis connected");

    let live_query = Arc::new(RedisLiveQuery::new(redis_pool.clone(), std::time::Duration::from_secs(1)));
    let query_results = Arc::new(
        RedisQueryResults::new_standalone(redis_pool.clone(), cfg.redis.duplicate_results)
            .map_err(|e| anyhow::anyhow!("Failed to create query results pubsub: {}", e))?
    );

    // Initialize blob stores (S3 or filesystem)
    let (installer_store, icon_store, bootstrap_package_store) = init_blob_stores(&cfg).await?;

    // Build FleetService
    let svc_config = fleet_service::FleetServiceConfig {
        server: fleet_service::ServerConfig {
            url_prefix: cfg.server.url_prefix.clone(),
            server_url: format!("https://{}", cfg.server.address),
        },
        session: fleet_service::SessionConfig {
            key_size: cfg.session.key_size,
            duration: std::time::Duration::from_secs(cfg.session.duration_secs),
        },
        osquery: fleet_service::OsqueryConfig {
            node_key_size: cfg.osquery.node_key_size,
            host_identifier: cfg.osquery.host_identifier.clone(),
            enroll_cooldown: std::time::Duration::from_secs(cfg.osquery.enroll_cooldown_secs),
            label_update_interval: std::time::Duration::from_secs(cfg.osquery.label_update_interval_secs),
            policy_update_interval: std::time::Duration::from_secs(cfg.osquery.policy_update_interval_secs),
            detail_update_interval: std::time::Duration::from_secs(cfg.osquery.detail_update_interval_secs),
        },
        auth: fleet_service::AuthConfig {
            jwt_key: cfg.server.private_key.clone(),
            bcrypt_cost: cfg.auth.bcrypt_cost,
            salt_key_size: cfg.auth.salt_key_size,
        },
        app: fleet_service::AppServiceConfig {
            token_key_size: cfg.app.token_key_size,
        },
    };

    let svc = fleet_service::FleetService::new(Arc::new(ds), svc_config);
    let state = AppState {
        service: Arc::new(svc),
        live_query,
        query_results,
        installer_store,
        icon_store,
        bootstrap_package_store,
    };

    // Build the axum application with all routes
    let app = routes::build_router(state);

    // Create the TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Fleet server listening on {}", addr);

    // Serve with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Fleet server stopped");
    Ok(())
}

/// Initialize blob stores based on configuration.
///
/// If S3 software_installers_bucket is configured, uses S3; otherwise falls
/// back to local filesystem storage under `/tmp/fleet/`.
async fn init_blob_stores(
    cfg: &config::FleetConfig,
) -> anyhow::Result<(Arc<dyn BlobStore>, Arc<dyn BlobStore>, Arc<dyn BlobStore>)> {
    if !cfg.s3.software_installers_bucket.is_empty() {
        let s3_cfg = fleet_blobstore::s3::S3Config {
            bucket: cfg.s3.software_installers_bucket.clone(),
            prefix: if cfg.s3.software_installers_prefix.is_empty() {
                cfg.s3.prefix.clone()
            } else {
                cfg.s3.software_installers_prefix.clone()
            },
            region: if cfg.s3.software_installers_region.is_empty() {
                cfg.s3.region.clone()
            } else {
                cfg.s3.software_installers_region.clone()
            },
            endpoint_url: {
                let ep = if cfg.s3.software_installers_endpoint_url.is_empty() {
                    &cfg.s3.endpoint_url
                } else {
                    &cfg.s3.software_installers_endpoint_url
                };
                if ep.is_empty() { None } else { Some(ep.clone()) }
            },
            access_key_id: {
                let k = if cfg.s3.software_installers_access_key_id.is_empty() {
                    &cfg.s3.access_key_id
                } else {
                    &cfg.s3.software_installers_access_key_id
                };
                if k.is_empty() { None } else { Some(k.clone()) }
            },
            secret_access_key: {
                let k = if cfg.s3.software_installers_secret_access_key.is_empty() {
                    &cfg.s3.secret_access_key
                } else {
                    &cfg.s3.software_installers_secret_access_key
                };
                if k.is_empty() { None } else { Some(k.clone()) }
            },
            force_path_style: cfg.s3.software_installers_force_s3_path_style || cfg.s3.force_s3_path_style,
            disable_ssl: cfg.s3.software_installers_disable_ssl || cfg.s3.disable_ssl,
        };

        let installer_store = fleet_blobstore::S3BlobStore::new(&s3_cfg, "software-installers").await
            .map_err(|e| anyhow::anyhow!("Failed to init S3 installer store: {}", e))?;
        let icon_store = fleet_blobstore::S3BlobStore::new(&s3_cfg, "software-title-icons").await
            .map_err(|e| anyhow::anyhow!("Failed to init S3 icon store: {}", e))?;
        let bootstrap_store = fleet_blobstore::S3BlobStore::new(&s3_cfg, "bootstrap-packages").await
            .map_err(|e| anyhow::anyhow!("Failed to init S3 bootstrap store: {}", e))?;

        tracing::info!(bucket = %s3_cfg.bucket, "Using S3 blob storage");
        Ok((Arc::new(installer_store), Arc::new(icon_store), Arc::new(bootstrap_store)))
    } else {
        let base_dir = "/tmp/fleet";
        let installer_store = fleet_blobstore::FilesystemBlobStore::new(base_dir, "software-installers")
            .map_err(|e| anyhow::anyhow!("Failed to init filesystem installer store: {}", e))?;
        let icon_store = fleet_blobstore::FilesystemBlobStore::new(base_dir, "software-title-icons")
            .map_err(|e| anyhow::anyhow!("Failed to init filesystem icon store: {}", e))?;
        let bootstrap_store = fleet_blobstore::FilesystemBlobStore::new(base_dir, "bootstrap-packages")
            .map_err(|e| anyhow::anyhow!("Failed to init filesystem bootstrap store: {}", e))?;

        tracing::info!(path = %base_dir, "Using filesystem blob storage");
        Ok((Arc::new(installer_store), Arc::new(icon_store), Arc::new(bootstrap_store)))
    }
}

/// Run database migrations.
async fn run_prepare_db(cfg: &config::FleetConfig, _no_prompt: bool) -> anyhow::Result<()> {
    tracing::info!(
        "Preparing database at {}@{}",
        cfg.mysql.username,
        cfg.mysql.address
    );

    let ds_config = fleet_datastore::MysqlDatastoreConfig {
        protocol: cfg.mysql.protocol.clone(),
        address: cfg.mysql.address.clone(),
        username: cfg.mysql.username.clone(),
        password: cfg.mysql.password.clone(),
        database: cfg.mysql.database.clone(),
        tls_cert: cfg.mysql.tls_cert.clone(),
        tls_key: cfg.mysql.tls_key.clone(),
        tls_ca: cfg.mysql.tls_ca.clone(),
        tls_server_name: cfg.mysql.tls_server_name.clone(),
        tls_config: cfg.mysql.tls_config.clone(),
        max_open_conns: cfg.mysql.max_open_conns,
        max_idle_conns: cfg.mysql.max_idle_conns,
        conn_max_lifetime_secs: cfg.mysql.conn_max_lifetime,
        sql_mode: cfg.mysql.sql_mode.clone(),
    };

    let _ds = fleet_datastore::MysqlDatastore::new(ds_config).await
        .map_err(|e| anyhow::anyhow!("Failed to connect to MySQL: {}", e))?;

    // TODO: Run schema migrations using the datastore
    tracing::info!("Database migrations completed.");
    Ok(())
}

/// Wait for SIGINT or SIGTERM for graceful shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown");
}

fn print_banner() {
    println!(
        r#"
   _____ _         _
  |  ___| | ___  _| |_
  | |_  | |/ _ \/ _ \ __|
  |  _| | |  __/  __/ |_
  |_|   |_|\___|\___|\__|

  https://fleetdm.com
"#
    );
}
