//! SCIM types matching Go's `server/fleet/scim.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Constants ───────────────────────────────────────────────────────────────

/// SCIMMaxFieldLength is the default maximum length for SCIM fields.
pub const SCIM_MAX_FIELD_LENGTH: usize = 255;

// ─── ScimUser ────────────────────────────────────────────────────────────────

/// ScimUser represents a SCIM user in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimUser {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    pub user_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub emails: Vec<ScimUserEmail>,
    #[serde(default)]
    pub groups: Vec<ScimUserGroup>,
}

impl ScimUser {
    /// Returns the display name for the SCIM user.
    pub fn display_name(&self) -> String {
        match (&self.given_name, &self.family_name) {
            (Some(given), Some(family)) if !given.is_empty() && !family.is_empty() => {
                format!("{} {}", given, family)
            }
            (Some(given), _) if !given.is_empty() => given.clone(),
            (_, Some(family)) if !family.is_empty() => family.clone(),
            _ => String::new(),
        }
    }
}

// ─── ScimUserEmail ───────────────────────────────────────────────────────────

/// ScimUserEmail represents an email address associated with a SCIM user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimUserEmail {
    pub scim_user_id: u32,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub email_type: Option<String>,
}

impl ScimUserEmail {
    /// Generates a unique string representation of the email for comparison.
    pub fn generate_comparison_key(&self) -> String {
        let type_value = self.email_type.as_deref().unwrap_or("nil");
        let primary_value = match self.primary {
            Some(true) => "true",
            Some(false) => "false",
            None => "nil",
        };
        format!("{}:{}:{}", self.email, type_value, primary_value)
    }
}

// ─── ScimUserGroup ───────────────────────────────────────────────────────────

/// ScimUserGroup represents a group associated with a SCIM user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimUserGroup {
    pub id: u32,
    pub display_name: String,
}

// ─── ScimListOptions ─────────────────────────────────────────────────────────

/// ScimListOptions holds pagination options for SCIM list operations.
#[derive(Debug, Clone, Default)]
pub struct ScimListOptions {
    /// 1-based index of the first result to return.
    pub start_index: u32,
    /// How many results per page.
    pub per_page: u32,
}

// ─── ScimUsersListOptions ────────────────────────────────────────────────────

/// ScimUsersListOptions holds options for listing SCIM users.
#[derive(Debug, Clone, Default)]
pub struct ScimUsersListOptions {
    pub scim_list_options: ScimListOptions,
    /// Filters by userName -- max of 1 response is expected.
    pub user_name_filter: Option<String>,
    /// Email type filter for Entra ID style queries.
    pub email_type_filter: Option<String>,
    /// Email value filter for Entra ID style queries.
    pub email_value_filter: Option<String>,
}

// ─── ScimGroupsListOptions ───────────────────────────────────────────────────

/// ScimGroupsListOptions holds options for listing SCIM groups.
#[derive(Debug, Clone, Default)]
pub struct ScimGroupsListOptions {
    pub scim_list_options: ScimListOptions,
    /// Filters by displayName.
    pub display_name_filter: Option<String>,
    /// If true, the group's users will not be fetched.
    pub exclude_users: bool,
}

// ─── ScimGroup ───────────────────────────────────────────────────────────────

/// ScimGroup represents a SCIM group in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimGroup {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    pub display_name: String,
    #[serde(default)]
    pub scim_users: Vec<u32>,
}

// ─── ScimLastRequest ─────────────────────────────────────────────────────────

/// ScimLastRequest represents the last SCIM request status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimLastRequest {
    pub status: String,
    pub details: String,
    pub requested_at: DateTime<Utc>,
}

// ─── ScimDetails ─────────────────────────────────────────────────────────────

/// ScimDetails holds SCIM integration details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_request: Option<ScimLastRequest>,
}
