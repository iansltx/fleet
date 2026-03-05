//! Script management and execution endpoints.
//!
//! Handles script CRUD, execution, batch operations, and result retrieval.

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::response::{fleet_error, fleet_ok, FleetResponse};
use crate::AppState;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RunScriptBody {
    pub host_id: u64,
    pub script_id: Option<u64>,
    pub script_contents: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RunScriptSyncBody {
    pub host_id: u64,
    pub script_id: Option<u64>,
    pub script_contents: Option<String>,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct BatchScriptRunBody {
    pub host_ids: Vec<u64>,
    pub script_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct ListScriptsParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub team_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct BatchSetScriptsBody {
    pub scripts: Vec<serde_json::Value>,
    pub team_id: Option<u64>,
    pub dry_run: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct BatchScriptExecutionListParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct BatchScriptExecutionHostResultsParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/scripts/run
pub async fn run_script(
    State(_state): State<AppState>,
    Json(_body): Json<RunScriptBody>,
) -> FleetResponse {
    fleet_ok("execution_id", serde_json::json!(""))
}

/// POST /api/_version_/fleet/scripts/run/sync
pub async fn run_script_sync(
    State(_state): State<AppState>,
    Json(_body): Json<RunScriptSyncBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/scripts/run/batch
pub async fn batch_script_run(
    State(_state): State<AppState>,
    Json(_body): Json<BatchScriptRunBody>,
) -> FleetResponse {
    fleet_ok("batch_execution_id", serde_json::json!(""))
}

/// GET /api/_version_/fleet/scripts/results/{execution_id}
pub async fn get_script_result(
    State(_state): State<AppState>,
    Path(_execution_id): Path<String>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/scripts
pub async fn create_script(
    State(_state): State<AppState>,
) -> FleetResponse {
    // TODO: handle multipart upload of script content
    fleet_ok("script", serde_json::json!({}))
}

/// GET /api/_version_/fleet/scripts
pub async fn list_scripts(
    State(_state): State<AppState>,
    Query(_params): Query<ListScriptsParams>,
) -> FleetResponse {
    fleet_ok("scripts", serde_json::json!([]))
}

/// GET /api/_version_/fleet/scripts/{script_id}
pub async fn get_script(
    State(_state): State<AppState>,
    Path(_script_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("script", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/scripts/{script_id}
pub async fn update_script(
    State(_state): State<AppState>,
    Path(_script_id): Path<u64>,
) -> FleetResponse {
    // TODO: handle multipart upload
    fleet_ok("script", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/scripts/{script_id}
pub async fn delete_script(
    State(_state): State<AppState>,
    Path(_script_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/scripts/batch
pub async fn batch_set_scripts(
    State(_state): State<AppState>,
    Json(_body): Json<BatchSetScriptsBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/scripts/batch/{batch_execution_id}/cancel
pub async fn batch_script_cancel(
    State(_state): State<AppState>,
    Path(_batch_execution_id): Path<String>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/scripts/batch/summary/{batch_execution_id}
pub async fn batch_script_execution_summary(
    State(_state): State<AppState>,
    Path(_batch_execution_id): Path<String>,
) -> FleetResponse {
    fleet_ok("summary", serde_json::json!({}))
}

/// GET /api/_version_/fleet/scripts/batch/{batch_execution_id}/host-results
pub async fn batch_script_execution_host_results(
    State(_state): State<AppState>,
    Path(_batch_execution_id): Path<String>,
    Query(_params): Query<BatchScriptExecutionHostResultsParams>,
) -> FleetResponse {
    fleet_ok("host_results", serde_json::json!([]))
}

/// GET /api/_version_/fleet/scripts/batch/{batch_execution_id}
pub async fn batch_script_execution_status(
    State(_state): State<AppState>,
    Path(_batch_execution_id): Path<String>,
) -> FleetResponse {
    fleet_ok("status", serde_json::json!({}))
}
