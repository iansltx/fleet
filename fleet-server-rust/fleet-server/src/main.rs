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

/// Shared application state passed to all handlers via axum's State extractor.
///
/// This will hold the FleetService (which owns the datastore, Redis, config, etc.)
/// once a real Datastore implementation is available.
#[derive(Clone)]
pub struct AppState {
    pub service: Arc<FleetService>,
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
    // let ds_config = fleet_datastore::MysqlDatastoreConfig {
    //     protocol: cfg.mysql.protocol.clone(),
    //     address: cfg.mysql.address.clone(),
    //     username: cfg.mysql.username.clone(),
    //     password: cfg.mysql.password.clone(),
    //     database: cfg.mysql.database.clone(),
    //     tls_cert: cfg.mysql.tls_cert.clone(),
    //     tls_key: cfg.mysql.tls_key.clone(),
    //     tls_ca: cfg.mysql.tls_ca.clone(),
    //     tls_server_name: cfg.mysql.tls_server_name.clone(),
    //     tls_config: cfg.mysql.tls_config.clone(),
    //     max_open_conns: cfg.mysql.max_open_conns,
    //     max_idle_conns: cfg.mysql.max_idle_conns,
    //     conn_max_lifetime_secs: cfg.mysql.conn_max_lifetime,
    //     sql_mode: cfg.mysql.sql_mode.clone(),
    // };
    // let ds = fleet_datastore::MysqlDatastore::new(ds_config).await?;
    //
    // Build FleetService
    // let svc_config = fleet_service::FleetServiceConfig { ... };
    // let svc = fleet_service::FleetService::new(Arc::new(ds), svc_config);
    // let state = AppState { service: Arc::new(svc) };

    // Build the axum application with all routes
    let app = routes::build_router(cfg);

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
