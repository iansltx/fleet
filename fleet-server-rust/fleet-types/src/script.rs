//! Script types matching Go's `server/fleet/scripts.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Constants ───────────────────────────────────────────────────────────────

pub const SAVED_SCRIPT_MAX_RUNE_LEN: usize = 500000;
pub const UNSAVED_SCRIPT_MAX_RUNE_LEN: usize = 10000;

// ─── Script ──────────────────────────────────────────────────────────────────

/// Script represents a saved script that can be executed on a host.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Script {
    pub id: u32,
    pub team_id: Option<u32>,
    pub name: String,
    #[serde(skip)]
    pub script_contents: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip)]
    pub script_content_id: u32,
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
    #[serde(skip)]
    pub host_id: u32,
    #[serde(skip)]
    pub script_id: u32,
    #[serde(skip)]
    pub hsr_id: u32,
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
    pub policy_id: Option<u32>,
    #[serde(default)]
    pub script_contents: String,
    #[serde(skip)]
    pub script_content_id: u32,
    #[serde(default)]
    pub script_name: String,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub team_id: u32,
    #[serde(skip)]
    pub user_id: Option<u32>,
    #[serde(skip)]
    pub sync_request: bool,
    #[serde(skip)]
    pub setup_experience_script_id: Option<u32>,
}

fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}

/// HostScriptResultPayload is the payload submitted by a host reporting script results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostScriptResultPayload {
    pub host_id: u32,
    pub execution_id: String,
    pub output: String,
    pub runtime: i32,
    pub exit_code: i32,
    #[serde(default)]
    pub timeout: i32,
}

/// HostScriptResult contains the result of a script execution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostScriptResult {
    #[serde(skip)]
    pub id: u32,
    pub host_id: u32,
    #[serde(default)]
    pub execution_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_execution_id: Option<String>,
    #[serde(skip)]
    pub script_contents: String,
    #[serde(default)]
    pub output: String,
    #[serde(default)]
    pub runtime: i32,
    pub exit_code: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i32>,
    #[serde(skip)]
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<u32>,
    #[serde(skip)]
    pub user_id: Option<u32>,
    #[serde(skip)]
    pub sync_request: bool,
    pub team_id: Option<u32>,
    #[serde(default)]
    pub message: String,
    #[serde(skip)]
    pub hostname: String,
    #[serde(skip)]
    pub host_deleted_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub setup_experience_script_id: Option<u32>,
    #[serde(skip)]
    pub canceled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempt_number: Option<i32>,
    /// host_timeout is a derived field (not in Go struct) kept for downstream compatibility.
    #[serde(default)]
    pub host_timeout: bool,
    /// updated_at is kept for downstream compatibility.
    #[serde(skip)]
    pub updated_at: DateTime<Utc>,
}

/// BatchScriptHost represents a host in a batch script execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchScriptHost {
    pub id: u32,
    #[serde(skip)]
    pub computer_name: String,
    #[serde(skip)]
    pub hostname: String,
    #[serde(skip)]
    pub hardware_model: String,
    #[serde(skip)]
    pub hardware_serial: String,
    pub display_name: String,
    pub script_execution_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub script_output_preview: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_executed_at: Option<DateTime<Utc>>,
    pub script_status: String,
}

/// ScriptPayload is the payload for creating/uploading a script.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptPayload {
    pub name: String,
    pub script_contents: Vec<u8>,
}

/// ScriptResponse is the response when listing scripts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResponse {
    pub team_id: Option<u32>,
    pub id: u32,
    pub name: String,
}

// ─── Lock/Wipe Status ────────────────────────────────────────────────────────

/// HostLockWipeStatus represents the lock/wipe status of a host.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostLockWipeStatus {
    pub host_fleet_platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_script: Option<HostScriptResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlock_pin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlock_requested_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlock_script: Option<HostScriptResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wipe_script: Option<HostScriptResult>,
    #[serde(default)]
    pub location_pending: bool,
}

/// DeviceStatus represents the device status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceStatus {
    #[serde(rename = "wiped")]
    Wiped,
    #[serde(rename = "locked")]
    Locked,
    #[serde(rename = "unlocked")]
    Unlocked,
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
    #[serde(rename = "location")]
    Location,
    #[serde(rename = "")]
    None,
}

// ─── Batch Execution ─────────────────────────────────────────────────────────

/// BatchScriptExecutionStatus represents the status of a batch script execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BatchScriptExecutionStatus {
    #[serde(rename = "ran")]
    Ran,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "errored")]
    Errored,
    #[serde(rename = "canceled")]
    Canceled,
    #[serde(rename = "incompatible")]
    Incompatible,
}

/// BatchExecutionStatusFilter filters batch execution status results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecutionStatusFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

/// BatchExecutionHost represents a host in a batch execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecutionHost {
    pub host_id: u32,
    pub host_display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// ScheduledBatchExecutionStatus represents the status of a scheduled batch execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScheduledBatchExecutionStatus {
    #[serde(rename = "started")]
    Started,
    #[serde(rename = "scheduled")]
    Scheduled,
    #[serde(rename = "finished")]
    Finished,
}

/// BatchExecutionActivityType represents the type of batch execution activity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BatchExecutionActivityType {
    #[serde(rename = "script")]
    Script,
}

/// BatchActivity represents a batch script execution activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchActivity {
    pub id: u32,
    pub batch_execution_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<u32>,
    #[serde(skip)]
    pub job_id: Option<u32>,
    #[serde(skip)]
    pub activity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_id: Option<u32>,
    pub script_name: String,
    pub team_id: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_before: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub canceled: bool,
    pub status: ScheduledBatchExecutionStatus,
    #[serde(rename = "targeted_host_count", skip_serializing_if = "Option::is_none")]
    pub num_targeted: Option<u32>,
    #[serde(rename = "pending_host_count", skip_serializing_if = "Option::is_none")]
    pub num_pending: Option<u32>,
    #[serde(rename = "ran_host_count", skip_serializing_if = "Option::is_none")]
    pub num_ran: Option<u32>,
    #[serde(rename = "errored_host_count", skip_serializing_if = "Option::is_none")]
    pub num_errored: Option<u32>,
    #[serde(rename = "canceled_host_count", skip_serializing_if = "Option::is_none")]
    pub num_canceled: Option<u32>,
    #[serde(rename = "incompatible_host_count", skip_serializing_if = "Option::is_none")]
    pub num_incompatible: Option<u32>,
}

/// BatchActivityHostResult stores a host's result for a batch activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchActivityHostResult {
    pub id: u32,
    pub batch_execution_id: String,
    pub host_id: u32,
    pub host_execution_id: Option<String>,
    pub error: Option<String>,
}

/// BatchActivityScriptJobArgs contains args for a batch script job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchActivityScriptJobArgs {
    pub execution_id: String,
}

pub const BATCH_ACTIVITY_SCRIPTS_JOB_NAME: &str = "batch_scripts";
