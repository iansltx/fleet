//! EnrollSecret types matching Go's `server/fleet/app.go` (EnrollSecret struct).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
}
