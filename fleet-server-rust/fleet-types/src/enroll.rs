//! EnrollSecret types matching Go's `server/fleet/app.go` (EnrollSecret struct).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const ENROLL_SECRET_KIND: &str = "enroll_secret";
pub const ENROLL_SECRET_DEFAULT_LENGTH: usize = 24;
/// Maximum number of enroll secrets that can be set per team, or globally.
pub const MAX_ENROLL_SECRETS_COUNT: usize = 50;

/// EnrollSecret represents an enrollment secret used by hosts to enroll in Fleet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollSecret {
    pub secret: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
}

impl EnrollSecret {
    /// Returns the team ID, if any.
    pub fn get_team_id(&self) -> Option<u32> {
        self.team_id
    }

    /// Returns whether the secret is global (no team).
    pub fn is_global_secret(&self) -> bool {
        self.team_id.is_none()
    }
}

/// EnrollSecretSpec is the fleetctl spec type for enroll secrets.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnrollSecretSpec {
    pub secrets: Vec<EnrollSecret>,
}
