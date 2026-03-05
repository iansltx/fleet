//! Pack and scheduled query endpoints.
//!
//! Handles pack CRUD, pack specs, scheduled queries in packs,
//! global schedule, and team schedule management.

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
pub struct ListPacksParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePackBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub host_ids: Option<Vec<u64>>,
    pub label_ids: Option<Vec<u64>>,
    pub team_ids: Option<Vec<u64>>,
}

#[derive(Debug, Deserialize)]
pub struct ModifyPackBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub host_ids: Option<Vec<u64>>,
    pub label_ids: Option<Vec<u64>>,
    pub team_ids: Option<Vec<u64>>,
}

#[derive(Debug, Deserialize)]
pub struct ApplyPackSpecsBody {
    pub specs: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ScheduleQueryBody {
    pub pack_id: Option<u64>,
    pub query_id: Option<u64>,
    pub interval: Option<u64>,
    pub snapshot: Option<bool>,
    pub removed: Option<bool>,
    pub platform: Option<String>,
    pub shard: Option<u64>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ModifyScheduledQueryBody {
    pub interval: Option<u64>,
    pub snapshot: Option<bool>,
    pub removed: Option<bool>,
    pub platform: Option<String>,
    pub shard: Option<u64>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetGlobalScheduleParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GlobalScheduleQueryBody {
    pub query_id: Option<u64>,
    pub interval: Option<u64>,
    pub snapshot: Option<bool>,
    pub removed: Option<bool>,
    pub platform: Option<String>,
    pub shard: Option<u64>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TeamScheduleQueryBody {
    pub query_id: Option<u64>,
    pub interval: Option<u64>,
    pub snapshot: Option<bool>,
    pub removed: Option<bool>,
    pub platform: Option<String>,
    pub shard: Option<u64>,
    pub version: Option<String>,
}

// ---------------------------------------------------------------------------
// Pack handlers
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/packs/{id}
pub async fn get_pack(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("pack", serde_json::json!({}))
}

/// POST /api/_version_/fleet/packs
pub async fn create_pack(Json(_body): Json<CreatePackBody>) -> FleetResponse {
    fleet_ok("pack", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/packs/{id}
pub async fn modify_pack(
    Path(_id): Path<u64>,
    Json(_body): Json<ModifyPackBody>,
) -> FleetResponse {
    fleet_ok("pack", serde_json::json!({}))
}

/// GET /api/_version_/fleet/packs
pub async fn list_packs(Query(_params): Query<ListPacksParams>) -> FleetResponse {
    fleet_ok("packs", serde_json::json!([]))
}

/// DELETE /api/_version_/fleet/packs/{name}
pub async fn delete_pack(Path(_name): Path<String>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/packs/id/{id}
pub async fn delete_pack_by_id(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/spec/packs
pub async fn apply_pack_specs(Json(_body): Json<ApplyPackSpecsBody>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/spec/packs
pub async fn get_pack_specs() -> FleetResponse {
    fleet_ok("specs", serde_json::json!([]))
}

/// GET /api/_version_/fleet/spec/packs/{name}
pub async fn get_pack_spec(Path(_name): Path<String>) -> FleetResponse {
    fleet_ok("spec", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Scheduled query handlers
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/packs/{id}/scheduled
pub async fn get_scheduled_queries_in_pack(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("scheduled", serde_json::json!([]))
}

/// POST /api/_version_/fleet/schedule  (v1)
/// POST /api/_version_/fleet/packs/schedule  (2022-04)
pub async fn schedule_query(Json(_body): Json<ScheduleQueryBody>) -> FleetResponse {
    fleet_ok("scheduled", serde_json::json!({}))
}

/// GET /api/_version_/fleet/schedule/{id}
pub async fn get_scheduled_query(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("scheduled", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/schedule/{id}  (v1)
/// PATCH /api/_version_/fleet/packs/schedule/{id}  (2022-04)
pub async fn modify_scheduled_query(
    Path(_id): Path<u64>,
    Json(_body): Json<ModifyScheduledQueryBody>,
) -> FleetResponse {
    fleet_ok("scheduled", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/schedule/{id}  (v1)
/// DELETE /api/_version_/fleet/packs/schedule/{id}  (2022-04)
pub async fn delete_scheduled_query(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Global schedule handlers
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/global/schedule  (v1)
/// GET /api/_version_/fleet/schedule  (2022-04)
pub async fn get_global_schedule(
    Query(_params): Query<GetGlobalScheduleParams>,
) -> FleetResponse {
    fleet_ok("global_schedule", serde_json::json!([]))
}

/// POST /api/_version_/fleet/global/schedule  (v1)
/// POST /api/_version_/fleet/schedule  (2022-04)
pub async fn global_schedule_query(Json(_body): Json<GlobalScheduleQueryBody>) -> FleetResponse {
    fleet_ok("scheduled", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/global/schedule/{id}  (v1)
/// PATCH /api/_version_/fleet/schedule/{id}  (2022-04)
pub async fn modify_global_schedule(
    Path(_id): Path<u64>,
    Json(_body): Json<ModifyScheduledQueryBody>,
) -> FleetResponse {
    fleet_ok("scheduled", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/global/schedule/{id}  (v1)
/// DELETE /api/_version_/fleet/schedule/{id}  (2022-04)
pub async fn delete_global_schedule(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Team schedule handlers
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/fleets/{fleet_id}/schedule
pub async fn get_team_schedule(Path(_fleet_id): Path<u64>) -> FleetResponse {
    fleet_ok("scheduled", serde_json::json!([]))
}

/// POST /api/_version_/fleet/fleets/{fleet_id}/schedule
pub async fn team_schedule_query(
    Path(_fleet_id): Path<u64>,
    Json(_body): Json<TeamScheduleQueryBody>,
) -> FleetResponse {
    fleet_ok("scheduled", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/fleets/{fleet_id}/schedule/{report_id}
pub async fn modify_team_schedule(
    Path((_fleet_id, _report_id)): Path<(u64, u64)>,
    Json(_body): Json<ModifyScheduledQueryBody>,
) -> FleetResponse {
    fleet_ok("scheduled", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/fleets/{fleet_id}/schedule/{report_id}
pub async fn delete_team_schedule(
    Path((_fleet_id, _report_id)): Path<(u64, u64)>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}
