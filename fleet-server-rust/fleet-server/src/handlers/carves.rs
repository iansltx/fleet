//! File carving endpoints.
//!
//! Handles carve listing, block retrieval, and the osquery carve protocol.

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
pub struct ListCarvesParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
    pub expired: Option<bool>,
}

/// Osquery carve begin request.
#[derive(Debug, Deserialize)]
pub struct CarveBeginBody {
    pub node_key: String,
    pub session_id: String,
    pub request_id: String,
    pub carve_size: u64,
    pub block_size: u64,
    pub carve_id: String,
}

/// Osquery carve block submission (unauthenticated, uses session ID).
#[derive(Debug, Deserialize)]
pub struct CarveBlockBody {
    pub session_id: String,
    pub request_id: String,
    pub block_id: u64,
    pub data: String, // base64-encoded block data
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/carves
pub async fn list_carves(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListCarvesParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let include_expired = params.expired.unwrap_or(false);
    match state.service.list_carves(&viewer, include_expired).await {
        Ok(carves) => fleet_ok("carves", serde_json::to_value(&carves).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/carves/{id}
pub async fn get_carve(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_carve(&viewer, id as i64).await {
        Ok(carve) => fleet_ok("carve", serde_json::to_value(&carve).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/carves/{id}/block/{block_id}
pub async fn get_carve_block(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path((id, block_id)): Path<(u64, u64)>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_carve_block(&viewer, id as i64, block_id as i64).await {
        Ok(data) => {
            let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
            fleet_ok("data", serde_json::json!(encoded))
        }
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/osquery/carve/begin
/// POST /api/v1/osquery/carve/begin
pub async fn carve_begin(
    State(state): State<AppState>,
    Json(body): Json<CarveBeginBody>,
) -> FleetResponse {
    // Authenticate by node_key
    let (host, _debug) = match state.service.authenticate_host(&body.node_key).await {
        Ok(h) => h,
        Err(e) => return encode_service_error(&e),
    };

    let block_size = body.block_size as i64;
    let carve_size = body.carve_size as i64;
    let block_count = if block_size > 0 {
        (carve_size + block_size - 1) / block_size
    } else {
        0
    };

    let payload = fleet_types::CarveBeginPayload {
        block_count,
        block_size,
        carve_size,
        carve_id: body.carve_id,
        request_id: body.request_id,
    };

    match state.service.carve_begin(&host, payload).await {
        Ok(carve) => fleet_ok("session_id", serde_json::json!(carve.session_id)),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/osquery/carve/block
/// POST /api/v1/osquery/carve/block
pub async fn carve_block(
    State(state): State<AppState>,
    Json(body): Json<CarveBlockBody>,
) -> FleetResponse {
    // Decode base64 data
    let data = match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &body.data) {
        Ok(d) => d,
        Err(e) => return fleet_error(StatusCode::BAD_REQUEST, &format!("invalid base64 data: {}", e)),
    };

    let payload = fleet_types::CarveBlockPayload {
        session_id: body.session_id,
        request_id: body.request_id,
        block_id: body.block_id as i64,
        data,
    };

    match state.service.carve_block(payload).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}
