//! User types matching Go's `server/fleet/users.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::team::UserTeam;

/// UserSummary contains the minimal user fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub gravatar_url: String,
    pub api_only: bool,
}

/// UserSettings contains user-specific settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserSettings {
    #[serde(default)]
    pub hidden_host_columns: Vec<String>,
}

/// UserListOptions defines options for listing users.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserListOptions {
    #[serde(flatten)]
    pub list_options: crate::ListOptions,
    pub team_id: Option<u32>,
}

/// User is the model struct that represents a Fleet user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: u32,
    /// Password is excluded from JSON serialization.
    #[serde(skip)]
    pub password: Vec<u8>,
    /// Salt is excluded from JSON serialization.
    #[serde(skip)]
    pub salt: String,
    pub name: String,
    pub email: String,
    #[serde(rename = "force_password_reset")]
    pub admin_forced_password_reset: bool,
    pub gravatar_url: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub position: String,
    pub sso_enabled: bool,
    pub mfa_enabled: bool,
    pub global_role: Option<String>,
    pub api_only: bool,

    /// Teams is the teams this user has roles in.
    #[serde(default)]
    pub teams: Vec<UserTeam>,

    /// Only used to prevent duplicate invite acceptance.
    #[serde(skip)]
    pub invite_id: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<UserSettings>,

    #[serde(skip)]
    pub deleted: bool,
}

impl User {
    /// Returns true if user is either a Global Observer or a Global Observer+.
    pub fn is_global_observer(&self) -> bool {
        match &self.global_role {
            Some(role) => role == "observer" || role == "observer_plus",
            None => false,
        }
    }

    /// Returns true if user is a global admin.
    pub fn is_global_admin(&self) -> bool {
        match &self.global_role {
            Some(role) => role == "admin",
            None => false,
        }
    }

    pub fn is_admin_forced_password_reset(&self) -> bool {
        if self.sso_enabled {
            return false;
        }
        self.admin_forced_password_reset
    }
}

/// UserPayload is the payload for creating/modifying a user.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPayload {
    pub name: Option<String>,
    pub email: Option<String>,
    #[serde(skip)]
    pub password: Option<String>,
    pub gravatar_url: Option<String>,
    pub position: Option<String>,
    pub invite_token: Option<String>,
    pub sso_invite: Option<bool>,
    pub mfa_enabled: Option<bool>,
    pub sso_enabled: Option<bool>,
    pub global_role: Option<String>,
    pub admin_forced_password_reset: Option<bool>,
    pub api_only: Option<bool>,
    pub teams: Option<Vec<UserTeam>>,
    #[serde(skip)]
    pub new_password: Option<String>,
    pub settings: Option<UserSettings>,
    #[serde(skip)]
    pub invite_id: Option<u32>,
}
