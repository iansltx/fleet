//! AppConfig types matching Go's `server/fleet/app.go`.

use serde::{Deserialize, Serialize};

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

/// ServerSettings holds server-related settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerSettings {
    #[serde(default)]
    pub server_url: String,
    #[serde(default)]
    pub live_query_disabled: bool,
    #[serde(default)]
    pub enable_analytics: bool,
    #[serde(default)]
    pub deferred_save_host: bool,
    #[serde(default)]
    pub query_reports_disabled: bool,
    #[serde(default)]
    pub scripts_disabled: bool,
    #[serde(default)]
    pub ai_features_disabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_host_ids: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_report_cap: Option<i32>,
}

/// SMTPSettings holds SMTP configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SMTPSettings {
    #[serde(rename = "enable_smtp")]
    pub smtp_enabled: bool,
    pub configured: bool,
    pub sender_address: String,
    pub server: String,
    pub port: u32,
    pub authentication_type: String,
    pub user_name: String,
    pub password: String,
    #[serde(rename = "enable_ssl_tls")]
    pub enable_tls: bool,
    pub authentication_method: String,
    pub domain: String,
    pub verify_ssl_certs: bool,
    pub enable_start_tls: bool,
}

/// ActivityExpirySettings controls activity log expiry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivityExpirySettings {
    #[serde(default)]
    pub activity_expiry_enabled: bool,
    #[serde(default)]
    pub activity_expiry_window: u32,
}

/// SSOProviderSettings holds identity provider configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SSOProviderSettings {
    pub entity_id: String,
    pub issuer_uri: String,
    pub metadata: String,
    pub metadata_url: String,
    pub idp_name: String,
}

/// SSOSettings holds single sign-on integration settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SSOSettings {
    #[serde(flatten)]
    pub provider: SSOProviderSettings,
    pub idp_image_url: String,
    pub enable_sso: bool,
    pub enable_sso_idp_login: bool,
    pub enable_jit_provisioning: bool,
    pub enable_jit_role_sync: bool,
    pub sso_server_url: String,
}

/// VulnerabilitySettings configures vulnerability scanning behavior.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VulnerabilitySettings {
    pub databases_path: String,
}

/// FleetDesktopSettings holds Fleet Desktop settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FleetDesktopSettings {
    #[serde(default)]
    pub transparency_url: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub alternative_browser_host: String,
}

/// WebhookSettings holds global webhook configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebhookSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activities_webhook: Option<ActivitiesWebhookSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_status_webhook: Option<HostStatusWebhookSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failing_policies_webhook: Option<FailingPoliciesWebhookSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vulnerabilities_webhook: Option<VulnerabilitiesWebhookSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<serde_json::Value>,
}

/// Integrations holds global integration configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Integrations {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub jira: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub zendesk: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_calendar: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ndes_scep_proxy: Option<serde_json::Value>,
}

/// MDMConfig is part of AppConfig and defines the MDM settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMConfig {
    #[serde(default)]
    pub apple_server_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apple_bm_default_team: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apple_business_manager: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub apple_bm_enabled_and_configured: bool,
    #[serde(default)]
    pub enabled_and_configured: bool,
    #[serde(default)]
    pub windows_enabled_and_configured: bool,
    #[serde(default)]
    pub android_enabled_and_configured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos_updates: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows_updates: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos_settings: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos_migration: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_user_authentication: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_purchasing_program: Option<Vec<serde_json::Value>>,
}

/// ConditionalAccessSettings holds the global conditional access settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConditionalAccessSettings {
    pub microsoft_entra_tenant_id: String,
    pub microsoft_entra_connection_configured: bool,
    #[serde(default)]
    pub bypass_disabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub okta_idp_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub okta_assertion_consumer_service_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub okta_audience_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub okta_certificate: Option<String>,
}

/// UIGitOpsModeConfig holds GitOps UI mode configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UIGitOpsModeConfig {
    // Fields can be added as needed.
}

/// SecretVariable represents a custom variable with encrypted value.
/// Corresponds to Go's `fleet.SecretVariable`.
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

/// AppConfig is the top-level application configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub org_info: OrgInfo,
    pub server_settings: ServerSettings,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smtp_settings: Option<SMTPSettings>,
    pub host_expiry_settings: crate::team::HostExpirySettings,
    pub activity_expiry_settings: ActivityExpirySettings,
    pub features: crate::team::Features,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_options: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sso_settings: Option<SSOSettings>,
    pub fleet_desktop: FleetDesktopSettings,
    pub vulnerability_settings: VulnerabilitySettings,
    pub webhook_settings: WebhookSettings,
    pub integrations: Integrations,
    pub mdm: MDMConfig,
    pub gitops: UIGitOpsModeConfig,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scripts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditional_access: Option<ConditionalAccessSettings>,
}

/// HostStatusWebhookSettings configures host status webhooks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostStatusWebhookSettings {
    pub enable_host_status_webhook: bool,
    pub destination_url: String,
    pub host_percentage: u32,
    pub days_count: u32,
}

/// FailingPoliciesWebhookSettings configures failing policies webhooks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FailingPoliciesWebhookSettings {
    pub enable_failing_policies_webhook: bool,
    pub destination_url: String,
    #[serde(default)]
    pub policy_ids: Vec<u32>,
    pub host_batch_size: u32,
}

/// VulnerabilitiesWebhookSettings configures vulnerability webhooks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VulnerabilitiesWebhookSettings {
    pub enable_vulnerabilities_webhook: bool,
    pub destination_url: String,
    pub host_batch_size: u32,
}

/// ActivitiesWebhookSettings configures activity webhooks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivitiesWebhookSettings {
    pub enable: bool,
    pub destination_url: String,
}

/// JiraIntegration holds global Jira integration configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIntegration {
    pub url: String,
    pub username: String,
    #[serde(skip)]
    pub api_token: String,
    pub project_key: String,
    pub enable_failing_policies: bool,
    pub enable_software_vulnerabilities: bool,
}

/// ZendeskIntegration holds global Zendesk integration configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZendeskIntegration {
    pub url: String,
    pub email: String,
    #[serde(skip)]
    pub api_token: String,
    pub group_id: u64,
    pub enable_failing_policies: bool,
    pub enable_software_vulnerabilities: bool,
}

/// GoogleCalendarIntegration holds global Google Calendar configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleCalendarIntegration {
    pub domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<serde_json::Value>,
}

/// LicenseInfo holds information about the Fleet license.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub tier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// EnrichedAppConfig extends AppConfig with additional runtime fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnrichedAppConfig {
    #[serde(flatten)]
    pub app_config: AppConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_interval: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vulnerabilities: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<LicenseInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<serde_json::Value>,
}

/// ApplySpecOptions controls how specs are applied.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApplySpecOptions {
    pub force: bool,
    pub dry_run: bool,
    pub no_cache: bool,
    pub overwrite: bool,
}
