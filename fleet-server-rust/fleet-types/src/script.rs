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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_content_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<u32>,
    #[serde(default)]
    pub sync_request: bool,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_execution_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<u32>,
    #[serde(default)]
    pub sync_request: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub hostname: String,
}

/// HostLockWipeStatus represents the lock/wipe status of a host.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostLockWipeStatus {
    pub host_fleet_platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_script: Option<HostScriptResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlock_script: Option<HostScriptResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wipe_script: Option<HostScriptResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlock_pin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlock_requested_at: Option<DateTime<Utc>>,
}

/// HostScriptResultPayload is the payload submitted by a host reporting script results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostScriptResultPayload {
    pub host_id: u32,
    pub execution_id: String,
    pub output: String,
    pub runtime: i32,
    pub exit_code: i64,
    #[serde(default)]
    pub timeout: bool,
}

/// BatchScriptHost represents a host in a batch script execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchScriptHost {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computer_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_serial: Option<String>,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_execution_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_executed_at: Option<DateTime<Utc>>,
    pub status: String,
}

/// ScriptPayload is the payload for creating/uploading a script.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptPayload {
    pub name: String,
    pub script_contents: String,
}

/// ScriptResponse is the response when listing scripts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResponse {
    pub team_id: Option<u32>,
    pub id: u32,
    pub name: String,
}

/// DeviceStatus represents the device status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceStatus {
    #[serde(rename = "unlocked")]
    Unlocked,
    #[serde(rename = "locked")]
    Locked,
    #[serde(rename = "wiped")]
    Wiped,
}

/// PendingDeviceAction represents a pending action on a device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PendingDeviceAction {
    #[serde(rename = "lock")]
    Lock,
    #[serde(rename = "unlock")]
    Unlock,
    #[serde(rename = "wipe")]
    Wipe,
    #[serde(rename = "")]
    None,
}
