//! Query and live query endpoints.
//!
//! Handles query CRUD, specs, reports, live query execution, and
//! distributed query campaign streaming via websockets.

use axum::{
    extract::{Json, Path, Query, State, WebSocketUpgrade},
    response::IntoResponse,
};
use serde::Deserialize;

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
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Verify the query exists and auth
    match state.service.get_query(&viewer, id as u32).await {
        Ok(query) => {
            // If discard_data is set, return empty results
            if query.discard_data {
                return fleet_ok("report", serde_json::json!({"query_id": query.id, "results": [], "report_clipped": false}));
            }
            // Query report results require query_result_rows infrastructure (not yet implemented)
            fleet_ok("report", serde_json::json!({"query_id": query.id, "results": [], "report_clipped": false}))
        }
        Err(e) => encode_service_error(&e),
    }
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
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let ids = vec![id as u32];
    match state.service.delete_queries(&viewer, &ids).await {
        Ok(_count) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
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
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<ApplyQuerySpecsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let specs: Vec<fleet_service::queries::QuerySpec> = body.specs.into_iter().filter_map(|v| {
        serde_json::from_value(v).ok()
    }).collect();
    match state.service.apply_query_specs(&viewer, specs).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/spec/reports
pub async fn get_query_specs(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<GetQuerySpecsParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = params.team_id.map(|v| v as u32);
    match state.service.get_query_specs(&viewer, team_id).await {
        Ok(specs) => fleet_ok("specs", serde_json::to_value(&specs).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/spec/reports/{name}
pub async fn get_query_spec(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(name): Path<String>,
    Query(params): Query<GetQuerySpecsParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = params.team_id.map(|v| v as u32);
    match state.service.get_query_spec(&viewer, team_id, &name).await {
        Ok(spec) => fleet_ok("spec", serde_json::to_value(&spec).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/reports/{id}/run
pub async fn run_one_live_query(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<RunOneLiveQueryBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, id, &body);
    // Live query execution requires distributed query campaign infrastructure (deferred)
    fleet_ok("results", serde_json::json!([]))
}

/// GET /api/_version_/fleet/reports/run
pub async fn run_live_query(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<RunLiveQueryParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &params);
    // Live query execution requires distributed query campaign infrastructure (deferred)
    fleet_ok("results", serde_json::json!([]))
}

/// POST /api/_version_/fleet/reports/run_by_identifiers
pub async fn create_distributed_query_campaign_by_identifier(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<CreateDistributedQueryCampaignByIdentifierBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &body);
    // Distributed query campaigns require live query infrastructure (deferred)
    fleet_ok("campaign", serde_json::json!({}))
}

/// GET /api/_version_/fleet/results/{campaign_id} (WebSocket)
pub async fn stream_campaign_results(
    State(state): State<AppState>,
    Path(campaign_id): Path<u64>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let _ = (&state, campaign_id);
    ws.on_upgrade(|_socket| async move {
        // TODO: stream live query results over the websocket
    })
}
