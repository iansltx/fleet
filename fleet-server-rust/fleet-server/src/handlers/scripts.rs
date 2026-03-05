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
    Json(body): Json<RunScriptBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };

    // Resolve script contents: either from script_id or inline script_contents
    let (script_contents, _script_id) = if let Some(script_id) = body.script_id {
        match state.service.get_script_contents(&viewer, script_id as u32).await {
            Ok(contents) => (contents, Some(script_id as u32)),
            Err(e) => return encode_service_error(&e),
        }
    } else if let Some(ref contents) = body.script_contents {
        (contents.clone(), None)
    } else {
        return fleet_error(StatusCode::BAD_REQUEST, "either script_id or script_contents is required");
    };

    // Create a host_script_result record for tracking
    let execution_id = uuid::Uuid::new_v4().to_string();
    let result = fleet_types::script::HostScriptResult {
        id: 0,
        host_id: body.host_id as u32,
        execution_id: execution_id.clone(),
        script_id: body.script_id.map(|id| id as u32),
        script_contents,
        output: String::new(),
        runtime: 0,
        exit_code: None,
        message: None,
        host_timeout: false,
        host_deleted_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    if let Err(e) = state.service.save_host_script_result(&viewer, &result).await {
        return encode_service_error(&e);
    }

    fleet_ok("execution_id", serde_json::json!(execution_id))
}

/// POST /api/_version_/fleet/scripts/run/sync
pub async fn run_script_sync(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<RunScriptSyncBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Resolve script contents
    let script_contents = if let Some(ref contents) = body.script_contents {
        contents.clone()
    } else if let Some(script_id) = body.script_id {
        match state.service.get_script_contents(&viewer, script_id as u32).await {
            Ok(contents) => contents,
            Err(e) => return encode_service_error(&e),
        }
    } else {
        return fleet_error(StatusCode::BAD_REQUEST, "script_id or script_contents is required");
    };

    let execution_id = uuid::Uuid::new_v4().to_string();
    let host_id = body.host_id as u32;

    // Create a sync execution request
    if let Err(e) = state.service.new_host_script_execution_request(
        &viewer, host_id, body.script_id.map(|id| id as u32), &script_contents, &execution_id, true,
    ).await {
        return encode_service_error(&e);
    }

    // Return immediately with execution_id; full sync wait requires live query infrastructure
    fleet_ok("", serde_json::json!({
        "host_id": host_id,
        "execution_id": execution_id,
        "script_contents": script_contents,
        "exit_code": null,
        "output": "",
        "message": "Script execution queued. Poll for results.",
        "runtime": 0,
        "host_timeout": false
    }))
}

/// POST /api/_version_/fleet/scripts/run/batch
pub async fn batch_script_run(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<BatchScriptRunBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let host_ids: Vec<u32> = body.host_ids.iter().map(|&id| id as u32).collect();
    match state.service.batch_run_script(&viewer, &host_ids, body.script_id as u32).await {
        Ok(batch_execution_id) => fleet_ok("batch_execution_id", serde_json::json!(batch_execution_id)),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/scripts/results/{execution_id}
pub async fn get_script_result(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(execution_id): Path<String>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_script_result(&viewer, &execution_id).await {
        Ok(result) => fleet_ok("", serde_json::to_value(&result).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/scripts
pub async fn create_script(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    mut multipart: axum::extract::Multipart,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };

    let mut team_id: Option<u32> = None;
    let mut script_name: Option<String> = None;
    let mut script_contents: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "team_id" => {
                if let Ok(text) = field.text().await {
                    team_id = text.parse::<u32>().ok();
                }
            }
            "script" => {
                // The file name becomes the script name
                if let Some(file_name) = field.file_name().map(|s| s.to_string()) {
                    script_name = Some(file_name);
                }
                match field.text().await {
                    Ok(text) => script_contents = Some(text),
                    Err(e) => return fleet_error(StatusCode::BAD_REQUEST, &format!("failed to read script contents: {}", e)),
                }
            }
            _ => {
                // Skip unknown fields
            }
        }
    }

    let name = match script_name {
        Some(n) => n,
        None => return fleet_error(StatusCode::BAD_REQUEST, "script file is required"),
    };
    let contents = match script_contents {
        Some(c) => c,
        None => return fleet_error(StatusCode::BAD_REQUEST, "script contents are required"),
    };

    match state.service.create_script(&viewer, team_id, &name, &contents).await {
        Ok(script) => fleet_ok("script", serde_json::to_value(&script).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
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

/// GET /api/_version_/fleet/scripts/{script_id}/content
pub async fn get_script_content(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(script_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_script_contents(&viewer, script_id as u32).await {
        Ok(contents) => fleet_ok("script_contents", serde_json::json!(contents)),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/scripts/{script_id}
pub async fn update_script(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(script_id): Path<u64>,
    mut multipart: axum::extract::Multipart,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };

    let mut script_contents: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "script" {
            match field.text().await {
                Ok(text) => script_contents = Some(text),
                Err(e) => return fleet_error(StatusCode::BAD_REQUEST, &format!("failed to read script contents: {}", e)),
            }
        }
    }

    let contents = match script_contents {
        Some(c) => c,
        None => return fleet_error(StatusCode::BAD_REQUEST, "script file is required"),
    };

    // Get existing script to verify it exists and get its name
    let existing = match state.service.get_script(&viewer, script_id as u32).await {
        Ok(s) => s,
        Err(e) => return encode_service_error(&e),
    };

    // Delete and recreate (scripts are immutable content-wise in Fleet)
    if let Err(e) = state.service.delete_script(&viewer, script_id as u32).await {
        return encode_service_error(&e);
    }
    match state.service.create_script(&viewer, existing.team_id, &existing.name, &contents).await {
        Ok(script) => fleet_ok("script", serde_json::to_value(&script).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
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
    Json(body): Json<BatchSetScriptsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Batch set scripts: for each script in the payload, create or update
    if body.dry_run.unwrap_or(false) {
        return fleet_ok("", serde_json::json!({}));
    }
    let team_id = body.team_id.map(|t| t as u32);
    // Process each script spec
    for script_val in &body.scripts {
        let name = script_val.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let contents = script_val.get("script_contents").and_then(|v| v.as_str()).unwrap_or("");
        if !name.is_empty() && !contents.is_empty() {
            // Best-effort: create script (ignore duplicates)
            let _ = state.service.create_script(&viewer, team_id, name, contents).await;
        }
    }
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/scripts/batch/{batch_execution_id}/cancel
pub async fn batch_script_cancel(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(batch_execution_id): Path<String>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.cancel_batch_script_execution(&viewer, &batch_execution_id).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/scripts/batch/summary/{batch_execution_id}
pub async fn batch_script_execution_summary(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(batch_execution_id): Path<String>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_batch_script_execution_summary(&viewer, &batch_execution_id).await {
        Ok(summary) => fleet_ok("summary", summary),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/scripts/batch/{batch_execution_id}/host-results
pub async fn batch_script_execution_host_results(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(batch_execution_id): Path<String>,
    Query(params): Query<BatchScriptExecutionHostResultsParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let limit = params.per_page.unwrap_or(20) as u32;
    let offset = params.page.unwrap_or(0) as u32 * limit;
    match state.service.list_batch_script_execution_hosts(&viewer, &batch_execution_id, limit, offset).await {
        Ok(results) => fleet_ok("host_results", serde_json::to_value(&results).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/scripts/batch/{batch_execution_id}
pub async fn batch_script_execution_status(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(batch_execution_id): Path<String>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_batch_script_execution_summary(&viewer, &batch_execution_id).await {
        Ok(status) => fleet_ok("status", status),
        Err(e) => encode_service_error(&e),
    }
}
