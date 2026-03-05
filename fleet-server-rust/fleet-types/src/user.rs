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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<UserSettings>,
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
