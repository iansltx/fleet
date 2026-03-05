//! Query and live query endpoints.
//!
//! Handles query CRUD, specs, reports, live query execution, and
//! distributed query campaign streaming via websockets.

use axum::{
    extract::{Json, Path, Query, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::response::{fleet_error, fleet_ok, FleetResponse};

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListQueriesParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
    pub query: Option<String>,
    pub team_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateQueryBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub query: Option<String>,
    pub team_id: Option<u64>,
    pub interval: Option<u64>,
    pub platform: Option<String>,
    pub min_osquery_version: Option<String>,
    pub automations_enabled: Option<bool>,
    pub logging: Option<String>,
    pub observer_can_run: Option<bool>,
    pub discard_data: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ModifyQueryBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub query: Option<String>,
    pub team_id: Option<u64>,
    pub interval: Option<u64>,
    pub platform: Option<String>,
    pub min_osquery_version: Option<String>,
    pub automations_enabled: Option<bool>,
    pub logging: Option<String>,
    pub observer_can_run: Option<bool>,
    pub discard_data: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteQueriesBody {
    pub ids: Vec<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ApplyQuerySpecsBody {
    pub specs: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct RunLiveQueryParams {
    pub query: Option<String>,
    pub query_id: Option<u64>,
    pub host_ids: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RunOneLiveQueryBody {
    pub host_ids: Option<Vec<u64>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDistributedQueryCampaignBody {
    pub query: Option<String>,
    pub query_id: Option<u64>,
    pub selected: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDistributedQueryCampaignByIdentifierBody {
    pub query: Option<String>,
    pub query_id: Option<u64>,
    pub selected: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct GetQuerySpecsParams {
    pub team_id: Option<u64>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/reports/{id}
pub async fn get_query(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("query", serde_json::json!({}))
}

/// GET /api/_version_/fleet/reports
pub async fn list_queries(Query(_params): Query<ListQueriesParams>) -> FleetResponse {
    fleet_ok("queries", serde_json::json!([]))
}

/// GET /api/_version_/fleet/reports/{id}/report
pub async fn get_query_report(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("report", serde_json::json!({}))
}

/// POST /api/_version_/fleet/reports
pub async fn create_query(Json(_body): Json<CreateQueryBody>) -> FleetResponse {
    fleet_ok("query", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/reports/{id}
pub async fn modify_query(
    Path(_id): Path<u64>,
    Json(_body): Json<ModifyQueryBody>,
) -> FleetResponse {
    fleet_ok("query", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/reports/{name}
pub async fn delete_query(Path(_name): Path<String>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/reports/id/{id}
pub async fn delete_query_by_id(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/reports/delete
pub async fn delete_queries(Json(_body): Json<DeleteQueriesBody>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/spec/reports
pub async fn apply_query_specs(Json(_body): Json<ApplyQuerySpecsBody>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/spec/reports
pub async fn get_query_specs(Query(_params): Query<GetQuerySpecsParams>) -> FleetResponse {
    fleet_ok("specs", serde_json::json!([]))
}

/// GET /api/_version_/fleet/spec/reports/{name}
pub async fn get_query_spec(Path(_name): Path<String>) -> FleetResponse {
    fleet_ok("spec", serde_json::json!({}))
}

/// POST /api/_version_/fleet/reports/{id}/run
pub async fn run_one_live_query(
    Path(_id): Path<u64>,
    Json(_body): Json<RunOneLiveQueryBody>,
) -> FleetResponse {
    fleet_ok("results", serde_json::json!([]))
}

/// GET /api/_version_/fleet/reports/run
pub async fn run_live_query(Query(_params): Query<RunLiveQueryParams>) -> FleetResponse {
    fleet_ok("results", serde_json::json!([]))
}

/// POST /api/_version_/fleet/reports/run_by_identifiers
pub async fn create_distributed_query_campaign_by_identifier(
    Json(_body): Json<CreateDistributedQueryCampaignByIdentifierBody>,
) -> FleetResponse {
    fleet_ok("campaign", serde_json::json!({}))
}

/// GET /api/_version_/fleet/results/{campaign_id} (WebSocket)
pub async fn stream_campaign_results(
    Path(_campaign_id): Path<u64>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|mut socket| async move {
        // TODO: stream live query results over the websocket
        // The Go implementation authenticates the websocket session on connect
        // and then streams distributed query results.
    })
}
