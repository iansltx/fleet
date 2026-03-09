//! Agent Options types matching Go's `server/fleet/agent_options.go` and
//! `server/fleet/agent_options_generated.go`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ─── Top-level Agent Options ─────────────────────────────────────────────────

/// AgentOptions represents osquery agent options configuration.
/// Matches Go's `AgentOptions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOptions {
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub script_execution_timeout: i32,
    pub config: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overrides: Option<AgentOptionsOverrides>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_line_flags: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update_channels: Option<serde_json::Value>,
}

/// AgentOptionsOverrides includes any platform-based overrides.
/// Matches Go's `AgentOptionsOverrides`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentOptionsOverrides {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platforms: Option<HashMap<String, serde_json::Value>>,
}

fn is_zero_i32(v: &i32) -> bool {
    *v == 0
}

// ─── Extension Info ──────────────────────────────────────────────────────────

/// ExtensionInfo holds the data of an osquery extension to apply to an Orbit client.
/// Matches Go's `ExtensionInfo` (from orbit.go).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    /// Platform is one of "windows", "linux" or "macos".
    pub platform: String,
    /// Channel is the selected TUF channel to listen for updates.
    pub channel: String,
    /// Labels are the label names the host must be a member of to run this extension.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
}

// ─── Orbit Update Channels ──────────────────────────────────────────────────

/// OrbitUpdateChannels holds the update channels that can be configured in fleetd agents.
/// Matches Go's `OrbitUpdateChannels` (from orbit.go).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrbitUpdateChannels {
    /// Orbit holds the orbit channel.
    #[serde(default)]
    pub orbit: String,
    /// Osqueryd holds the osqueryd channel.
    #[serde(default)]
    pub osqueryd: String,
    /// Desktop holds the Fleet Desktop channel.
    #[serde(default)]
    pub desktop: String,
}

// ─── Osquery Agent Options (config-level) ────────────────────────────────────

/// JSON definition of the available configuration options in osquery.
/// Matches Go's `osqueryAgentOptions`.
///
/// See <https://osquery.readthedocs.io/en/stable/deployment/configuration/#configuration-specification>
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OsqueryAgentOptions {
    #[serde(default)]
    pub options: OsqueryOptions,

    /// Schedule is allowed as a top-level key but its value is not validated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schedule: Option<serde_json::Value>,

    /// Packs is allowed as a top-level key but its value is not validated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packs: Option<serde_json::Value>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_paths: Option<HashMap<String, Vec<String>>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_accesses: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_paths_query: Option<HashMap<String, Vec<String>>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_paths: Option<HashMap<String, Vec<String>>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yara: Option<OsqueryYara>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prometheus_targets: Option<OsqueryPrometheusTargets>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub views: Option<HashMap<String, String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorators: Option<OsqueryDecorators>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_table_construction: Option<HashMap<String, OsqueryAutoTableConstruction>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<OsqueryEvents>,
}

/// YARA configuration for osquery.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OsqueryYara {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signatures: Option<HashMap<String, Vec<String>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_paths: Option<HashMap<String, Vec<String>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_urls: Option<Vec<String>>,
}

/// Prometheus targets configuration for osquery.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OsqueryPrometheusTargets {
    #[serde(default)]
    pub timeout: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<String>>,
}

/// Decorators configuration for osquery.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OsqueryDecorators {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub always: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval: Option<HashMap<String, Vec<String>>>,
}

/// Auto table construction entry for osquery.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OsqueryAutoTableConstruction {
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<String>>,
    #[serde(default)]
    pub platform: String,
}

/// Events configuration for osquery.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OsqueryEvents {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_subscribers: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_subscribers: Option<Vec<String>>,
}

// ─── Osquery Options (generated from osquery 5.21.0) ─────────────────────────

/// Osquery options that can be set in the config's "options" key.
/// Matches Go's `osqueryOptions` (from agent_options_generated.go).
///
/// This struct includes all cross-platform options as well as embedded
/// OS-specific fields via `#[serde(flatten)]`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OsqueryOptions {
    #[serde(default)]
    pub audit_allow_config: bool,
    #[serde(default)]
    pub audit_allow_fim_events: bool,
    #[serde(default)]
    pub audit_allow_process_events: bool,
    #[serde(default)]
    pub audit_allow_sockets: bool,
    #[serde(default)]
    pub audit_allow_user_events: bool,
    #[serde(default)]
    pub augeas_lenses: String,
    #[serde(default)]
    pub aws_access_key_id: String,
    #[serde(default)]
    pub aws_debug: bool,
    #[serde(default)]
    pub aws_disable_imdsv1_fallback: bool,
    #[serde(default)]
    pub aws_enable_proxy: bool,
    #[serde(default)]
    pub aws_firehose_endpoint: String,
    #[serde(default)]
    pub aws_firehose_period: u64,
    #[serde(default)]
    pub aws_firehose_region: String,
    #[serde(default)]
    pub aws_firehose_stream: String,
    #[serde(default)]
    pub aws_imdsv2_request_attempts: u32,
    #[serde(default)]
    pub aws_imdsv2_request_interval: u32,
    #[serde(default)]
    pub aws_kinesis_disable_log_status: bool,
    #[serde(default)]
    pub aws_kinesis_endpoint: String,
    #[serde(default)]
    pub aws_kinesis_period: u64,
    #[serde(default)]
    pub aws_kinesis_random_partition_key: bool,
    #[serde(default)]
    pub aws_kinesis_region: String,
    #[serde(default)]
    pub aws_kinesis_stream: String,
    #[serde(default)]
    pub aws_profile_name: String,
    #[serde(default)]
    pub aws_proxy_host: String,
    #[serde(default)]
    pub aws_proxy_password: String,
    #[serde(default)]
    pub aws_proxy_port: u32,
    #[serde(default)]
    pub aws_proxy_scheme: String,
    #[serde(default)]
    pub aws_proxy_username: String,
    #[serde(default)]
    pub aws_region: String,
    #[serde(default)]
    pub aws_secret_access_key: String,
    #[serde(default)]
    pub aws_session_token: String,
    #[serde(default)]
    pub aws_sts_arn_role: String,
    #[serde(default)]
    pub aws_sts_region: String,
    #[serde(default)]
    pub aws_sts_session_name: String,
    #[serde(default)]
    pub aws_sts_timeout: u64,
    #[serde(default)]
    pub buffered_log_max: u64,
    #[serde(default)]
    pub decorations_top_level: bool,
    #[serde(default)]
    pub disable_audit: bool,
    #[serde(default)]
    pub disable_caching: bool,
    #[serde(default)]
    pub disable_database: bool,
    #[serde(default)]
    pub disable_decorators: bool,
    #[serde(default)]
    pub disable_distributed: bool,
    #[serde(default)]
    pub disable_events: bool,
    #[serde(default)]
    pub disable_hash_cache: bool,
    #[serde(default)]
    pub disable_logging: bool,
    #[serde(default)]
    pub distributed_denylist_duration: u64,
    #[serde(default)]
    pub distributed_interval: u64,
    #[serde(default)]
    pub distributed_loginfo: bool,
    #[serde(default)]
    pub distributed_plugin: String,
    #[serde(default)]
    pub distributed_tls_max_attempts: u64,
    #[serde(default)]
    pub distributed_tls_read_endpoint: String,
    #[serde(default)]
    pub distributed_tls_write_endpoint: String,
    #[serde(default)]
    pub dns_resolver_refresh_interval: i32,
    #[serde(default)]
    pub docker_socket: String,
    #[serde(default)]
    pub enable_file_events: bool,
    #[serde(default)]
    pub enable_foreign: bool,
    #[serde(default)]
    pub enable_numeric_monitoring: bool,
    #[serde(default)]
    pub ephemeral: bool,
    #[serde(default)]
    pub es_fim_enable_open_events: bool,
    #[serde(default)]
    pub events_expiry: u64,
    #[serde(default)]
    pub events_max: u64,
    #[serde(default)]
    pub events_optimize: bool,
    #[serde(default)]
    pub experiment_list: String,
    #[serde(default)]
    pub extensions_default_index: bool,
    #[serde(default)]
    pub hash_cache_max: u32,
    #[serde(default)]
    pub host_identifier: String,
    #[serde(default)]
    pub ignore_table_exceptions: bool,
    #[serde(default)]
    pub keychain_access_cache: bool,
    #[serde(default)]
    pub keychain_access_interval: u32,
    #[serde(default)]
    pub logger_event_type: bool,
    #[serde(default)]
    pub logger_kafka_acks: String,
    #[serde(default)]
    pub logger_kafka_brokers: String,
    #[serde(default)]
    pub logger_kafka_compression: String,
    #[serde(default)]
    pub logger_kafka_topic: String,
    #[serde(default)]
    pub logger_min_status: i32,
    #[serde(default)]
    pub logger_min_stderr: i32,
    #[serde(default)]
    pub logger_numerics: bool,
    #[serde(default)]
    pub logger_path: String,
    #[serde(default)]
    pub logger_rotate: bool,
    #[serde(default)]
    pub logger_rotate_max_files: u64,
    #[serde(default)]
    pub logger_rotate_size: u64,
    #[serde(default)]
    pub logger_snapshot_event_type: bool,
    #[serde(default)]
    pub logger_syslog_facility: i32,
    #[serde(default)]
    pub logger_syslog_prepend_cee: bool,
    #[serde(default)]
    pub logger_tls_backoff_max: u64,
    #[serde(default)]
    pub logger_tls_compress: bool,
    #[serde(default)]
    pub logger_tls_endpoint: String,
    #[serde(default)]
    pub logger_tls_max_lines: u64,
    #[serde(default)]
    pub logger_tls_max_linesize: u64,
    #[serde(default)]
    pub logger_tls_period: u64,
    #[serde(default)]
    pub nullvalue: String,
    #[serde(default)]
    pub numeric_monitoring_filesystem_path: String,
    #[serde(default)]
    pub numeric_monitoring_plugins: String,
    #[serde(default)]
    pub numeric_monitoring_pre_aggregation_time: u64,
    #[serde(default)]
    pub pack_delimiter: String,
    #[serde(default)]
    pub pack_refresh_interval: u64,
    #[serde(default)]
    pub read_max: u64,
    #[serde(default)]
    pub schedule_default_interval: u64,
    #[serde(default)]
    pub schedule_epoch: u64,
    #[serde(default)]
    pub schedule_lognames: bool,
    #[serde(default)]
    pub schedule_max_drift: u64,
    #[serde(default)]
    pub schedule_reload: u64,
    #[serde(default)]
    pub schedule_splay_percent: u64,
    #[serde(default)]
    pub schedule_timeout: u64,
    #[serde(default)]
    pub specified_identifier: String,
    #[serde(default)]
    pub table_delay: u64,
    #[serde(default)]
    pub thrift_string_size_limit: i32,
    #[serde(default)]
    pub thrift_timeout: u32,
    #[serde(default)]
    pub thrift_verbose: bool,
    #[serde(default)]
    pub tls_disable_status_log: bool,
    #[serde(default)]
    pub verbose: bool,
    #[serde(default)]
    pub yara_delay: u32,
    #[serde(default)]
    pub yara_sigurl_authenticate: bool,

    // OS-specific flags (flattened from embedded structs in Go)
    #[serde(flatten)]
    pub linux_flags: OsqueryCommandLineFlagsLinux,
    #[serde(flatten)]
    pub windows_flags: OsqueryCommandLineFlagsWindows,
    #[serde(flatten)]
    pub macos_flags: OsqueryCommandLineFlagsMacOS,
    #[serde(flatten)]
    pub hidden_flags: OsqueryCommandLineFlagsHidden,
}

// ─── Osquery Command-Line Flags ──────────────────────────────────────────────

/// Full set of osquery command-line flags.
/// Matches Go's `osqueryCommandLineFlags` (from agent_options_generated.go).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OsqueryCommandLineFlags {
    #[serde(default)]
    pub alarm_timeout: u64,
    #[serde(default)]
    pub audit_allow_config: bool,
    #[serde(default)]
    pub audit_allow_fim_events: bool,
    #[serde(default)]
    pub audit_allow_process_events: bool,
    #[serde(default)]
    pub audit_allow_sockets: bool,
    #[serde(default)]
    pub audit_allow_user_events: bool,
    #[serde(default)]
    pub augeas_lenses: String,
    #[serde(default)]
    pub aws_access_key_id: String,
    #[serde(default)]
    pub aws_debug: bool,
    #[serde(default)]
    pub aws_disable_imdsv1_fallback: bool,
    #[serde(default)]
    pub aws_enable_proxy: bool,
    #[serde(default)]
    pub aws_enforce_fips: bool,
    #[serde(default)]
    pub aws_firehose_endpoint: String,
    #[serde(default)]
    pub aws_firehose_period: u64,
    #[serde(default)]
    pub aws_firehose_region: String,
    #[serde(default)]
    pub aws_firehose_stream: String,
    #[serde(default)]
    pub aws_imdsv2_request_attempts: u32,
    #[serde(default)]
    pub aws_imdsv2_request_interval: u32,
    #[serde(default)]
    pub aws_kinesis_disable_log_status: bool,
    #[serde(default)]
    pub aws_kinesis_endpoint: String,
    #[serde(default)]
    pub aws_kinesis_period: u64,
    #[serde(default)]
    pub aws_kinesis_random_partition_key: bool,
    #[serde(default)]
    pub aws_kinesis_region: String,
    #[serde(default)]
    pub aws_kinesis_stream: String,
    #[serde(default)]
    pub aws_profile_name: String,
    #[serde(default)]
    pub aws_proxy_host: String,
    #[serde(default)]
    pub aws_proxy_password: String,
    #[serde(default)]
    pub aws_proxy_port: u32,
    #[serde(default)]
    pub aws_proxy_scheme: String,
    #[serde(default)]
    pub aws_proxy_username: String,
    #[serde(default)]
    pub aws_region: String,
    #[serde(default)]
    pub aws_secret_access_key: String,
    #[serde(default)]
    pub aws_session_token: String,
    #[serde(default)]
    pub aws_sts_arn_role: String,
    #[serde(default)]
    pub aws_sts_region: String,
    #[serde(default)]
    pub aws_sts_session_name: String,
    #[serde(default)]
    pub aws_sts_timeout: u64,
    #[serde(default)]
    pub buffered_log_max: u64,
    #[serde(default)]
    pub carver_block_size: u32,
    #[serde(default)]
    pub carver_compression: bool,
    #[serde(default)]
    pub carver_continue_endpoint: String,
    #[serde(default)]
    pub carver_disable_function: bool,
    #[serde(default)]
    pub carver_expiry: u32,
    #[serde(default)]
    pub carver_start_endpoint: String,
    #[serde(default)]
    pub config_accelerated_refresh: u64,
    #[serde(default)]
    pub config_check: bool,
    #[serde(default)]
    pub config_dump: bool,
    #[serde(default)]
    pub config_enable_backup: bool,
    #[serde(default)]
    pub config_path: String,
    #[serde(default)]
    pub config_plugin: String,
    #[serde(default)]
    pub config_refresh: u64,
    #[serde(default)]
    pub config_tls_endpoint: String,
    #[serde(default)]
    pub config_tls_max_attempts: u64,
    #[serde(default)]
    pub daemonize: bool,
    #[serde(default)]
    pub database_dump: bool,
    #[serde(default)]
    pub database_path: String,
    #[serde(default)]
    pub decorations_top_level: bool,
    #[serde(default)]
    pub disable_audit: bool,
    #[serde(default)]
    pub disable_caching: bool,
    #[serde(default)]
    pub disable_carver: bool,
    #[serde(default)]
    pub disable_database: bool,
    #[serde(default)]
    pub disable_decorators: bool,
    #[serde(default)]
    pub disable_distributed: bool,
    #[serde(default)]
    pub disable_enrollment: bool,
    #[serde(default)]
    pub disable_events: bool,
    #[serde(default)]
    pub disable_extensions: bool,
    #[serde(default)]
    pub disable_hash_cache: bool,
    #[serde(default)]
    pub disable_logging: bool,
    #[serde(default)]
    pub disable_reenrollment: bool,
    #[serde(default)]
    pub disable_tables: String,
    #[serde(default)]
    pub disable_watchdog: bool,
    #[serde(default)]
    pub distributed_denylist_duration: u64,
    #[serde(default)]
    pub distributed_interval: u64,
    #[serde(default)]
    pub distributed_loginfo: bool,
    #[serde(default)]
    pub distributed_plugin: String,
    #[serde(default)]
    pub distributed_tls_max_attempts: u64,
    #[serde(default)]
    pub distributed_tls_read_endpoint: String,
    #[serde(default)]
    pub distributed_tls_write_endpoint: String,
    #[serde(default)]
    pub dns_resolver_refresh_interval: i32,
    #[serde(default)]
    pub docker_socket: String,
    #[serde(default)]
    pub enable_extensions_watchdog: bool,
    #[serde(default)]
    pub enable_file_events: bool,
    #[serde(default)]
    pub enable_foreign: bool,
    #[serde(default)]
    pub enable_numeric_monitoring: bool,
    #[serde(default)]
    pub enable_tables: String,
    #[serde(default)]
    pub enable_watchdog_debug: bool,
    #[serde(default)]
    pub enroll_always: bool,
    #[serde(default)]
    pub enroll_secret_env: String,
    #[serde(default)]
    pub enroll_secret_path: String,
    #[serde(default)]
    pub enroll_tls_endpoint: String,
    #[serde(default)]
    pub ephemeral: bool,
    #[serde(default)]
    pub es_fim_enable_open_events: bool,
    #[serde(default)]
    pub events_expiry: u64,
    #[serde(default)]
    pub events_max: u64,
    #[serde(default)]
    pub events_optimize: bool,
    #[serde(default)]
    pub experiment_list: String,
    #[serde(default)]
    pub extensions_autoload: String,
    #[serde(default)]
    pub extensions_default_index: bool,
    #[serde(default)]
    pub extensions_interval: String,
    #[serde(default)]
    pub extensions_require: String,
    #[serde(default)]
    pub extensions_socket: String,
    #[serde(default)]
    pub extensions_timeout: String,
    #[serde(default)]
    pub force: bool,
    #[serde(default)]
    pub hash_cache_max: u32,
    #[serde(default)]
    pub host_identifier: String,
    #[serde(default)]
    pub ignore_table_exceptions: bool,
    #[serde(default)]
    pub install: bool,
    #[serde(default)]
    pub keychain_access_cache: bool,
    #[serde(default)]
    pub keychain_access_interval: u32,
    #[serde(default)]
    pub logger_event_type: bool,
    #[serde(default)]
    pub logger_kafka_acks: String,
    #[serde(default)]
    pub logger_kafka_brokers: String,
    #[serde(default)]
    pub logger_kafka_compression: String,
    #[serde(default)]
    pub logger_kafka_topic: String,
    #[serde(default)]
    pub logger_min_status: i32,
    #[serde(default)]
    pub logger_min_stderr: i32,
    #[serde(default)]
    pub logger_mode: String,
    #[serde(default)]
    pub logger_numerics: bool,
    #[serde(default)]
    pub logger_path: String,
    #[serde(default)]
    pub logger_plugin: String,
    #[serde(default)]
    pub logger_rotate: bool,
    #[serde(default)]
    pub logger_rotate_max_files: u64,
    #[serde(default)]
    pub logger_rotate_size: u64,
    #[serde(default)]
    pub logger_snapshot_event_type: bool,
    #[serde(default)]
    pub logger_stderr: bool,
    #[serde(default)]
    pub logger_syslog_facility: i32,
    #[serde(default)]
    pub logger_syslog_prepend_cee: bool,
    #[serde(default)]
    pub logger_tls_backoff_max: u64,
    #[serde(default)]
    pub logger_tls_compress: bool,
    #[serde(default)]
    pub logger_tls_endpoint: String,
    #[serde(default)]
    pub logger_tls_max_lines: u64,
    #[serde(default)]
    pub logger_tls_max_linesize: u64,
    #[serde(default)]
    pub logger_tls_period: u64,
    #[serde(default)]
    pub logtostderr: bool,
    #[serde(default)]
    pub nullvalue: String,
    #[serde(default)]
    pub numeric_monitoring_filesystem_path: String,
    #[serde(default)]
    pub numeric_monitoring_plugins: String,
    #[serde(default)]
    pub numeric_monitoring_pre_aggregation_time: u64,
    #[serde(default)]
    pub pack_delimiter: String,
    #[serde(default)]
    pub pack_refresh_interval: u64,
    #[serde(default)]
    pub pidfile: String,
    #[serde(default)]
    pub proxy_hostname: String,
    #[serde(default)]
    pub read_max: u64,
    #[serde(default)]
    pub schedule_default_interval: u64,
    #[serde(default)]
    pub schedule_epoch: u64,
    #[serde(default)]
    pub schedule_lognames: bool,
    #[serde(default)]
    pub schedule_max_drift: u64,
    #[serde(default)]
    pub schedule_reload: u64,
    #[serde(default)]
    pub schedule_splay_percent: u64,
    #[serde(default)]
    pub schedule_timeout: u64,
    #[serde(default)]
    pub specified_identifier: String,
    #[serde(default)]
    pub stderrthreshold: i32,
    #[serde(default)]
    pub table_delay: u64,
    #[serde(default)]
    pub thrift_string_size_limit: i32,
    #[serde(default)]
    pub thrift_timeout: u32,
    #[serde(default)]
    pub thrift_verbose: bool,
    #[serde(default)]
    pub tls_accept_gzip: bool,
    #[serde(default)]
    pub tls_client_cert: String,
    #[serde(default)]
    pub tls_client_key: String,
    #[serde(default)]
    pub tls_disable_status_log: bool,
    #[serde(default)]
    pub tls_enroll_max_attempts: u64,
    #[serde(default)]
    pub tls_enroll_max_interval: u64,
    #[serde(default)]
    pub tls_hostname: String,
    #[serde(default)]
    pub tls_server_certs: String,
    #[serde(default)]
    pub tls_session_reuse: bool,
    #[serde(default)]
    pub tls_session_timeout: u32,
    #[serde(default)]
    pub uninstall: bool,
    #[serde(default)]
    pub verbose: bool,
    #[serde(default)]
    pub watchdog_delay: u64,
    #[serde(default)]
    pub watchdog_forced_shutdown_delay: u64,
    #[serde(default)]
    pub watchdog_latency_limit: u64,
    #[serde(default)]
    pub watchdog_level: i32,
    #[serde(default)]
    pub watchdog_memory_limit: u64,
    #[serde(default)]
    pub watchdog_utilization_limit: u64,
    #[serde(default)]
    pub yara_delay: u32,
    #[serde(default)]
    pub yara_sigurl_authenticate: bool,

    // OS-specific flags (flattened from embedded structs in Go)
    #[serde(flatten)]
    pub linux_flags: OsqueryCommandLineFlagsLinux,
    #[serde(flatten)]
    pub windows_flags: OsqueryCommandLineFlagsWindows,
    #[serde(flatten)]
    pub macos_flags: OsqueryCommandLineFlagsMacOS,
    #[serde(flatten)]
    pub hidden_flags: OsqueryCommandLineFlagsHidden,
}

// ─── OS-Specific Command-Line Flags ──────────────────────────────────────────

/// Linux-specific osquery command-line flags.
/// Matches Go's `OsqueryCommandLineFlagsLinux`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OsqueryCommandLineFlagsLinux {
    #[serde(default)]
    pub audit_allow_accept_socket_events: bool,
    #[serde(default)]
    pub audit_allow_apparmor_events: bool,
    #[serde(default)]
    pub audit_allow_failed_socket_events: bool,
    #[serde(default)]
    pub audit_allow_fork_process_events: bool,
    #[serde(default)]
    pub audit_allow_kill_process_events: bool,
    #[serde(default)]
    pub audit_allow_null_accept_socket_events: bool,
    #[serde(default)]
    pub audit_allow_seccomp_events: bool,
    #[serde(default)]
    pub audit_allow_selinux_events: bool,
    #[serde(default)]
    pub audit_backlog_limit: i32,
    #[serde(default)]
    pub audit_backlog_wait_time: i32,
    #[serde(default)]
    pub audit_force_reconfigure: bool,
    #[serde(default)]
    pub audit_force_unconfigure: bool,
    #[serde(default)]
    pub audit_persist: bool,
    #[serde(default)]
    pub bpf_buffer_storage_size: u64,
    #[serde(default)]
    pub bpf_perf_event_array_exp: u64,
    #[serde(default)]
    pub disable_memory: bool,
    #[serde(default)]
    pub enable_bpf_events: bool,
    #[serde(default)]
    pub enable_syslog: bool,
    #[serde(default)]
    pub experiments_linuxevents_circular_buffer_size: u32,
    #[serde(default)]
    pub experiments_linuxevents_perf_output_size: u32,
    #[serde(default)]
    pub hardware_disabled_types: String,
    #[serde(default)]
    pub keep_container_worker_open: bool,
    #[serde(default)]
    pub lxd_socket: String,
    #[serde(default)]
    pub malloc_trim_threshold: u64,
    #[serde(default)]
    pub syslog_events_expiry: u64,
    #[serde(default)]
    pub syslog_events_max: u64,
    #[serde(default)]
    pub syslog_pipe_path: String,
    #[serde(default)]
    pub syslog_rate_limit: u64,
}

/// Windows-specific osquery command-line flags.
/// Matches Go's `OsqueryCommandLineFlagsWindows`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OsqueryCommandLineFlagsWindows {
    #[serde(default)]
    pub users_service_delay: u64,
    #[serde(default)]
    pub users_service_interval: u64,
    #[serde(default)]
    pub groups_service_delay: u64,
    #[serde(default)]
    pub groups_service_interval: u64,
    #[serde(default)]
    pub enable_ntfs_event_publisher: bool,
    #[serde(default)]
    pub enable_powershell_events_subscriber: bool,
    #[serde(default)]
    pub enable_process_etw_events: bool,
    #[serde(default)]
    pub enable_windows_events_publisher: bool,
    #[serde(default)]
    pub enable_windows_events_subscriber: bool,
    #[serde(default)]
    pub etw_kernel_trace_buffer_size: u32,
    #[serde(default)]
    pub etw_kernel_trace_flush_timer: u32,
    #[serde(default)]
    pub etw_kernel_trace_maximum_buffers: u32,
    #[serde(default)]
    pub etw_kernel_trace_minimum_buffers: u32,
    #[serde(default)]
    pub etw_userspace_trace_buffer_size: u32,
    #[serde(default)]
    pub etw_userspace_trace_flush_timer: u32,
    #[serde(default)]
    pub etw_userspace_trace_maximum_buffers: u32,
    #[serde(default)]
    pub etw_userspace_trace_minimum_buffers: u32,
    #[serde(default)]
    pub ntfs_event_publisher_debug: bool,
    #[serde(default)]
    pub windows_event_channels: String,
    #[serde(default)]
    pub usn_journal_reader_debug: bool,
    #[serde(default)]
    pub enable_dns_lookup_events: bool,
}

/// macOS-specific osquery command-line flags.
/// Matches Go's `OsqueryCommandLineFlagsMacOS`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OsqueryCommandLineFlagsMacOS {
    #[serde(default)]
    pub disable_endpointsecurity: bool,
    #[serde(default)]
    pub disable_endpointsecurity_fim: bool,
    #[serde(default)]
    pub enable_keyboard_events: bool,
    #[serde(default)]
    pub enable_mouse_events: bool,
    #[serde(default)]
    pub es_fim_mute_path_literal: String,
    #[serde(default)]
    pub es_fim_mute_path_prefix: String,
}

/// Hidden (non-help-visible) osquery command-line flags.
/// Matches Go's `OsqueryCommandLineFlagsHidden`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OsqueryCommandLineFlagsHidden {
    #[serde(default)]
    pub alsologtostderr: bool,
    #[serde(default)]
    pub events_streaming_plugin: String,
    #[serde(default)]
    pub ignore_registry_exceptions: bool,
    #[serde(default)]
    pub logbufsecs: i32,
    #[serde(default)]
    pub log_dir: String,
    #[serde(default)]
    pub max_log_size: i32,
    #[serde(default)]
    pub minloglevel: i32,
    #[serde(default)]
    pub stop_logging_if_full_disk: bool,
    #[serde(default)]
    pub allow_unsafe: bool,
    #[serde(default)]
    pub tls_dump: bool,
    #[serde(default)]
    pub audit_debug: bool,
    #[serde(default)]
    pub audit_fim_debug: bool,
    #[serde(default)]
    pub audit_show_partial_fim_events: bool,
    #[serde(default)]
    pub audit_show_untracked_res_warnings: bool,
    #[serde(default)]
    pub audit_fim_show_accesses: bool,
    #[serde(default)]
    pub vmodule: String,
}
