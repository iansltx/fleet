//! Script types matching Go's `server/fleet/scripts.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Script represents a saved script that can be executed on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: u32,
    pub team_id: Option<u32>,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// HostScriptDetail represents the details of a script that applies to a specific host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostScriptDetail {
    #[serde(skip)]
    pub host_id: u32,
    pub script_id: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_execution: Option<HostScriptExecution>,
}

/// HostScriptExecution represents a single execution of a script on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostScriptExecution {
    pub execution_id: String,
    pub executed_at: DateTime<Utc>,
    /// Status is one of "pending", "ran", or "error".
    pub status: String,
}

/// HostScriptRequestPayload is the payload for running a script on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostScriptRequestPayload {
    pub host_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_contents: Option<String>,
}

/// HostScriptResult contains the result of a script execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostScriptResult {
    pub id: u32,
    pub host_id: u32,
    pub execution_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_id: Option<u32>,
    pub script_contents: String,
    pub output: String,
    pub runtime: i32,
    pub exit_code: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub host_timeout: bool,
    pub host_deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
