//! File carving endpoints.
//!
//! Handles carve listing, block retrieval, and the osquery carve protocol.

use axum::{
    extract::{Json, Path, Query},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::response::{fleet_error, fleet_ok, FleetResponse};

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
pub async fn list_carves(Query(_params): Query<ListCarvesParams>) -> FleetResponse {
    fleet_ok("carves", serde_json::json!([]))
}

/// GET /api/_version_/fleet/carves/{id}
pub async fn get_carve(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("carve", serde_json::json!({}))
}

/// GET /api/_version_/fleet/carves/{id}/block/{block_id}
pub async fn get_carve_block(Path((_id, _block_id)): Path<(u64, u64)>) -> FleetResponse {
    fleet_ok("data", serde_json::json!(""))
}

/// POST /api/osquery/carve/begin
/// POST /api/v1/osquery/carve/begin
pub async fn carve_begin(Json(_body): Json<CarveBeginBody>) -> FleetResponse {
    fleet_ok("session_id", serde_json::json!(""))
}

/// POST /api/osquery/carve/block
/// POST /api/v1/osquery/carve/block
pub async fn carve_block(Json(_body): Json<CarveBlockBody>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}
