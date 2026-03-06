//! AppConfig types matching Go's `server/fleet/app.go`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::mdm::MDMProfileSpec;

// ─── SMTP auth method/type name constants ───────────────────────────────────

pub const AUTH_METHOD_NAME_CRAM_MD5: &str = "authmethod_cram_md5";
pub const AUTH_METHOD_NAME_LOGIN: &str = "authmethod_login";
pub const AUTH_METHOD_NAME_PLAIN: &str = "authmethod_plain";
pub const AUTH_TYPE_NAME_USERNAME_PASSWORD: &str = "authtype_username_password";
pub const AUTH_TYPE_NAME_NONE: &str = "authtype_none";

pub const APP_CONFIG_KIND: &str = "config";
pub const MASKED_PASSWORD: &str = "********";

pub const DEFAULT_ORG_INFO_CONTACT_URL: &str = "https://fleetdm.com/company/contact";
pub const DEFAULT_TRANSPARENCY_URL: &str = "https://fleetdm.com/transparency";
pub const SECUREFRAME_TRANSPARENCY_URL: &str = "https://fleetdm.com/better?utm_content=secureframe";
pub const DEFAULT_MAX_QUERY_REPORT_ROWS: i32 = 1000;

// ─── License constants ──────────────────────────────────────────────────────

pub const TIER_PREMIUM: &str = "premium";
pub const TIER_FREE: &str = "free";
pub const TIER_TRIAL: &str = "trial";

pub const HEADER_LICENSE_KEY: &str = "X-Fleet-License";
pub const HEADER_LICENSE_VALUE_EXPIRED: &str = "Expired";

// ─── OrgInfo ────────────────────────────────────────────────────────────────

/// OrgInfo holds organization information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrgInfo {
    #[serde(default)]
    pub org_name: String,
    #[serde(default)]
    pub org_logo_url: String,
    #[serde(default)]
    pub org_logo_url_light_background: String,
    #[serde(default)]
    pub contact_url: String,
}

// ─── ServerSettings ─────────────────────────────────────────────────────────

/// ServerSettings holds server-related settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerSettings {
    #[serde(default)]
    pub server_url: String,
    #[serde(default)]
    pub live_query_disabled: bool,
    #[serde(default)]
    pub enable_analytics: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug_host_ids: Option<Vec<u32>>,
    #[serde(default)]
    pub deferred_save_host: bool,
    #[serde(default)]
    pub query_reports_disabled: bool,
    #[serde(default)]
    pub scripts_disabled: bool,
    #[serde(default)]
    pub ai_features_disabled: bool,
    #[serde(default)]
    pub query_report_cap: i32,
}

// ─── SMTPSettings ───────────────────────────────────────────────────────────

/// SMTPSettings holds SMTP configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SMTPSettings {
    #[serde(rename = "enable_smtp", default)]
    pub smtp_enabled: bool,
    #[serde(default)]
    pub configured: bool,
    #[serde(default)]
    pub sender_address: String,
    #[serde(default)]
    pub server: String,
    #[serde(default)]
    pub port: u32,
    #[serde(default)]
    pub authentication_type: String,
    #[serde(default)]
    pub user_name: String,
    #[serde(default)]
    pub password: String,
    #[serde(rename = "enable_ssl_tls", default)]
    pub enable_tls: bool,
    #[serde(default)]
    pub authentication_method: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub verify_ssl_certs: bool,
    #[serde(default)]
    pub enable_start_tls: bool,
}

// ─── HostExpirySettings ─────────────────────────────────────────────────────

/// HostExpirySettings controls host expiry behavior.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostExpirySettings {
    #[serde(default)]
    pub host_expiry_enabled: bool,
    #[serde(default)]
    pub host_expiry_window: i32,
}

// ─── ActivityExpirySettings ─────────────────────────────────────────────────

/// ActivityExpirySettings controls activity log expiry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivityExpirySettings {
    #[serde(default)]
    pub activity_expiry_enabled: bool,
    #[serde(default)]
    pub activity_expiry_window: i32,
}

// ─── SSO settings ───────────────────────────────────────────────────────────

/// SSOProviderSettings holds identity provider configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SSOProviderSettings {
    #[serde(default)]
    pub entity_id: String,
    #[serde(default)]
    pub issuer_uri: String,
    #[serde(default)]
    pub metadata: String,
    #[serde(default)]
    pub metadata_url: String,
    #[serde(default)]
    pub idp_name: String,
}

/// SSOSettings holds single sign-on integration settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SSOSettings {
    #[serde(flatten)]
    pub provider: SSOProviderSettings,
    #[serde(default)]
    pub idp_image_url: String,
    #[serde(default)]
    pub enable_sso: bool,
    #[serde(default)]
    pub enable_sso_idp_login: bool,
    #[serde(default)]
    pub enable_jit_provisioning: bool,
    #[serde(default)]
    pub enable_jit_role_sync: bool,
    #[serde(default)]
    pub sso_server_url: String,
}

// ─── ConditionalAccessSettings ──────────────────────────────────────────────

/// ConditionalAccessSettings holds the global conditional access settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConditionalAccessSettings {
    #[serde(default)]
    pub microsoft_entra_tenant_id: String,
    #[serde(default)]
    pub microsoft_entra_connection_configured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub okta_idp_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub okta_assertion_consumer_service_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub okta_audience_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub okta_certificate: Option<String>,
    #[serde(default)]
    pub bypass_disabled: bool,
}

// ─── VulnerabilitySettings ──────────────────────────────────────────────────

/// VulnerabilitySettings configures vulnerability scanning behavior.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VulnerabilitySettings {
    #[serde(default)]
    pub databases_path: String,
}

// ─── FleetDesktopSettings ───────────────────────────────────────────────────

/// FleetDesktopSettings holds Fleet Desktop settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FleetDesktopSettings {
    #[serde(default)]
    pub transparency_url: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub alternative_browser_host: String,
}

// ─── Features ───────────────────────────────────────────────────────────────

/// Features allows enabling or disabling features.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Features {
    #[serde(default)]
    pub enable_host_users: bool,
    #[serde(default)]
    pub enable_software_inventory: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_queries: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail_query_overrides: Option<HashMap<String, Option<String>>>,
}

// ─── Webhook settings ───────────────────────────────────────────────────────

/// WebhookSettings holds global webhook configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebhookSettings {
    #[serde(default)]
    pub activities_webhook: ActivitiesWebhookSettings,
    #[serde(default)]
    pub host_status_webhook: HostStatusWebhookSettings,
    #[serde(default)]
    pub failing_policies_webhook: FailingPoliciesWebhookSettings,
    #[serde(default)]
    pub vulnerabilities_webhook: VulnerabilitiesWebhookSettings,
    /// Interval is a duration string (e.g. "24h0m0s").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<serde_json::Value>,
}

/// ActivitiesWebhookSettings configures activity webhooks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivitiesWebhookSettings {
    #[serde(default)]
    pub enable_activities_webhook: bool,
    #[serde(default)]
    pub destination_url: String,
}

/// HostStatusWebhookSettings configures host status webhooks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostStatusWebhookSettings {
    #[serde(default)]
    pub enable_host_status_webhook: bool,
    #[serde(default)]
    pub destination_url: String,
    #[serde(default)]
    pub host_percentage: f64,
    #[serde(default)]
    pub days_count: i32,
}

/// FailingPoliciesWebhookSettings configures failing policies webhooks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FailingPoliciesWebhookSettings {
    #[serde(default)]
    pub enable_failing_policies_webhook: bool,
    #[serde(default)]
    pub destination_url: String,
    #[serde(default)]
    pub policy_ids: Vec<u32>,
    #[serde(default)]
    pub host_batch_size: i32,
}

/// VulnerabilitiesWebhookSettings configures vulnerability webhooks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VulnerabilitiesWebhookSettings {
    #[serde(default)]
    pub enable_vulnerabilities_webhook: bool,
    #[serde(default)]
    pub destination_url: String,
    #[serde(default)]
    pub host_batch_size: i32,
}

// ─── Integration types ──────────────────────────────────────────────────────

/// JiraIntegration holds global Jira integration configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JiraIntegration {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub api_token: String,
    #[serde(default)]
    pub project_key: String,
    #[serde(default)]
    pub enable_failing_policies: bool,
    #[serde(default)]
    pub enable_software_vulnerabilities: bool,
}

/// ZendeskIntegration holds global Zendesk integration configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZendeskIntegration {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub api_token: String,
    #[serde(default)]
    pub group_id: i64,
    #[serde(default)]
    pub enable_failing_policies: bool,
    #[serde(default)]
    pub enable_software_vulnerabilities: bool,
}

/// GoogleCalendarApiKey wraps the Google Calendar API key JSON.
/// In Go this is a custom type with masked serialization support.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GoogleCalendarApiKey {
    #[default]
    Empty,
    /// Masked value (serializes to "********").
    Masked(String),
    /// Actual key-value pairs from the service account JSON.
    Values(HashMap<String, String>),
}

/// GoogleCalendarIntegration holds global Google Calendar configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoogleCalendarIntegration {
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub api_key_json: GoogleCalendarApiKey,
}

/// Integrations holds global integration configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Integrations {
    #[serde(default)]
    pub jira: Vec<JiraIntegration>,
    #[serde(default)]
    pub zendesk: Vec<ZendeskIntegration>,
    #[serde(default)]
    pub google_calendar: Vec<GoogleCalendarIntegration>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conditional_access_enabled: Option<bool>,
}

// ─── MDM-related AppConfig types ────────────────────────────────────────────

/// MDMAppleABMAssignmentInfo represents ABM token to team association.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMAppleABMAssignmentInfo {
    #[serde(default)]
    pub organization_name: String,
    #[serde(default)]
    pub macos_team: String,
    #[serde(default)]
    pub ios_team: String,
    #[serde(default)]
    pub ipados_team: String,
}

/// MDMAppleVolumePurchasingProgramInfo represents a VPP token to team association.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMAppleVolumePurchasingProgramInfo {
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub teams: Vec<String>,
}

/// AppleOSUpdateSettings contains the settings for OS updates on Apple devices.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppleOSUpdateSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_new_hosts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
}

/// WindowsUpdates contains the settings for Windows OS updates.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowsUpdates {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline_days: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grace_period_days: Option<i32>,
}

/// MacOSSettings contains settings specific to macOS.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MacOSSettings {
    #[serde(default)]
    pub custom_settings: Vec<MDMProfileSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_disk_encryption: Option<bool>,
}

/// MacOSSetupSoftware represents a VPP app or a software package to install during setup.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MacOSSetupSoftware {
    #[serde(default)]
    pub app_store_id: String,
    #[serde(default)]
    pub package_path: String,
}

/// MacOSSetup contains settings related to the setup of DEP enrolled devices.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MacOSSetup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bootstrap_package: Option<String>,
    #[serde(default)]
    pub enable_end_user_authentication: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_end_user_info: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos_setup_assistant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_release_device_manually: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software: Option<Vec<MacOSSetupSoftware>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual_agent_install: Option<bool>,
    #[serde(default, rename = "require_all_software_macos")]
    pub require_all_software: bool,
}

/// MacOSMigrationMode defines the possible modes for MDM migration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum MacOSMigrationMode {
    #[default]
    #[serde(rename = "forced")]
    Forced,
    #[serde(rename = "voluntary")]
    Voluntary,
}

/// MacOSMigration contains settings related to the MDM migration work flow.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MacOSMigration {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub mode: MacOSMigrationMode,
    #[serde(default)]
    pub webhook_url: String,
}

/// MDMEndUserAuthentication contains settings related to end user authentication for MDM.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMEndUserAuthentication {
    #[serde(flatten)]
    pub sso_provider_settings: SSOProviderSettings,
}

/// WindowsSettings contains settings specific to Windows MDM.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowsSettings {
    #[serde(default)]
    pub custom_settings: Vec<MDMProfileSpec>,
}

/// CertificateTemplateSpec defines a certificate template to be deployed to devices.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CertificateTemplateSpec {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub certificate_authority_name: String,
    #[serde(default)]
    pub subject_name: String,
}

/// AndroidSettings contains settings specific to Android MDM.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AndroidSettings {
    #[serde(default)]
    pub custom_settings: Vec<MDMProfileSpec>,
    #[serde(default)]
    pub certificates: Vec<CertificateTemplateSpec>,
}

/// DiskEncryptionConfig holds the disk encryption configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiskEncryptionConfig {
    pub enabled: bool,
    pub bitlocker_pin_required: bool,
}

/// MDMConfig is part of AppConfig and defines the MDM settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMConfig {
    #[serde(default)]
    pub apple_server_url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub apple_bm_default_team: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub apple_business_manager: Option<Vec<MDMAppleABMAssignmentInfo>>,

    #[serde(default)]
    pub apple_bm_enabled_and_configured: bool,

    #[serde(default)]
    pub apple_bm_terms_expired: bool,

    #[serde(default)]
    pub enabled_and_configured: bool,

    #[serde(default)]
    pub macos_updates: AppleOSUpdateSettings,
    #[serde(default)]
    pub ios_updates: AppleOSUpdateSettings,
    #[serde(default)]
    pub ipados_updates: AppleOSUpdateSettings,
    #[serde(default)]
    pub windows_updates: WindowsUpdates,

    #[serde(default)]
    pub macos_settings: MacOSSettings,
    #[serde(default)]
    pub macos_setup: MacOSSetup,
    #[serde(default)]
    pub macos_migration: MacOSMigration,
    #[serde(default)]
    pub windows_migration_enabled: bool,
    #[serde(default)]
    pub enable_turn_on_windows_mdm_manually: bool,
    #[serde(default)]
    pub end_user_authentication: MDMEndUserAuthentication,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows_entra_tenant_ids: Option<Vec<String>>,

    #[serde(default)]
    pub windows_enabled_and_configured: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_disk_encryption: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_recovery_lock_password: Option<bool>,

    #[serde(rename = "windows_require_bitlocker_pin", skip_serializing_if = "Option::is_none")]
    pub require_bitlocker_pin: Option<bool>,

    #[serde(default)]
    pub windows_settings: WindowsSettings,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_purchasing_program: Option<Vec<MDMAppleVolumePurchasingProgramInfo>>,

    #[serde(default)]
    pub android_enabled_and_configured: bool,
    #[serde(default)]
    pub android_settings: AndroidSettings,
}

// ─── UIGitOpsModeConfig ─────────────────────────────────────────────────────

/// UIGitOpsModeConfig holds GitOps UI mode configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UIGitOpsModeConfig {
    #[serde(default)]
    pub gitops_mode_enabled: bool,
    #[serde(default)]
    pub repository_url: String,
}

// ─── YaraRule ───────────────────────────────────────────────────────────────

/// YaraRuleSpec defines a YARA rule file path spec.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct YaraRuleSpec {
    #[serde(default)]
    pub path: String,
}

/// YaraRule holds a YARA rule name and contents.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct YaraRule {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub contents: String,
}

// ─── SecretVariable ─────────────────────────────────────────────────────────

/// SecretVariable represents a custom variable with encrypted value.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecretVariable {
    pub id: u32,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub value: String,
    #[serde(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ─── AppConfig ──────────────────────────────────────────────────────────────

/// AppConfig is the top-level application configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub org_info: OrgInfo,
    #[serde(default)]
    pub server_settings: ServerSettings,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smtp_settings: Option<SMTPSettings>,
    #[serde(default)]
    pub host_expiry_settings: HostExpirySettings,
    #[serde(default)]
    pub activity_expiry_settings: ActivityExpirySettings,
    #[serde(default)]
    pub features: Features,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_settings: Option<Features>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_options: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smtp_test: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sso_settings: Option<SSOSettings>,
    #[serde(default)]
    pub fleet_desktop: FleetDesktopSettings,
    #[serde(default)]
    pub vulnerability_settings: VulnerabilitySettings,
    #[serde(default)]
    pub webhook_settings: WebhookSettings,
    #[serde(default)]
    pub integrations: Integrations,
    #[serde(default)]
    pub mdm: MDMConfig,
    #[serde(default, rename = "gitops")]
    pub ui_gitops_mode: UIGitOpsModeConfig,
    #[serde(default)]
    pub scripts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yara_rules: Option<Vec<YaraRule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditional_access: Option<ConditionalAccessSettings>,
}

// ─── LicenseInfo ────────────────────────────────────────────────────────────

/// Partnerships contains specialized configuration options for Fleet partners.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Partnerships {
    #[serde(default, skip_serializing_if = "is_false")]
    pub enable_primo: bool,
}

fn is_false(v: &bool) -> bool {
    !*v
}

/// LicenseInfo holds information about the Fleet license.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LicenseInfo {
    #[serde(default)]
    pub tier: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_disable_telemetry: Option<bool>,
    #[serde(default)]
    pub managed_cloud: bool,
}

impl LicenseInfo {
    pub fn is_premium(&self) -> bool {
        self.tier == TIER_PREMIUM || self.tier == "basic" || self.tier == TIER_TRIAL
    }
}

// ─── Logging ────────────────────────────────────────────────────────────────

/// LoggingPlugin holds the plugin name and configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoggingPlugin {
    #[serde(default)]
    pub plugin: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
}

/// Logging holds the logging configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Logging {
    #[serde(default)]
    pub debug: bool,
    #[serde(default)]
    pub json: bool,
    #[serde(default)]
    pub result: LoggingPlugin,
    #[serde(default)]
    pub status: LoggingPlugin,
    #[serde(default)]
    pub audit: LoggingPlugin,
}

/// EmailConfig holds email backend configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmailConfig {
    #[serde(default)]
    pub backend: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
}

/// SESConfig holds AWS SES email configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SESConfig {
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub source_arn: String,
}

/// UpdateIntervalConfig holds osquery update interval configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateIntervalConfig {
    /// Duration in nanoseconds for osquery detail queries.
    #[serde(default)]
    pub osquery_detail: u64,
    /// Duration in nanoseconds for osquery policy queries.
    #[serde(default)]
    pub osquery_policy: u64,
}

/// VulnerabilitiesConfig holds the full vulnerabilities runtime configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VulnerabilitiesConfig {
    #[serde(default)]
    pub databases_path: String,
    #[serde(default)]
    pub periodicity: u64,
    #[serde(default)]
    pub cpe_database_url: String,
    #[serde(default)]
    pub cpe_translations_url: String,
    #[serde(default)]
    pub cve_feed_prefix_url: String,
    #[serde(default)]
    pub current_instance_checks: String,
    #[serde(default)]
    pub disable_data_sync: bool,
    #[serde(default)]
    pub recent_vulnerability_max_age: u64,
    #[serde(default)]
    pub disable_win_os_vulnerabilities: bool,
}

/// FirehoseConfig holds AWS Firehose logging configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FirehoseConfig {
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub status_stream: String,
    #[serde(default)]
    pub result_stream: String,
    #[serde(default)]
    pub audit_stream: String,
}

/// KinesisConfig holds AWS Kinesis logging configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KinesisConfig {
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub status_stream: String,
    #[serde(default)]
    pub result_stream: String,
    #[serde(default)]
    pub audit_stream: String,
}

/// LambdaConfig holds AWS Lambda logging configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LambdaConfig {
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub status_function: String,
    #[serde(default)]
    pub result_function: String,
    #[serde(default)]
    pub audit_function: String,
}

/// KafkaRESTConfig holds Kafka REST proxy configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KafkaRESTConfig {
    #[serde(default)]
    pub status_topic: String,
    #[serde(default)]
    pub result_topic: String,
    #[serde(default)]
    pub audit_topic: String,
    #[serde(default)]
    pub proxyhost: String,
}

/// NatsConfig holds NATS logging configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NatsConfig {
    #[serde(default)]
    pub server: String,
    #[serde(default)]
    pub status_subject: String,
    #[serde(default)]
    pub result_subject: String,
    #[serde(default)]
    pub audit_subject: String,
}

// ─── EnrichedAppConfig ──────────────────────────────────────────────────────

/// EnrichedAppConfig extends AppConfig with additional runtime fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnrichedAppConfig {
    #[serde(flatten)]
    pub app_config: AppConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_interval: Option<UpdateIntervalConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vulnerabilities: Option<VulnerabilitiesConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<LicenseInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<Logging>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<EmailConfig>,
}

// ─── DeviceGlobalConfig ─────────────────────────────────────────────────────

/// DeviceGlobalConfig is a subset of AppConfig used by device endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceGlobalConfig {
    #[serde(default)]
    pub mdm: DeviceGlobalMDMConfig,
    #[serde(default)]
    pub features: DeviceFeatures,
}

/// DeviceGlobalMDMConfig is a subset of MDMConfig used by device endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceGlobalMDMConfig {
    #[serde(default)]
    pub enabled_and_configured: bool,
    #[serde(default, rename = "require_all_software_macos")]
    pub require_all_software: bool,
}

/// DeviceFeatures is a subset of Features used by device endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceFeatures {
    #[serde(default)]
    pub enable_software_inventory: bool,
    #[serde(default)]
    pub enable_conditional_access: bool,
    #[serde(default)]
    pub enable_conditional_access_bypass: bool,
}

// ─── ApplySpecOptions ───────────────────────────────────────────────────────

/// ApplySpecOptions controls how specs are applied.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApplySpecOptions {
    #[serde(default)]
    pub force: bool,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub team_for_policies: String,
    #[serde(default)]
    pub no_cache: bool,
    #[serde(default)]
    pub overwrite: bool,
}

/// ApplyTeamSpecOptions extends ApplySpecOptions for team spec application.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApplyTeamSpecOptions {
    #[serde(flatten)]
    pub apply_spec_options: ApplySpecOptions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run_assumptions: Option<TeamSpecsDryRunAssumptions>,
}

/// TeamSpecsDryRunAssumptions holds assumptions for team spec dry-run mode.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamSpecsDryRunAssumptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows_enabled_and_configured: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub android_enabled_and_configured: Option<bool>,
}

/// ApplyClientSpecOptions extends ApplySpecOptions with client-side configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApplyClientSpecOptions {
    #[serde(flatten)]
    pub apply_spec_options: ApplySpecOptions,
    #[serde(default)]
    pub expand_env_config_profiles: bool,
}

// ─── ListOptions ────────────────────────────────────────────────────────────

/// OrderDirection specifies the direction of ordering.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderDirection {
    #[default]
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

/// ListOptions defines options related to paging and ordering.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ListOptions {
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub per_page: u32,
    #[serde(default)]
    pub order_key: String,
    #[serde(default)]
    pub order_direction: OrderDirection,
    #[serde(default)]
    pub match_query: String,
    #[serde(default)]
    pub after: String,
    #[serde(default)]
    pub include_metadata: bool,
}

/// ListQueryOptions extends ListOptions with query-specific filtering.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListQueryOptions {
    #[serde(flatten)]
    pub list_options: ListOptions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_scheduled: Option<bool>,
    #[serde(default)]
    pub merge_inherited: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
}
