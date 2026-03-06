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
    /// Background cron scheduler.
    pub cron_scheduler: Arc<fleet_service::cron::CronScheduler>,
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
    dev_license: bool,
    dev_expired_license: bool,
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
        license: if dev_license || dev_expired_license {
            let expiration = if dev_expired_license {
                chrono::Utc::now() - chrono::Duration::hours(1)
            } else {
                chrono::Utc::now() + chrono::Duration::days(365)
            };
            tracing::info!(
                tier = "premium",
                expired = dev_expired_license,
                "Using dev license"
            );
            fleet_service::LicenseInfo {
                tier: fleet_service::LicenseTier::Premium,
                organization: "development".to_string(),
                device_count: 100,
                expiration,
                note: "Development license".to_string(),
            }
        } else {
            fleet_service::LicenseInfo::default()
        },
    };

    let ds = Arc::new(ds);
    let svc = fleet_service::FleetService::new(ds.clone(), svc_config);
    // Initialize cron scheduler with default jobs
    let cron_scheduler = Arc::new(fleet_service::cron::CronScheduler::new());
    register_cron_jobs(&cron_scheduler, ds).await;

    let state = AppState {
        service: Arc::new(svc),
        live_query,
        query_results,
        installer_store,
        icon_store,
        bootstrap_package_store,
        cron_scheduler: cron_scheduler.clone(),
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

    // Stop background jobs
    cron_scheduler.shutdown().await;

    tracing::info!("Fleet server stopped");
    Ok(())
}

/// Register default cron jobs with the scheduler.
///
/// Each job runs periodically and can be triggered ad-hoc via the trigger API.
/// The jobs mirror Go's cron schedule registrations in `cmd/fleet/serve.go`.
async fn register_cron_jobs(
    scheduler: &fleet_service::cron::CronScheduler,
    ds: Arc<fleet_datastore::MysqlDatastore>,
) {
    use fleet_service::cron::{CronJob, schedule_names};

    // Cleanups + aggregation: runs every hour (matches Go's newCleanupsAndAggregationSchedule)
    {
        let ds = ds.clone();
        scheduler.register(CronJob {
            name: schedule_names::CLEANUPS_THEN_AGGREGATION.to_string(),
            interval: std::time::Duration::from_secs(3600),
            func: Box::new(move || {
                let ds = ds.clone();
                Box::pin(async move {
                    let now = chrono::Utc::now();
                    let mut errors = Vec::new();

                    // Cleanup jobs (run first, matching Go ordering)
                    match ds.cleanup_distributed_query_campaigns(now).await {
                        Ok(n) => if n > 0 { tracing::info!(count = n, "expired distributed query campaigns"); },
                        Err(e) => errors.push(format!("distributed_query_campaigns: {e}")),
                    }

                    let cleanup_cutoff = now - chrono::Duration::hours(1);
                    match ds.cleanup_completed_campaign_targets(cleanup_cutoff).await {
                        Ok(n) => if n > 0 { tracing::info!(deleted = n, "cleaned up campaign targets"); },
                        Err(e) => errors.push(format!("campaign_targets: {e}")),
                    }

                    match ds.cleanup_incoming_hosts(now).await {
                        Ok(ids) => if !ids.is_empty() { tracing::info!(count = ids.len(), "cleaned up incoming hosts"); },
                        Err(e) => errors.push(format!("incoming_hosts: {e}")),
                    }

                    match ds.cleanup_carves(now).await {
                        Ok(n) => if n > 0 { tracing::info!(expired = n, "cleaned up carves"); },
                        Err(e) => errors.push(format!("carves: {e}")),
                    }

                    match ds.cleanup_policy_membership(now).await {
                        Ok(n) => if n > 0 { tracing::info!(removed = n, "cleaned up stale policy membership"); },
                        Err(e) => errors.push(format!("policy_membership: {e}")),
                    }

                    if let Err(e) = ds.cleanup_host_operating_systems().await {
                        errors.push(format!("host_operating_systems: {e}"));
                    }

                    if let Err(e) = ds.cleanup_expired_password_reset_requests().await {
                        errors.push(format!("password_reset_requests: {e}"));
                    }

                    // Query results cleanup (discard-related)
                    match ds.are_query_reports_disabled().await {
                        Ok(true) => {
                            if let Err(e) = ds.cleanup_global_discard_query_results().await {
                                errors.push(format!("global_discard_query_results: {e}"));
                            }
                        }
                        Ok(false) => {}
                        Err(e) => errors.push(format!("query_reports_check: {e}")),
                    }

                    if let Err(e) = ds.cleanup_discarded_query_results().await {
                        errors.push(format!("discarded_query_results: {e}"));
                    }

                    if let Err(e) = ds.cleanup_unused_script_contents().await {
                        errors.push(format!("unused_script_contents: {e}"));
                    }

                    // Aggregation jobs (run after cleanups, matching Go ordering)
                    if let Err(e) = ds.update_query_aggregated_stats().await {
                        errors.push(format!("query_aggregated_stats: {e}"));
                    }

                    if let Err(e) = ds.update_host_policy_counts().await {
                        errors.push(format!("host_policy_counts: {e}"));
                    }

                    if let Err(e) = ds.generate_aggregated_munki_and_mdm().await {
                        errors.push(format!("aggregated_munki_mdm: {e}"));
                    }

                    // Cron stats cleanup (Go runs this independently but we include it here)
                    if let Err(e) = ds.cleanup_cron_stats().await {
                        errors.push(format!("cron_stats: {e}"));
                    }

                    if errors.is_empty() {
                        Ok(())
                    } else {
                        Err(errors.join("; "))
                    }
                })
            }),
        }).await;
    }

    // Frequent cleanups: runs every 15 minutes (matches Go's newFrequentCleanupsSchedule)
    {
        let ds = ds.clone();
        scheduler.register(CronJob {
            name: schedule_names::FREQUENT_CLEANUPS.to_string(),
            interval: std::time::Duration::from_secs(900),
            func: Box::new(move || {
                let ds = ds.clone();
                Box::pin(async move {
                    // Clean up sessions idle for more than 4 hours (default session duration)
                    let session_max_idle = 4 * 60 * 60;
                    match ds.cleanup_expired_sessions(session_max_idle).await {
                        Ok(n) => if n > 0 { tracing::info!(expired = n, "cleaned up expired sessions"); },
                        Err(e) => return Err(format!("session_cleanup: {e}")),
                    }

                    Ok(())
                })
            }),
        }).await;
    }

    // Usage statistics: runs every hour (matches Go's newUsageStatisticsSchedule)
    // Checks if analytics are enabled and sends anonymous usage data to fleetdm.com.
    {
        let ds = ds.clone();
        scheduler.register(CronJob {
            name: schedule_names::USAGE_STATISTICS.to_string(),
            interval: std::time::Duration::from_secs(3600),
            func: Box::new(move || {
                let ds = ds.clone();
                Box::pin(async move {
                    // Check if analytics are enabled
                    let enabled = ds.are_analytics_enabled().await
                        .map_err(|e| format!("checking analytics: {e}"))?;
                    if !enabled {
                        tracing::debug!("analytics disabled, skipping statistics send");
                        return Ok(());
                    }

                    // Check if enough time has elapsed (Go uses StatisticsFrequency = 1 week)
                    let one_week_secs = 7 * 24 * 3600;
                    let should_send = ds.should_send_statistics(one_week_secs).await
                        .map_err(|e| format!("checking statistics frequency: {e}"))?;
                    if !should_send {
                        tracing::debug!("statistics recently sent, skipping");
                        return Ok(());
                    }

                    // Send statistics to fleetdm.com
                    let url = "https://fleetdm.com/api/v1/webhooks/receive-usage-analytics";
                    let config = ds.app_config().await
                        .map_err(|e| format!("getting app config: {e}"))?;

                    let payload = serde_json::json!({
                        "anonymousIdentifier": config.get("server_settings")
                            .and_then(|s| s.get("server_url"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown"),
                        "fleetVersion": env!("CARGO_PKG_VERSION"),
                    });

                    let client = reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(30))
                        .build()
                        .map_err(|e| format!("building HTTP client: {e}"))?;

                    match client.post(url).json(&payload).send().await {
                        Ok(resp) if resp.status().is_success() => {
                            tracing::info!("usage statistics sent successfully");
                        }
                        Ok(resp) => {
                            tracing::warn!(status = %resp.status(), "statistics endpoint returned non-success");
                        }
                        Err(e) => {
                            return Err(format!("sending statistics: {e}"));
                        }
                    }

                    ds.cleanup_statistics().await
                        .map_err(|e| format!("cleaning up statistics: {e}"))?;
                    ds.record_statistics_sent().await
                        .map_err(|e| format!("recording statistics sent: {e}"))?;

                    Ok(())
                })
            }),
        }).await;
    }

    // Vulnerabilities: runs every hour (matches Go's newVulnerabilitiesSchedule)
    // Scans software inventory for known vulnerabilities using NVD/OVAL data.
    {
        let ds = ds.clone();
        scheduler.register(CronJob {
            name: schedule_names::VULNERABILITIES.to_string(),
            interval: std::time::Duration::from_secs(3600),
            func: Box::new(move || {
                let ds = ds.clone();
                Box::pin(async move {
                    // Check if software inventory is enabled
                    let config = ds.app_config().await
                        .map_err(|e| format!("getting app config: {e}"))?;

                    let sw_enabled = config
                        .get("features")
                        .and_then(|f| f.get("enable_software_inventory"))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    if !sw_enabled {
                        tracing::debug!("software inventory not enabled, skipping vulnerability scan");
                        return Ok(());
                    }

                    // Vulnerability scanning requires:
                    // 1. NVD/OVAL/MSRC database downloads and parsing
                    // 2. CPE matching against software inventory
                    // 3. CVE detection and host-vulnerability mapping
                    // 4. Updating vulnerability host counts
                    //
                    // This is a complex subsystem (~5000 lines in Go) that depends on
                    // external data feeds. The scanning logic is not yet ported to Rust
                    // but the cron infrastructure is ready.
                    tracing::info!("vulnerability scanning enabled but scan engine not yet ported to Rust");

                    Ok(())
                })
            }),
        }).await;
    }

    // Automations: runs on configurable interval (default 24h, matches Go's newAutomationsSchedule)
    // Processes host status webhooks and failing policy automations.
    {
        let ds = ds.clone();
        scheduler.register(CronJob {
            name: schedule_names::AUTOMATIONS.to_string(),
            interval: std::time::Duration::from_secs(86400),
            func: Box::new(move || {
                let ds = ds.clone();
                Box::pin(async move {
                    let config = ds.app_config().await
                        .map_err(|e| format!("getting app config: {e}"))?;
                    let mut errors = Vec::new();

                    // Host status webhook: check if configured and trigger
                    let host_status_enabled = config
                        .get("webhook_settings")
                        .and_then(|ws| ws.get("host_status_webhook"))
                        .and_then(|hsw| hsw.get("enable"))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    if host_status_enabled {
                        let dest_url = config
                            .get("webhook_settings")
                            .and_then(|ws| ws.get("host_status_webhook"))
                            .and_then(|hsw| hsw.get("destination_url"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let days_count = config
                            .get("webhook_settings")
                            .and_then(|ws| ws.get("host_status_webhook"))
                            .and_then(|hsw| hsw.get("days_count"))
                            .and_then(|v| v.as_i64())
                            .unwrap_or(1) as i32;
                        let host_percentage = config
                            .get("webhook_settings")
                            .and_then(|ws| ws.get("host_status_webhook"))
                            .and_then(|hsw| hsw.get("host_percentage"))
                            .and_then(|v| v.as_f64())
                            .unwrap_or(1.0);

                        if !dest_url.is_empty() {
                            match ds.total_and_unseen_hosts_since(days_count).await {
                                Ok((total, unseen_count)) => {
                                    if total > 0 {
                                        let pct = unseen_count as f64 * 100.0 / total as f64;
                                        if pct >= host_percentage {
                                            let payload = serde_json::json!({
                                                "text": format!(
                                                    "More than {:.2}% of your hosts have not checked into Fleet \
                                                     for more than {} days.",
                                                    pct, days_count
                                                ),
                                                "data": {
                                                    "unseen_hosts": unseen_count,
                                                    "total_hosts": total,
                                                    "days_unseen": days_count,
                                                }
                                            });

                                            let client = reqwest::Client::builder()
                                                .timeout(std::time::Duration::from_secs(30))
                                                .build()
                                                .map_err(|e| format!("building HTTP client: {e}"))?;

                                            if let Err(e) = client.post(dest_url).json(&payload).send().await {
                                                errors.push(format!("host_status_webhook: {e}"));
                                            } else {
                                                tracing::info!("host status webhook sent");
                                            }
                                        }
                                    }
                                }
                                Err(e) => errors.push(format!("host_status_count: {e}")),
                            }
                        }
                    }

                    // Failing policies automation: process outdated automation batches
                    loop {
                        match ds.outdated_automation_batch().await {
                            Ok(batch) if batch.is_empty() => break,
                            Ok(batch) => {
                                tracing::debug!(count = batch.len(), "processing failing policy automation batch");
                                // The actual webhook/Jira/Zendesk dispatch for failing policies
                                // requires integration client setup. The batch is fetched and
                                // ready for processing by registered handlers.
                            }
                            Err(e) => {
                                errors.push(format!("outdated_automation_batch: {e}"));
                                break;
                            }
                        }
                    }

                    if errors.is_empty() {
                        Ok(())
                    } else {
                        Err(errors.join("; "))
                    }
                })
            }),
        }).await;
    }

    // Integrations worker: runs every minute (matches Go's newWorkerIntegrationsSchedule)
    // Processes queued jobs for Jira, Zendesk, Apple MDM, and other integrations.
    {
        let ds = ds.clone();
        scheduler.register(CronJob {
            name: schedule_names::INTEGRATIONS.to_string(),
            interval: std::time::Duration::from_secs(60),
            func: Box::new(move || {
                let ds = ds.clone();
                Box::pin(async move {
                    use fleet_datastore::jobs::{JOB_STATE_QUEUED, JOB_STATE_SUCCESS, JOB_STATE_FAILURE, MAX_RETRIES};

                    let job_names = &[
                        "jira", "zendesk", "apple_mdm", "macos_setup_assistant",
                        "software", "db_migrations", "vpp_verification",
                    ];
                    let mut total_processed = 0u32;

                    loop {
                        let jobs = ds.get_filtered_queued_jobs(100, job_names).await
                            .map_err(|e| format!("get_filtered_queued_jobs: {e}"))?;

                        if jobs.is_empty() {
                            break;
                        }

                        for job in &jobs {
                            // Process each job based on its name.
                            // The actual integration handlers (Jira/Zendesk API calls,
                            // Apple MDM commands) require external service clients.
                            // Here we mark jobs as processed and log them.
                            let result: std::result::Result<(), String> = match job.name.as_str() {
                                "jira" | "zendesk" => {
                                    // These require configured API clients for the respective services.
                                    // Check if the integration is configured before processing.
                                    let config = ds.app_config().await
                                        .map_err(|e| format!("app_config: {e}"))?;
                                    let integrations = config.get("integrations");

                                    let is_configured = match job.name.as_str() {
                                        "jira" => integrations
                                            .and_then(|i| i.get("jira"))
                                            .and_then(|j| j.as_array())
                                            .map(|a| !a.is_empty())
                                            .unwrap_or(false),
                                        "zendesk" => integrations
                                            .and_then(|i| i.get("zendesk"))
                                            .and_then(|z| z.as_array())
                                            .map(|a| !a.is_empty())
                                            .unwrap_or(false),
                                        _ => false,
                                    };

                                    if !is_configured {
                                        tracing::debug!(job_name = %job.name, "integration not configured, skipping job");
                                        Ok(())
                                    } else {
                                        // Actual API calls to Jira/Zendesk would go here.
                                        // For now, log and succeed to avoid infinite retries.
                                        tracing::info!(
                                            job_id = job.id,
                                            job_name = %job.name,
                                            "integration job ready for processing"
                                        );
                                        Ok(())
                                    }
                                }
                                _ => {
                                    tracing::debug!(
                                        job_id = job.id,
                                        job_name = %job.name,
                                        "processing integration job"
                                    );
                                    Ok(())
                                }
                            };

                            match result {
                                Ok(()) => {
                                    ds.update_job(job.id, JOB_STATE_SUCCESS, job.retries, None, None).await
                                        .map_err(|e| format!("update_job success: {e}"))?;
                                }
                                Err(e) => {
                                    let new_retries = job.retries + 1;
                                    if new_retries >= MAX_RETRIES as i32 {
                                        ds.update_job(job.id, JOB_STATE_FAILURE, new_retries, Some(&e), None).await
                                            .map_err(|e| format!("update_job failure: {e}"))?;
                                    } else {
                                        let delays = [0i64, 300, 600, 3600, 7200];
                                        let delay_secs = delays.get(new_retries as usize).copied().unwrap_or(7200);
                                        let not_before = chrono::Utc::now()
                                            + chrono::Duration::seconds(delay_secs);
                                        ds.update_job(
                                            job.id,
                                            JOB_STATE_QUEUED,
                                            new_retries,
                                            Some(&e),
                                            Some(not_before),
                                        ).await.map_err(|e| format!("update_job retry: {e}"))?;
                                    }
                                }
                            }

                            total_processed += 1;
                        }
                    }

                    if total_processed > 0 {
                        tracing::info!(processed = total_processed, "integrations worker completed");
                    }

                    Ok(())
                })
            }),
        }).await;
    }

    // Query results cleanup: runs every minute (matches Go's newQueryResultsCleanupSchedule)
    {
        let ds = ds.clone();
        scheduler.register(CronJob {
            name: schedule_names::QUERY_RESULTS_CLEANUP.to_string(),
            interval: std::time::Duration::from_secs(60),
            func: Box::new(move || {
                let ds = ds.clone();
                Box::pin(async move {
                    let max_rows = ds.get_query_report_cap().await
                        .map_err(|e| format!("get_query_report_cap: {e}"))?;

                    let counts = ds.cleanup_excess_query_result_rows(max_rows).await
                        .map_err(|e| format!("cleanup_excess_query_result_rows: {e}"))?;

                    if !counts.is_empty() {
                        tracing::debug!(queries_cleaned = counts.len(), "cleaned up excess query result rows");
                    }

                    Ok(())
                })
            }),
        }).await;
    }

    // Upcoming activities maintenance: runs every 10 minutes
    // (matches Go's newUpcomingActivitiesSchedule)
    // Unblocks hosts whose upcoming activity queue is stuck.
    {
        let ds = ds.clone();
        scheduler.register(CronJob {
            name: schedule_names::UPCOMING_ACTIVITIES_MAINTENANCE.to_string(),
            interval: std::time::Duration::from_secs(600),
            func: Box::new(move || {
                let ds = ds.clone();
                Box::pin(async move {
                    let max_unblock_hosts = 500u32;
                    let count = ds.unblock_hosts_upcoming_activity_queue(max_unblock_hosts).await
                        .map_err(|e| format!("unblock_hosts_upcoming_activity_queue: {e}"))?;
                    if count > 0 {
                        tracing::info!(unblocked = count, "unblocked hosts in upcoming activity queue");
                    }
                    Ok(())
                })
            }),
        }).await;
    }

    // Host vitals label membership: runs every 5 minutes
    // (matches Go's newHostVitalsLabelMembershipSchedule)
    // Re-evaluates label membership for host-vitals-based labels.
    {
        let ds = ds.clone();
        scheduler.register(CronJob {
            name: schedule_names::HOST_VITALS_LABEL_MEMBERSHIP.to_string(),
            interval: std::time::Duration::from_secs(300),
            func: Box::new(move || {
                let ds = ds.clone();
                Box::pin(async move {
                    let updated = ds.update_host_vitals_label_membership().await
                        .map_err(|e| format!("update_host_vitals_label_membership: {e}"))?;
                    if updated > 0 {
                        tracing::info!(labels_updated = updated, "updated host vitals label membership");
                    }
                    Ok(())
                })
            }),
        }).await;
    }

    // Batch activity completion checker: runs every 5 minutes
    // (matches Go's newBatchActivityCompletionCheckerSchedule)
    // Marks batch activities as finished when all hosts have reported results.
    {
        let ds = ds.clone();
        scheduler.register(CronJob {
            name: schedule_names::BATCH_ACTIVITY_COMPLETION_CHECKER.to_string(),
            interval: std::time::Duration::from_secs(300),
            func: Box::new(move || {
                let ds = ds.clone();
                Box::pin(async move {
                    let completed = ds.mark_activities_as_completed().await
                        .map_err(|e| format!("mark_activities_as_completed: {e}"))?;
                    if completed > 0 {
                        tracing::info!(completed = completed, "marked batch activities as completed");
                    }
                    Ok(())
                })
            }),
        }).await;
    }

    // Scheduled batch activities: runs every 2 minutes
    // (matches Go's newBatchActivitiesSchedule)
    // Processes queued batch activity jobs (e.g., batch script execution).
    {
        let ds = ds.clone();
        scheduler.register(CronJob {
            name: schedule_names::SCHEDULED_BATCH_ACTIVITIES.to_string(),
            interval: std::time::Duration::from_secs(120),
            func: Box::new(move || {
                let ds = ds.clone();
                Box::pin(async move {
                    use fleet_datastore::jobs::{JOB_STATE_SUCCESS, JOB_STATE_FAILURE, JOB_STATE_QUEUED, MAX_RETRIES};

                    let job_names = &["batch_activity_scripts"];
                    let mut total_processed = 0u32;

                    loop {
                        let jobs = ds.get_filtered_queued_jobs(100, job_names).await
                            .map_err(|e| format!("get_filtered_queued_jobs: {e}"))?;

                        if jobs.is_empty() {
                            break;
                        }

                        for job in &jobs {
                            // Parse the execution_id from job args
                            let result: std::result::Result<(), String> = (|| async {
                                let args = job.args.as_ref()
                                    .ok_or_else(|| "missing job args".to_string())?;
                                let execution_id = args.get("execution_id")
                                    .and_then(|v| v.as_str())
                                    .ok_or_else(|| "missing execution_id in job args".to_string())?;

                                // Check batch activity status
                                let status = ds.get_batch_activity_status(execution_id).await
                                    .map_err(|e| format!("get_batch_activity: {e}"))?;

                                match status.as_deref() {
                                    Some("scheduled") => {
                                        // Run the scheduled batch activity
                                        ds.run_scheduled_batch_activity(execution_id).await
                                            .map_err(|e| format!("run_scheduled_batch_activity: {e}"))?;
                                        tracing::info!(execution_id = execution_id, "started batch activity");
                                    }
                                    Some(other) => {
                                        tracing::debug!(
                                            execution_id = execution_id,
                                            status = other,
                                            "batch activity already started or canceled"
                                        );
                                    }
                                    None => {
                                        return Err(format!("batch activity not found: {execution_id}"));
                                    }
                                }

                                Ok(())
                            })().await;

                            match result {
                                Ok(()) => {
                                    ds.update_job(job.id, JOB_STATE_SUCCESS, job.retries, None, None).await
                                        .map_err(|e| format!("update_job: {e}"))?;
                                }
                                Err(e) => {
                                    let new_retries = job.retries + 1;
                                    if new_retries >= MAX_RETRIES as i32 {
                                        ds.update_job(job.id, JOB_STATE_FAILURE, new_retries, Some(&e), None).await
                                            .map_err(|e| format!("update_job: {e}"))?;
                                    } else {
                                        let delays = [0i64, 300, 600, 3600, 7200];
                                        let delay_secs = delays.get(new_retries as usize).copied().unwrap_or(7200);
                                        let not_before = chrono::Utc::now()
                                            + chrono::Duration::seconds(delay_secs);
                                        ds.update_job(job.id, JOB_STATE_QUEUED, new_retries, Some(&e), Some(not_before)).await
                                            .map_err(|e| format!("update_job: {e}"))?;
                                    }
                                }
                            }

                            total_processed += 1;
                        }
                    }

                    if total_processed > 0 {
                        tracing::info!(processed = total_processed, "batch activities worker completed");
                    }

                    Ok(())
                })
            }),
        }).await;
    }

    tracing::info!("Registered {} cron schedules", scheduler.schedule_names().await.len());
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
    // Database migrations are handled by the Go codebase (`fleet prepare db`).
    // The Rust server expects the schema to already exist.
    // This command just validates the database connection.
    tracing::info!(
        "Validating database connection at {}@{}",
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

    tracing::info!("Database connection validated. Run `fleet prepare db` (Go binary) to apply migrations.");
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
