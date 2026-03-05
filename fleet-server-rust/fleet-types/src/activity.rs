//! Activity types matching Go's `server/fleet/activities.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Activity represents an audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: u32,
    pub created_at: DateTime<Utc>,
    pub user_id: Option<u32>,
    pub user_name: String,
    pub user_email: String,
    pub activity_type: String,
    pub details: serde_json::Value,
}
