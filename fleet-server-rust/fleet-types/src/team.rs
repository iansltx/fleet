//! Team types matching Go's `server/fleet/teams.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::{
    AndroidSettings, AppleOSUpdateSettings, Features, HostExpirySettings, MacOSSetup,
    MacOSSettings, WindowsSettings, WindowsUpdates,
};
use crate::enroll::EnrollSecret;
use crate::host::Host;
use crate::user::User;

// ─── Role constants ─────────────────────────────────────────────────────────

pub const ROLE_ADMIN: &str = "admin";
pub const ROLE_MAINTAINER: &str = "maintainer";
pub const ROLE_OBSERVER: &str = "observer";
pub const ROLE_OBSERVER_PLUS: &str = "observer_plus";
pub const ROLE_GITOPS: &str = "gitops";
pub const ROLE_TECHNICIAN: &str = "technician";

pub const TEAM_NAME_NO_TEAM: &str = "No team";
pub const TEAM_NAME_ALL_TEAMS: &str = "All teams";

pub const RESERVED_NAME_ALL_TEAMS: &str = "All teams";
pub const RESERVED_NAME_NO_TEAM: &str = "No team";

pub const DISPLAY_NAME_NO_TEAM: &str = "Unassigned";
pub const DISPLAY_NAME_ALL_TEAMS: &str = "All fleets";

pub const TEAM_KIND: &str = "team";
pub const FLEET_KIND: &str = "fleet";

// ─── Webhook settings ───────────────────────────────────────────────────────

/// HostStatusWebhookSettings for teams (same shape as global).
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

/// FailingPoliciesWebhookSettings for teams.
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

/// TeamWebhookSettings holds webhook-related settings for a team.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamWebhookSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_status_webhook: Option<HostStatusWebhookSettings>,
    #[serde(default)]
    pub failing_policies_webhook: FailingPoliciesWebhookSettings,
}

// ─── Team integration types ─────────────────────────────────────────────────

/// TeamJiraIntegration holds Jira configuration for a team.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamJiraIntegration {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub project_key: String,
    #[serde(default)]
    pub enable_failing_policies: bool,
}

/// TeamZendeskIntegration holds Zendesk configuration for a team.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamZendeskIntegration {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub group_id: i64,
    #[serde(default)]
    pub enable_failing_policies: bool,
}

/// TeamGoogleCalendarIntegration holds Google Calendar configuration for a team.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamGoogleCalendarIntegration {
    #[serde(default, rename = "enable_calendar_events")]
    pub enable: bool,
    #[serde(default)]
    pub webhook_url: String,
}

/// TeamIntegrations holds integration settings for a team.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamIntegrations {
    #[serde(default)]
    pub jira: Vec<TeamJiraIntegration>,
    #[serde(default)]
    pub zendesk: Vec<TeamZendeskIntegration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_calendar: Option<TeamGoogleCalendarIntegration>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conditional_access_enabled: Option<bool>,
}

// ─── TeamMDM ────────────────────────────────────────────────────────────────

/// TeamMDM holds MDM settings for a team.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamMDM {
    #[serde(default)]
    pub enable_disk_encryption: bool,
    #[serde(default)]
    pub enable_recovery_lock_password: bool,
    #[serde(rename = "windows_require_bitlocker_pin", default)]
    pub require_bitlocker_pin: bool,
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
    pub windows_settings: WindowsSettings,
    #[serde(default)]
    pub android_settings: AndroidSettings,
}

/// TeamPayloadMDM is a payload-oriented version of TeamMDM with optional fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamPayloadMDM {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_disk_encryption: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_recovery_lock_password: Option<bool>,
    #[serde(rename = "windows_require_bitlocker_pin", skip_serializing_if = "Option::is_none")]
    pub require_bitlocker_pin: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos_updates: Option<AppleOSUpdateSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ios_updates: Option<AppleOSUpdateSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipados_updates: Option<AppleOSUpdateSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows_updates: Option<WindowsUpdates>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos_setup: Option<MacOSSetup>,
}

/// TeamSpecMDM is used in team specs (YAML/gitops) with optional booleans.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamSpecMDM {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_disk_encryption: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_recovery_lock_password: Option<bool>,
    #[serde(rename = "windows_require_bitlocker_pin", skip_serializing_if = "Option::is_none")]
    pub require_bitlocker_pin: Option<bool>,
    #[serde(default)]
    pub macos_updates: AppleOSUpdateSettings,
    #[serde(default)]
    pub ios_updates: AppleOSUpdateSettings,
    #[serde(default)]
    pub ipados_updates: AppleOSUpdateSettings,
    #[serde(default)]
    pub windows_updates: WindowsUpdates,
    /// A map for macos settings to detect sub-key presence.
    #[serde(default)]
    pub macos_settings: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    pub macos_setup: MacOSSetup,
    #[serde(default)]
    pub windows_settings: WindowsSettings,
    #[serde(default)]
    pub android_settings: AndroidSettings,
}

// ─── TeamConfig ─────────────────────────────────────────────────────────────

/// TeamConfig holds the configuration for a team.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_options: Option<serde_json::Value>,
    #[serde(default)]
    pub host_expiry_settings: HostExpirySettings,
    #[serde(default)]
    pub webhook_settings: TeamWebhookSettings,
    #[serde(default)]
    pub integrations: TeamIntegrations,
    #[serde(default)]
    pub mdm: TeamMDM,
    #[serde(default)]
    pub features: Features,
    #[serde(default)]
    pub scripts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software: Option<serde_json::Value>,
}

/// TeamConfigLite contains only TeamConfig fields stored as-is from the teams.config JSON.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamConfigLite {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_options: Option<serde_json::Value>,
    #[serde(default)]
    pub host_expiry_settings: HostExpirySettings,
    #[serde(default)]
    pub webhook_settings: TeamWebhookSettings,
    #[serde(default)]
    pub integrations: TeamIntegrations,
    #[serde(default)]
    pub mdm: TeamMDM,
}

// ─── TeamPayload ────────────────────────────────────────────────────────────

/// TeamPayload is the payload for creating/modifying a team.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<Vec<EnrollSecret>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_settings: Option<TeamWebhookSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrations: Option<TeamIntegrations>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mdm: Option<TeamPayloadMDM>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_expiry_settings: Option<HostExpirySettings>,
}

// ─── Team ───────────────────────────────────────────────────────────────────

/// Team represents a group of hosts and users that can perform operations on those hosts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gitops_filename: Option<String>,
    pub created_at: DateTime<Utc>,
    pub name: String,
    #[serde(default)]
    pub description: String,
    // Config is flattened in JSON output (matching Go's custom MarshalJSON).
    #[serde(flatten)]
    pub config: TeamConfig,
    #[serde(default)]
    pub user_count: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<TeamUser>,
    #[serde(default)]
    pub host_count: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hosts: Vec<Host>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secrets: Option<Vec<EnrollSecret>>,
}

/// TeamUser represents a user with a role on a team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamUser {
    #[serde(flatten)]
    pub user: User,
    pub role: String,
}

/// UserTeam represents a team that a user belongs to with their role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTeam {
    #[serde(flatten)]
    pub team: Team,
    pub role: String,
}

/// TeamRole holds a user's role on a specific team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamRole {
    pub id: u32,
    pub role: String,
}

/// TeamLite is a lightweight representation of a team with config from DB JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamLite {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gitops_filename: Option<String>,
    pub created_at: DateTime<Utc>,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub config: TeamConfigLite,
}

/// TeamSummary contains the minimal team fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSummary {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub description: String,
}

/// TeamFilter is used to filter teams in queries.
#[derive(Debug, Clone)]
pub struct TeamFilter {
    pub user: Option<User>,
    pub include_observer: bool,
    pub team_id: Option<u32>,
}

// ─── TeamSpec types ─────────────────────────────────────────────────────────

/// TeamSpecWebhookSettings holds webhook settings used in team specs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamSpecWebhookSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_status_webhook: Option<HostStatusWebhookSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failing_policies_webhook: Option<FailingPoliciesWebhookSettings>,
}

/// TeamSpecIntegrations holds integration settings used in team specs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamSpecIntegrations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_calendar: Option<TeamGoogleCalendarIntegration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditional_access_enabled: Option<bool>,
}

/// TeamSpec is the spec format for teams (used in YAML/gitops).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamSpec {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gitops_filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_options: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_expiry_settings: Option<HostExpirySettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<Vec<EnrollSecret>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<serde_json::Value>,
    #[serde(default)]
    pub mdm: TeamSpecMDM,
    #[serde(default)]
    pub scripts: Vec<String>,
    #[serde(default)]
    pub webhook_settings: TeamSpecWebhookSettings,
    #[serde(default)]
    pub integrations: TeamSpecIntegrations,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software: Option<serde_json::Value>,
}

// ─── Default team types ─────────────────────────────────────────────────────

/// DefaultTeam represents the limited team information returned for team ID 0.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DefaultTeam {
    pub id: u32,
    pub name: String,
    #[serde(flatten)]
    pub config: DefaultTeamConfig,
}

/// DefaultTeamConfig holds configuration for team ID 0.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DefaultTeamConfig {
    #[serde(default)]
    pub webhook_settings: DefaultTeamWebhookSettings,
    #[serde(default)]
    pub integrations: DefaultTeamIntegrations,
}

/// DefaultTeamWebhookSettings contains webhook settings for team ID 0.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DefaultTeamWebhookSettings {
    #[serde(default)]
    pub failing_policies_webhook: FailingPoliciesWebhookSettings,
}

/// DefaultTeamIntegrations contains only the integrations supported for team ID 0.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DefaultTeamIntegrations {
    #[serde(default)]
    pub jira: Vec<TeamJiraIntegration>,
    #[serde(default)]
    pub zendesk: Vec<TeamZendeskIntegration>,
}

// ─── TeamSpec software types ────────────────────────────────────────────────

/// TeamSpecSoftwareAsset represents a software asset path in team specs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamSpecSoftwareAsset {
    #[serde(default)]
    pub path: String,
}

/// TeamSpecAppStoreApp represents a VPP app in team specs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamSpecAppStoreApp {
    #[serde(default)]
    pub app_store_id: String,
    #[serde(default)]
    pub self_service: bool,
    #[serde(default)]
    pub labels_include_any: Vec<String>,
    #[serde(default)]
    pub labels_exclude_any: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(rename = "setup_experience", skip_serializing_if = "Option::is_none")]
    pub install_during_setup: Option<bool>,
    #[serde(default)]
    pub icon: TeamSpecSoftwareAsset,
    #[serde(default)]
    pub platform: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub display_name: String,
    #[serde(default)]
    pub configuration: TeamSpecSoftwareAsset,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_update_enabled: Option<bool>,
    #[serde(rename = "auto_update_window_start", skip_serializing_if = "Option::is_none")]
    pub auto_update_start_time: Option<String>,
    #[serde(rename = "auto_update_window_end", skip_serializing_if = "Option::is_none")]
    pub auto_update_end_time: Option<String>,
}

// ─── Helper functions ───────────────────────────────────────────────────────

/// Checks if a team name is reserved (case-insensitive).
pub fn is_reserved_team_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower == "no team" || lower == "all teams" || lower == "unassigned" || lower == "all fleets"
}

/// Checks if a role is a valid team role.
pub fn valid_team_role(role: &str) -> bool {
    matches!(
        role,
        ROLE_ADMIN
            | ROLE_MAINTAINER
            | ROLE_OBSERVER
            | ROLE_OBSERVER_PLUS
            | ROLE_GITOPS
            | ROLE_TECHNICIAN
    )
}

/// Checks if a role is a valid global role.
pub fn valid_global_role(role: &str) -> bool {
    matches!(
        role,
        ROLE_OBSERVER
            | ROLE_MAINTAINER
            | ROLE_ADMIN
            | ROLE_TECHNICIAN
            | ROLE_OBSERVER_PLUS
            | ROLE_GITOPS
    )
}
