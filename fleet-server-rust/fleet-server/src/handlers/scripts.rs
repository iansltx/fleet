//! Script management and execution endpoints.
//!
//! Handles script CRUD, execution, batch operations, and result retrieval.

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;

use crate::middleware::auth::AuthenticatedUser;
use crate::response::{fleet_error, fleet_ok, encode_service_error, FleetResponse};
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
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(_body): Json<RunScriptBody>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Script execution requires async job queue (not yet implemented)
    fleet_ok("execution_id", serde_json::json!(""))
}

/// POST /api/_version_/fleet/scripts/run/sync
pub async fn run_script_sync(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(_body): Json<RunScriptSyncBody>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Sync script execution requires waiting for result (not yet implemented)
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/scripts/run/batch
pub async fn batch_script_run(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(_body): Json<BatchScriptRunBody>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("batch_execution_id", serde_json::json!(""))
}

/// GET /api/_version_/fleet/scripts/results/{execution_id}
pub async fn get_script_result(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_execution_id): Path<String>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Script execution result retrieval requires async job infrastructure
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/scripts
pub async fn create_script(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // TODO: handle multipart upload of script content
    fleet_error(StatusCode::NOT_IMPLEMENTED, "multipart upload not yet implemented")
}

/// GET /api/_version_/fleet/scripts
pub async fn list_scripts(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListScriptsParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = params.team_id.map(|t| t as u32);
    match state.service.list_scripts(&viewer, team_id).await {
        Ok(scripts) => fleet_ok("scripts", serde_json::to_value(&scripts).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/scripts/{script_id}
pub async fn get_script(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(script_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_script(&viewer, script_id as u32).await {
        Ok(script) => fleet_ok("script", serde_json::to_value(&script).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/scripts/{script_id}
pub async fn update_script(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_script_id): Path<u64>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // TODO: handle multipart upload
    fleet_error(StatusCode::NOT_IMPLEMENTED, "multipart upload not yet implemented")
}

/// DELETE /api/_version_/fleet/scripts/{script_id}
pub async fn delete_script(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(script_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_script(&viewer, script_id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/scripts/batch
pub async fn batch_set_scripts(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(_body): Json<BatchSetScriptsBody>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/scripts/batch/{batch_execution_id}/cancel
pub async fn batch_script_cancel(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_batch_execution_id): Path<String>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/scripts/batch/summary/{batch_execution_id}
pub async fn batch_script_execution_summary(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_batch_execution_id): Path<String>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("summary", serde_json::json!({}))
}

/// GET /api/_version_/fleet/scripts/batch/{batch_execution_id}/host-results
pub async fn batch_script_execution_host_results(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_batch_execution_id): Path<String>,
    Query(_params): Query<BatchScriptExecutionHostResultsParams>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("host_results", serde_json::json!([]))
}

/// GET /api/_version_/fleet/scripts/batch/{batch_execution_id}
pub async fn batch_script_execution_status(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_batch_execution_id): Path<String>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("status", serde_json::json!({}))
}
