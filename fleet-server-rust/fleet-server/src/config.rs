//! Configuration loading for Fleet server.
//!
//! Loads config from YAML files and environment variables (FLEET_ prefix),
//! matching the Go server's configuration structure and defaults.

use serde::Deserialize;
use std::time::Duration;

/// Top-level Fleet configuration, matching the Go `FleetConfig` struct.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FleetConfig {
    pub mysql: MysqlConfig,
    pub mysql_read_replica: MysqlConfig,
    pub redis: RedisConfig,
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub app: AppConfig,
    pub session: SessionConfig,
    pub osquery: OsqueryConfig,
    pub activity: ActivityConfig,
    pub logging: LoggingConfig,
    pub firehose: FirehoseConfig,
    pub kinesis: KinesisConfig,
    pub lambda: LambdaConfig,
    pub s3: S3Config,
    pub email: EmailConfig,
    pub ses: SesConfig,
    pub filesystem: FilesystemConfig,
    pub webhook: WebhookConfig,
    pub license: LicenseConfig,
    pub vulnerabilities: VulnerabilitiesConfig,
    pub sentry: SentryConfig,
    pub geoip: GeoIPConfig,
    pub prometheus: PrometheusConfig,
    pub mdm: MdmConfig,
    pub calendar: CalendarConfig,
}

impl Default for FleetConfig {
    fn default() -> Self {
        Self {
            mysql: MysqlConfig::default(),
            mysql_read_replica: MysqlConfig {
                address: String::new(),
                ..MysqlConfig::default()
            },
            redis: RedisConfig::default(),
            server: ServerConfig::default(),
            auth: AuthConfig::default(),
            app: AppConfig::default(),
            session: SessionConfig::default(),
            osquery: OsqueryConfig::default(),
            activity: ActivityConfig::default(),
            logging: LoggingConfig::default(),
            firehose: FirehoseConfig::default(),
            kinesis: KinesisConfig::default(),
            lambda: LambdaConfig::default(),
            s3: S3Config::default(),
            email: EmailConfig::default(),
            ses: SesConfig::default(),
            filesystem: FilesystemConfig::default(),
            webhook: WebhookConfig::default(),
            license: LicenseConfig::default(),
            vulnerabilities: VulnerabilitiesConfig::default(),
            sentry: SentryConfig::default(),
            geoip: GeoIPConfig::default(),
            prometheus: PrometheusConfig::default(),
            mdm: MdmConfig::default(),
            calendar: CalendarConfig::default(),
        }
    }
}

impl FleetConfig {
    /// Load configuration from an optional YAML file path and environment
    /// variables with the `FLEET_` prefix.
    pub fn load(config_path: Option<&str>) -> anyhow::Result<Self> {
        let mut builder = config::Config::builder();

        // Start with defaults
        builder = builder.add_source(config::Config::try_from(&FleetConfig::default())?);

        // Layer on config file if provided
        if let Some(path) = config_path {
            builder = builder.add_source(config::File::with_name(path).required(true));
        }

        // Layer on environment variables with FLEET_ prefix
        // e.g. FLEET_MYSQL_ADDRESS maps to mysql.address
        builder = builder.add_source(
            config::Environment::with_prefix("FLEET")
                .separator("_")
                .try_parsing(true),
        );

        let cfg: FleetConfig = builder.build()?.try_deserialize()?;
        Ok(cfg)
    }

    /// Returns true if OTEL tracing is enabled (tracing is on and type is not elasticapm).
    pub fn otel_enabled(&self) -> bool {
        self.logging.tracing_enabled && self.logging.tracing_type != "elasticapm"
    }
}

// Serialize is needed for config::Config::try_from in defaults
impl serde::Serialize for FleetConfig {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("FleetConfig", 24)?;
        state.serialize_field("mysql", &self.mysql)?;
        state.serialize_field("mysql_read_replica", &self.mysql_read_replica)?;
        state.serialize_field("redis", &self.redis)?;
        state.serialize_field("server", &self.server)?;
        state.serialize_field("auth", &self.auth)?;
        state.serialize_field("app", &self.app)?;
        state.serialize_field("session", &self.session)?;
        state.serialize_field("osquery", &self.osquery)?;
        state.serialize_field("activity", &self.activity)?;
        state.serialize_field("logging", &self.logging)?;
        state.serialize_field("firehose", &self.firehose)?;
        state.serialize_field("kinesis", &self.kinesis)?;
        state.serialize_field("lambda", &self.lambda)?;
        state.serialize_field("s3", &self.s3)?;
        state.serialize_field("email", &self.email)?;
        state.serialize_field("ses", &self.ses)?;
        state.serialize_field("filesystem", &self.filesystem)?;
        state.serialize_field("webhook", &self.webhook)?;
        state.serialize_field("license", &self.license)?;
        state.serialize_field("vulnerabilities", &self.vulnerabilities)?;
        state.serialize_field("sentry", &self.sentry)?;
        state.serialize_field("geoip", &self.geoip)?;
        state.serialize_field("prometheus", &self.prometheus)?;
        state.serialize_field("mdm", &self.mdm)?;
        state.serialize_field("calendar", &self.calendar)?;
        state.end()
    }
}

/// MySQL database configuration.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MysqlConfig {
    pub protocol: String,
    pub address: String,
    pub username: String,
    pub password: String,
    pub password_path: String,
    pub database: String,
    pub tls_cert: String,
    pub tls_key: String,
    pub tls_ca: String,
    pub tls_server_name: String,
    pub tls_config: String,
    pub max_open_conns: u32,
    pub max_idle_conns: u32,
    pub conn_max_lifetime: u64,
    pub sql_mode: String,
    pub region: String,
    pub sts_assume_role_arn: String,
    pub sts_external_id: String,
}

impl Default for MysqlConfig {
    fn default() -> Self {
        Self {
            protocol: "tcp".to_string(),
            address: "localhost:3306".to_string(),
            username: "fleet".to_string(),
            password: String::new(),
            password_path: String::new(),
            database: "fleet".to_string(),
            tls_cert: String::new(),
            tls_key: String::new(),
            tls_ca: String::new(),
            tls_server_name: String::new(),
            tls_config: String::new(),
            max_open_conns: 50,
            max_idle_conns: 50,
            conn_max_lifetime: 0,
            sql_mode: String::new(),
            region: String::new(),
            sts_assume_role_arn: String::new(),
            sts_external_id: String::new(),
        }
    }
}

/// Redis configuration.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RedisConfig {
    pub address: String,
    pub username: String,
    pub password: String,
    pub database: u32,
    pub region: String,
    pub cache_name: String,
    pub use_tls: bool,
    pub duplicate_results: bool,
    pub connect_timeout_secs: u64,
    pub keep_alive_secs: u64,
    pub connect_retry_attempts: u32,
    pub cluster_follow_redirections: bool,
    pub cluster_read_from_replica: bool,
    pub tls_cert: String,
    pub tls_key: String,
    pub tls_ca: String,
    pub tls_server_name: String,
    pub tls_handshake_timeout_secs: u64,
    pub max_idle_conns: u32,
    pub max_open_conns: u32,
    pub conn_max_lifetime_secs: u64,
    pub idle_timeout_secs: u64,
    pub conn_wait_timeout_secs: u64,
    pub write_timeout_secs: u64,
    pub read_timeout_secs: u64,
    pub sts_assume_role_arn: String,
    pub sts_external_id: String,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            address: "localhost:6379".to_string(),
            username: String::new(),
            password: String::new(),
            database: 0,
            region: String::new(),
            cache_name: String::new(),
            use_tls: false,
            duplicate_results: false,
            connect_timeout_secs: 5,
            keep_alive_secs: 10,
            connect_retry_attempts: 0,
            cluster_follow_redirections: true,
            cluster_read_from_replica: false,
            tls_cert: String::new(),
            tls_key: String::new(),
            tls_ca: String::new(),
            tls_server_name: String::new(),
            tls_handshake_timeout_secs: 10,
            max_idle_conns: 3,
            max_open_conns: 0,
            conn_max_lifetime_secs: 0,
            idle_timeout_secs: 240,
            conn_wait_timeout_secs: 0,
            write_timeout_secs: 10,
            read_timeout_secs: 10,
            sts_assume_role_arn: String::new(),
            sts_external_id: String::new(),
        }
    }
}

/// Server configuration.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ServerConfig {
    pub address: String,
    pub cert: String,
    pub key: String,
    pub tls: bool,
    pub tls_compatibility: String,
    pub url_prefix: String,
    pub keepalive: bool,
    pub sandbox_enabled: bool,
    pub websockets_allow_unsafe_origin: bool,
    pub frequent_cleanups_enabled: bool,
    pub force_h2c: bool,
    pub private_key: String,
    pub private_key_arn: String,
    pub private_key_region: String,
    pub private_key_sts_assume_role_arn: String,
    pub private_key_sts_external_id: String,
    pub vpp_verify_timeout_secs: u64,
    pub vpp_verify_request_delay_secs: u64,
    pub cleanup_dist_targets_age_secs: u64,
    pub max_installer_size: i64,
    pub trusted_proxies: String,
    pub gzip_responses: bool,
    pub default_max_request_body_size: i64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0:8080".to_string(),
            cert: "./tools/osquery/fleet.crt".to_string(),
            key: "./tools/osquery/fleet.key".to_string(),
            tls: true,
            tls_compatibility: "intermediate".to_string(),
            url_prefix: String::new(),
            keepalive: true,
            sandbox_enabled: false,
            websockets_allow_unsafe_origin: false,
            frequent_cleanups_enabled: false,
            force_h2c: false,
            private_key: String::new(),
            private_key_arn: String::new(),
            private_key_region: String::new(),
            private_key_sts_assume_role_arn: String::new(),
            private_key_sts_external_id: String::new(),
            vpp_verify_timeout_secs: 600,       // 10 minutes
            vpp_verify_request_delay_secs: 5,
            cleanup_dist_targets_age_secs: 86400, // 24 hours
            max_installer_size: 3_221_225_472,   // ~3GiB
            trusted_proxies: String::new(),
            gzip_responses: false,
            default_max_request_body_size: 524_288, // 512KB
        }
    }
}

/// Auth configuration.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AuthConfig {
    pub bcrypt_cost: u32,
    pub salt_key_size: usize,
    pub sso_session_validity_period_secs: u64,
    pub require_http_message_signature: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            bcrypt_cost: 12,
            salt_key_size: 24,
            sso_session_validity_period_secs: 300, // 5 minutes
            require_http_message_signature: false,
        }
    }
}

/// App configuration.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AppConfig {
    pub token_key_size: usize,
    pub invite_token_validity_period_secs: u64,
    pub enable_scheduled_query_stats: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            token_key_size: 24,
            invite_token_validity_period_secs: 432_000, // 5 days
            enable_scheduled_query_stats: true,
        }
    }
}

/// Session configuration.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SessionConfig {
    pub key_size: usize,
    pub duration_secs: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            key_size: 64,
            duration_secs: 432_000, // 5 days
        }
    }
}

/// Osquery configuration.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OsqueryConfig {
    pub node_key_size: usize,
    pub host_identifier: String,
    pub enroll_cooldown_secs: u64,
    pub status_log_plugin: String,
    pub result_log_plugin: String,
    pub label_update_interval_secs: u64,
    pub policy_update_interval_secs: u64,
    pub detail_update_interval_secs: u64,
    pub enable_log_rotation: bool,
    pub max_jitter_percent: u32,
    pub enable_async_host_processing: String,
    pub async_host_collect_interval_secs: u64,
    pub async_host_collect_max_jitter_percent: u32,
    pub async_host_collect_lock_timeout_secs: u64,
    pub async_host_collect_log_stats_interval_secs: u64,
    pub async_host_insert_batch: u32,
    pub async_host_delete_batch: u32,
    pub async_host_update_batch: u32,
    pub async_host_redis_pop_count: u32,
    pub async_host_redis_scan_keys_count: u32,
    pub min_software_last_opened_at_diff_secs: u64,
}

impl Default for OsqueryConfig {
    fn default() -> Self {
        Self {
            node_key_size: 24,
            host_identifier: "provided".to_string(),
            enroll_cooldown_secs: 0,
            status_log_plugin: "filesystem".to_string(),
            result_log_plugin: "filesystem".to_string(),
            label_update_interval_secs: 3600,   // 1 hour
            policy_update_interval_secs: 3600,  // 1 hour
            detail_update_interval_secs: 3600,  // 1 hour
            enable_log_rotation: false,
            max_jitter_percent: 10,
            enable_async_host_processing: "false".to_string(),
            async_host_collect_interval_secs: 30,
            async_host_collect_max_jitter_percent: 10,
            async_host_collect_lock_timeout_secs: 60,
            async_host_collect_log_stats_interval_secs: 60,
            async_host_insert_batch: 2000,
            async_host_delete_batch: 2000,
            async_host_update_batch: 1000,
            async_host_redis_pop_count: 1000,
            async_host_redis_scan_keys_count: 1000,
            min_software_last_opened_at_diff_secs: 120, // 2 minutes
        }
    }
}

/// Activity configuration.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ActivityConfig {
    pub enable_audit_log: bool,
    pub audit_log_plugin: String,
}

impl Default for ActivityConfig {
    fn default() -> Self {
        Self {
            enable_audit_log: false,
            audit_log_plugin: "filesystem".to_string(),
        }
    }
}

/// Logging configuration.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub debug: bool,
    pub json: bool,
    pub disable_banner: bool,
    pub error_retention_period_secs: u64,
    pub tracing_enabled: bool,
    pub tracing_type: String,
    pub otel_logs_enabled: bool,
    pub enable_topics: String,
    pub disable_topics: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            debug: false,
            json: false,
            disable_banner: false,
            error_retention_period_secs: 86400, // 24 hours
            tracing_enabled: false,
            tracing_type: String::new(),
            otel_logs_enabled: false,
            enable_topics: String::new(),
            disable_topics: String::new(),
        }
    }
}

/// AWS Firehose logging configuration.
#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct FirehoseConfig {
    pub region: String,
    pub endpoint_url: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub sts_assume_role_arn: String,
    pub sts_external_id: String,
    pub status_stream: String,
    pub result_stream: String,
    pub audit_stream: String,
}

/// AWS Kinesis logging configuration.
#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct KinesisConfig {
    pub region: String,
    pub endpoint_url: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub sts_assume_role_arn: String,
    pub sts_external_id: String,
    pub status_stream: String,
    pub result_stream: String,
    pub audit_stream: String,
}

/// AWS Lambda logging configuration.
#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LambdaConfig {
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub sts_assume_role_arn: String,
    pub sts_external_id: String,
    pub status_function: String,
    pub result_function: String,
    pub audit_function: String,
}

/// S3 storage configuration.
#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct S3Config {
    pub bucket: String,
    pub prefix: String,
    pub region: String,
    pub endpoint_url: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub sts_assume_role_arn: String,
    pub sts_external_id: String,
    pub disable_ssl: bool,
    pub force_s3_path_style: bool,

    pub carves_bucket: String,
    pub carves_prefix: String,
    pub carves_region: String,
    pub carves_endpoint_url: String,
    pub carves_access_key_id: String,
    pub carves_secret_access_key: String,
    pub carves_disable_ssl: bool,
    pub carves_force_s3_path_style: bool,

    pub software_installers_bucket: String,
    pub software_installers_prefix: String,
    pub software_installers_region: String,
    pub software_installers_endpoint_url: String,
    pub software_installers_access_key_id: String,
    pub software_installers_secret_access_key: String,
    pub software_installers_disable_ssl: bool,
    pub software_installers_force_s3_path_style: bool,
    pub software_installers_cloudfront_url: String,
}

/// Email configuration.
#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct EmailConfig {
    pub backend: String,
}

/// AWS SES email configuration.
#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SesConfig {
    pub region: String,
    pub endpoint_url: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub sts_assume_role_arn: String,
    pub sts_external_id: String,
    pub source_arn: String,
}

/// Filesystem logging configuration.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct FilesystemConfig {
    pub status_log_file: String,
    pub result_log_file: String,
    pub audit_log_file: String,
    pub enable_log_rotation: bool,
    pub enable_log_compression: bool,
    pub max_size: u32,
    pub max_age: u32,
    pub max_backups: u32,
}

impl Default for FilesystemConfig {
    fn default() -> Self {
        Self {
            status_log_file: "/tmp/osquery_status".to_string(),
            result_log_file: "/tmp/osquery_result".to_string(),
            audit_log_file: "/tmp/fleet_audit".to_string(),
            enable_log_rotation: false,
            enable_log_compression: false,
            max_size: 500,   // 500 MB
            max_age: 28,     // 28 days
            max_backups: 3,
        }
    }
}

/// Webhook configuration.
#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct WebhookConfig {
    pub host_status_webhook_url: String,
}

/// License configuration.
#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LicenseConfig {
    pub key: String,
    pub enforce_host_limit: bool,
}

/// Vulnerabilities processing configuration.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct VulnerabilitiesConfig {
    pub databases_path: String,
    pub periodicity_secs: u64,
    pub cpe_database_url: String,
    pub cpe_translations_url: String,
    pub cve_feed_prefix_url: String,
    pub disable_data_sync: bool,
    pub recent_vulnerability_max_age_days: u32,
    pub disable_win_os_vulnerabilities: bool,
}

impl Default for VulnerabilitiesConfig {
    fn default() -> Self {
        Self {
            databases_path: String::new(),
            periodicity_secs: 3600,
            cpe_database_url: String::new(),
            cpe_translations_url: String::new(),
            cve_feed_prefix_url: String::new(),
            disable_data_sync: false,
            recent_vulnerability_max_age_days: 30,
            disable_win_os_vulnerabilities: false,
        }
    }
}

/// Sentry error reporting configuration.
#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SentryConfig {
    pub dsn: String,
}

/// GeoIP configuration.
#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct GeoIPConfig {
    pub database_path: String,
}

/// Prometheus metrics configuration.
#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PrometheusConfig {
    pub basic_auth: PrometheusBasicAuth,
}

#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PrometheusBasicAuth {
    pub username: String,
    pub password: String,
}

/// MDM configuration.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MdmConfig {
    pub apple_scep_signer_validity_days: u32,
    pub apple_scep_signer_allow_renewal_days: u32,
    pub apple_scep_challenge: String,
    pub windows_wstep_identity_cert: String,
    pub windows_wstep_identity_key: String,
}

impl Default for MdmConfig {
    fn default() -> Self {
        Self {
            apple_scep_signer_validity_days: 365,
            apple_scep_signer_allow_renewal_days: 14,
            apple_scep_challenge: String::new(),
            windows_wstep_identity_cert: String::new(),
            windows_wstep_identity_key: String::new(),
        }
    }
}

/// Calendar integration configuration.
#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CalendarConfig {
    pub periodicity_secs: u64,
}
