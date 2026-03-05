//! Invite types matching Go's `server/fleet/invites.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::team::UserTeam;

/// InvitePayload contains fields required to create or update a user invite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitePayload {
    pub email: Option<String>,
    pub name: Option<String>,
    pub position: Option<String>,
    pub sso_enabled: Option<bool>,
    pub mfa_enabled: Option<bool>,
    pub global_role: Option<String>,
    #[serde(default)]
    pub teams: Vec<UserTeam>,
}

/// Invite represents an invitation for a user to join Fleet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invite {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: u32,
    pub invited_by: u32,
    pub email: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub position: String,
    #[serde(skip)]
    pub token: String,
    pub sso_enabled: bool,
    pub mfa_enabled: bool,
    pub global_role: Option<String>,
    #[serde(default)]
    pub teams: Vec<UserTeam>,
}
