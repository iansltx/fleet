//! Team types matching Go's `server/fleet/teams.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::enroll::EnrollSecret;
use crate::host::Host;
use crate::user::User;

// Role constants
pub const ROLE_ADMIN: &str = "admin";
pub const ROLE_MAINTAINER: &str = "maintainer";
pub const ROLE_OBSERVER: &str = "observer";
pub const ROLE_OBSERVER_PLUS: &str = "observer_plus";
pub const ROLE_GITOPS: &str = "gitops";
pub const ROLE_TECHNICIAN: &str = "technician";

/// HostExpirySettings controls host expiry behavior.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostExpirySettings {
    pub host_expiry_enabled: bool,
    #[serde(default)]
    pub host_expiry_window: u32,
}

/// Features allows enabling or disabling features.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Features {
    #[serde(default)]
    pub enable_host_users: bool,
    #[serde(default)]
    pub enable_software_inventory: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_queries: Option<serde_json::Value>,
}

/// TeamWebhookSettings holds webhook-related settings for a team.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamWebhookSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failing_policies_webhook: Option<FailingPoliciesWebhook>,
}

/// FailingPoliciesWebhook configures the webhook for failing policies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FailingPoliciesWebhook {
    pub enable_failing_policies_webhook: bool,
    pub destination_url: String,
    #[serde(default)]
    pub policy_ids: Vec<u32>,
    pub host_batch_size: u32,
}

/// TeamIntegrations holds integration settings for a team.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamIntegrations {
    // Jira, Zendesk, and other integrations can be added here.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_calendar: Option<serde_json::Value>,
}

/// TeamMDM holds MDM settings for a team.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamMDM {
    #[serde(default)]
    pub enable_disk_encryption: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos_updates: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows_updates: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos_setup: Option<serde_json::Value>,
    // Additional MDM fields can be added here.
}

/// TeamConfig holds the configuration for a team.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_options: Option<serde_json::Value>,
    pub host_expiry_settings: HostExpirySettings,
    pub webhook_settings: TeamWebhookSettings,
    pub integrations: TeamIntegrations,
    pub mdm: TeamMDM,
    pub features: Features,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scripts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software: Option<serde_json::Value>,
}

/// TeamPayload is the payload for creating/modifying a team.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub secrets: Option<Vec<EnrollSecret>>,
    pub webhook_settings: Option<TeamWebhookSettings>,
    pub integrations: Option<TeamIntegrations>,
    pub mdm: Option<serde_json::Value>,
    pub host_expiry_settings: Option<HostExpirySettings>,
}

/// Team represents a group of hosts and users that can perform operations on those hosts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gitops_filename: Option<String>,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub description: String,
    // Config is flattened in JSON output (matching Go's custom MarshalJSON).
    #[serde(flatten)]
    pub config: TeamConfig,
    pub user_count: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<TeamUser>,
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

/// Checks if a team name is reserved (case-insensitive).
pub fn is_reserved_team_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower == "no team" || lower == "all teams" || lower == "unassigned" || lower == "all fleets"
}
