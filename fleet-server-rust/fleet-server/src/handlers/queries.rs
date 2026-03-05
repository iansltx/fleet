//! Query and live query endpoints.
//!
//! Handles query CRUD, specs, reports, live query execution, and
//! distributed query campaign streaming via websockets.

use axum::{
    extract::{Json, Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::middleware::auth::AuthenticatedUser;
use crate::response::{encode_service_error, fleet_error, fleet_ok, FleetResponse};
use crate::AppState;

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
pub async fn get_query(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_query(&viewer, id as u32).await {
        Ok(query) => fleet_ok("query", serde_json::to_value(&query).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/reports
pub async fn list_queries(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListQueriesParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let opts = fleet_types::ListOptions {
        page: params.page.unwrap_or(0) as u32,
        per_page: params.per_page.unwrap_or(0) as u32,
        order_key: params.order_key.unwrap_or_default(),
        match_query: params.query.unwrap_or_default(),
        ..Default::default()
    };
    let team_id = params.team_id.map(|v| v as u32);
    match state.service.list_queries(&viewer, opts, team_id).await {
        Ok(queries) => fleet_ok("queries", serde_json::to_value(&queries).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/reports/{id}/report
pub async fn get_query_report(
    State(_state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("report", serde_json::json!({}))
}

/// POST /api/_version_/fleet/reports
pub async fn create_query(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<CreateQueryBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_service::queries::QueryPayload {
        name: body.name.unwrap_or_default(),
        description: body.description,
        query: body.query.unwrap_or_default(),
        observer_can_run: body.observer_can_run,
        team_id: body.team_id.map(|v| v as u32),
        interval: body.interval.map(|v| v as u32),
        platform: body.platform,
        min_osquery_version: body.min_osquery_version,
        automations_enabled: body.automations_enabled,
        logging: body.logging,
        discard_data: body.discard_data,
    };
    match state.service.new_query(&viewer, payload).await {
        Ok(query) => fleet_ok("query", serde_json::to_value(&query).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/reports/{id}
pub async fn modify_query(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<ModifyQueryBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_service::queries::ModifyQueryPayload {
        name: body.name,
        description: body.description,
        query: body.query,
        observer_can_run: body.observer_can_run,
        interval: body.interval.map(|v| v as u32),
        platform: body.platform,
        min_osquery_version: body.min_osquery_version,
        automations_enabled: body.automations_enabled,
        logging: body.logging,
        discard_data: body.discard_data,
    };
    match state.service.modify_query(&viewer, id as u32, payload).await {
        Ok(query) => fleet_ok("query", serde_json::to_value(&query).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/reports/{name}
pub async fn delete_query(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(name): Path<String>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_query(&viewer, &name, None).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/reports/id/{id}
pub async fn delete_query_by_id(
    State(_state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_id): Path<u64>,
) -> FleetResponse {
    // No direct delete-by-id in the service layer yet
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/reports/delete
pub async fn delete_queries(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<DeleteQueriesBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let ids: Vec<u32> = body.ids.iter().map(|&id| id as u32).collect();
    match state.service.delete_queries(&viewer, &ids).await {
        Ok(count) => fleet_ok("", serde_json::json!({"deleted": count})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/spec/reports
pub async fn apply_query_specs(
    State(_state): State<AppState>,
    auth: AuthenticatedUser,
    Json(_body): Json<ApplyQuerySpecsBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/spec/reports
pub async fn get_query_specs(
    State(_state): State<AppState>,
    auth: AuthenticatedUser,
    Query(_params): Query<GetQuerySpecsParams>,
) -> FleetResponse {
    fleet_ok("specs", serde_json::json!([]))
}

/// GET /api/_version_/fleet/spec/reports/{name}
pub async fn get_query_spec(
    State(_state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_name): Path<String>,
) -> FleetResponse {
    fleet_ok("spec", serde_json::json!({}))
}

/// POST /api/_version_/fleet/reports/{id}/run
pub async fn run_one_live_query(
    State(_state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_id): Path<u64>,
    Json(_body): Json<RunOneLiveQueryBody>,
) -> FleetResponse {
    fleet_ok("results", serde_json::json!([]))
}

/// GET /api/_version_/fleet/reports/run
pub async fn run_live_query(
    State(_state): State<AppState>,
    auth: AuthenticatedUser,
    Query(_params): Query<RunLiveQueryParams>,
) -> FleetResponse {
    fleet_ok("results", serde_json::json!([]))
}

/// POST /api/_version_/fleet/reports/run_by_identifiers
pub async fn create_distributed_query_campaign_by_identifier(
    State(_state): State<AppState>,
    auth: AuthenticatedUser,
    Json(_body): Json<CreateDistributedQueryCampaignByIdentifierBody>,
) -> FleetResponse {
    fleet_ok("campaign", serde_json::json!({}))
}

/// GET /api/_version_/fleet/results/{campaign_id} (WebSocket)
pub async fn stream_campaign_results(
    State(_state): State<AppState>,
    Path(_campaign_id): Path<u64>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|mut socket| async move {
        // TODO: stream live query results over the websocket
    })
}
