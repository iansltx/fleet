//! Label management endpoints.
//!
//! Handles label CRUD, specs, and listing hosts within labels.

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
pub struct ListLabelsParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLabelBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub query: Option<String>,
    pub platform: Option<String>,
    pub label_membership_type: Option<String>,
    pub hosts: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ModifyLabelBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub hosts: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ApplyLabelSpecsBody {
    pub specs: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListHostsInLabelParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
    pub status: Option<String>,
    pub query: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/labels
pub async fn create_label(Json(_body): Json<CreateLabelBody>) -> FleetResponse {
    fleet_ok("label", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/labels/{id}
pub async fn modify_label(
    Path(_id): Path<u64>,
    Json(_body): Json<ModifyLabelBody>,
) -> FleetResponse {
    fleet_ok("label", serde_json::json!({}))
}

/// GET /api/_version_/fleet/labels/{id}
pub async fn get_label(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("label", serde_json::json!({}))
}

/// GET /api/_version_/fleet/labels
pub async fn list_labels(Query(_params): Query<ListLabelsParams>) -> FleetResponse {
    fleet_ok("labels", serde_json::json!([]))
}

/// GET /api/_version_/fleet/labels/summary
pub async fn get_labels_summary() -> FleetResponse {
    fleet_ok("labels", serde_json::json!([]))
}

/// GET /api/_version_/fleet/labels/{id}/hosts
pub async fn list_hosts_in_label(
    Path(_id): Path<u64>,
    Query(_params): Query<ListHostsInLabelParams>,
) -> FleetResponse {
    fleet_ok("hosts", serde_json::json!([]))
}

/// DELETE /api/_version_/fleet/labels/{name}
pub async fn delete_label(Path(_name): Path<String>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/labels/id/{id}
pub async fn delete_label_by_id(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/spec/labels
pub async fn apply_label_specs(Json(_body): Json<ApplyLabelSpecsBody>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/spec/labels
pub async fn get_label_specs() -> FleetResponse {
    fleet_ok("specs", serde_json::json!([]))
}

/// GET /api/_version_/fleet/spec/labels/{name}
pub async fn get_label_spec(Path(_name): Path<String>) -> FleetResponse {
    fleet_ok("spec", serde_json::json!({}))
}
