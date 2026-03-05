//! Pack and scheduled query endpoints.
//!
//! Handles pack CRUD, pack specs, scheduled queries in packs,
//! global schedule, and team schedule management.

use axum::extract::{Json, Path, Query, State};
use serde::Deserialize;

use crate::middleware::auth::AuthenticatedUser;
use crate::response::{encode_service_error, fleet_error, fleet_ok, FleetResponse};
use crate::AppState;

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
pub async fn get_pack(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_pack(&viewer, id as u32).await {
        Ok(pack) => fleet_ok("pack", serde_json::to_value(&pack).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/packs
pub async fn create_pack(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<CreatePackBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_service::packs::PackPayload {
        name: body.name.unwrap_or_default(),
        description: body.description,
        host_ids: body.host_ids.map(|ids| ids.iter().map(|&id| id as u32).collect()),
        label_ids: body.label_ids.map(|ids| ids.iter().map(|&id| id as u32).collect()),
        team_ids: body.team_ids.map(|ids| ids.iter().map(|&id| id as u32).collect()),
        ..Default::default()
    };
    match state.service.new_pack(&viewer, payload).await {
        Ok(pack) => fleet_ok("pack", serde_json::to_value(&pack).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/packs/{id}
pub async fn modify_pack(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<ModifyPackBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_service::packs::ModifyPackPayload {
        name: body.name,
        description: body.description,
        host_ids: body.host_ids.map(|ids| ids.iter().map(|&id| id as u32).collect()),
        label_ids: body.label_ids.map(|ids| ids.iter().map(|&id| id as u32).collect()),
        team_ids: body.team_ids.map(|ids| ids.iter().map(|&id| id as u32).collect()),
        ..Default::default()
    };
    match state.service.modify_pack(&viewer, id as u32, payload).await {
        Ok(pack) => fleet_ok("pack", serde_json::to_value(&pack).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/packs
pub async fn list_packs(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListPacksParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let opts = fleet_types::ListOptions {
        page: params.page.unwrap_or(0) as u32,
        per_page: params.per_page.unwrap_or(0) as u32,
        order_key: params.order_key.unwrap_or_default(),
        ..Default::default()
    };
    match state.service.list_packs(&viewer, opts).await {
        Ok(packs) => fleet_ok("packs", serde_json::to_value(&packs).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/packs/{name}
pub async fn delete_pack(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(name): Path<String>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_pack(&viewer, &name).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/packs/id/{id}
pub async fn delete_pack_by_id(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_pack_by_id(&viewer, id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/spec/packs
pub async fn apply_pack_specs(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<ApplyPackSpecsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let specs: Vec<fleet_service::packs::PackSpec> = body
        .specs
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();
    match state.service.apply_pack_specs(&viewer, specs).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/spec/packs
pub async fn get_pack_specs(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_pack_specs(&viewer).await {
        Ok(specs) => fleet_ok("specs", serde_json::to_value(&specs).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/spec/packs/{name}
pub async fn get_pack_spec(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(name): Path<String>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_pack_spec(&viewer, &name).await {
        Ok(spec) => fleet_ok("spec", serde_json::to_value(&spec).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

// ---------------------------------------------------------------------------
// Scheduled query handlers
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/packs/{id}/scheduled
pub async fn get_scheduled_queries_in_pack(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_scheduled_queries_in_pack(&viewer, id as u32).await {
        Ok(scheduled) => fleet_ok("scheduled", serde_json::to_value(&scheduled).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/schedule  (v1)
/// POST /api/_version_/fleet/packs/schedule  (2022-04)
pub async fn schedule_query(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<ScheduleQueryBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let sq = fleet_types::ScheduledQuery {
        pack_id: body.pack_id.unwrap_or(0) as u32,
        query_id: body.query_id.unwrap_or(0) as u32,
        interval: body.interval.unwrap_or(0) as u32,
        snapshot: body.snapshot,
        removed: body.removed,
        platform: body.platform.unwrap_or_default(),
        shard: body.shard.map(|s| s as u32),
        version: body.version.unwrap_or_default(),
        ..Default::default()
    };
    match state.service.schedule_query(&viewer, sq).await {
        Ok(scheduled) => fleet_ok("scheduled", serde_json::to_value(&scheduled).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/schedule/{id}
pub async fn get_scheduled_query(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_scheduled_query(&viewer, id as u32).await {
        Ok(scheduled) => fleet_ok("scheduled", serde_json::to_value(&scheduled).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/schedule/{id}  (v1)
/// PATCH /api/_version_/fleet/packs/schedule/{id}  (2022-04)
pub async fn modify_scheduled_query(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<ModifyScheduledQueryBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let sq = fleet_types::ScheduledQuery {
        id: id as u32,
        interval: body.interval.unwrap_or(0) as u32,
        snapshot: body.snapshot,
        removed: body.removed,
        platform: body.platform.unwrap_or_default(),
        shard: body.shard.map(|s| s as u32),
        version: body.version.unwrap_or_default(),
        ..Default::default()
    };
    match state.service.modify_scheduled_query(&viewer, sq).await {
        Ok(scheduled) => fleet_ok("scheduled", serde_json::to_value(&scheduled).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/schedule/{id}  (v1)
/// DELETE /api/_version_/fleet/packs/schedule/{id}  (2022-04)
pub async fn delete_scheduled_query(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_scheduled_query(&viewer, id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

// ---------------------------------------------------------------------------
// Global schedule handlers
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/global/schedule  (v1)
/// GET /api/_version_/fleet/schedule  (2022-04)
pub async fn get_global_schedule(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(_params): Query<GetGlobalScheduleParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Global schedule maps to the global pack; return empty for now.
    let _ = &viewer;
    fleet_ok("global_schedule", serde_json::json!([]))
}

/// POST /api/_version_/fleet/global/schedule  (v1)
/// POST /api/_version_/fleet/schedule  (2022-04)
pub async fn global_schedule_query(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<GlobalScheduleQueryBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let sq = fleet_types::ScheduledQuery {
        query_id: body.query_id.unwrap_or(0) as u32,
        interval: body.interval.unwrap_or(0) as u32,
        snapshot: body.snapshot,
        removed: body.removed,
        platform: body.platform.unwrap_or_default(),
        shard: body.shard.map(|s| s as u32),
        version: body.version.unwrap_or_default(),
        ..Default::default()
    };
    match state.service.schedule_query(&viewer, sq).await {
        Ok(scheduled) => fleet_ok("scheduled", serde_json::to_value(&scheduled).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/global/schedule/{id}  (v1)
/// PATCH /api/_version_/fleet/schedule/{id}  (2022-04)
pub async fn modify_global_schedule(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<ModifyScheduledQueryBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let sq = fleet_types::ScheduledQuery {
        id: id as u32,
        interval: body.interval.unwrap_or(0) as u32,
        snapshot: body.snapshot,
        removed: body.removed,
        platform: body.platform.unwrap_or_default(),
        shard: body.shard.map(|s| s as u32),
        version: body.version.unwrap_or_default(),
        ..Default::default()
    };
    match state.service.modify_scheduled_query(&viewer, sq).await {
        Ok(scheduled) => fleet_ok("scheduled", serde_json::to_value(&scheduled).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/global/schedule/{id}  (v1)
/// DELETE /api/_version_/fleet/schedule/{id}  (2022-04)
pub async fn delete_global_schedule(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_scheduled_query(&viewer, id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

// ---------------------------------------------------------------------------
// Team schedule handlers
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/fleets/{fleet_id}/schedule
pub async fn get_team_schedule(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_fleet_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Team schedule maps to a team-specific pack; return empty for now.
    let _ = &viewer;
    fleet_ok("scheduled", serde_json::json!([]))
}

/// POST /api/_version_/fleet/fleets/{fleet_id}/schedule
pub async fn team_schedule_query(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_fleet_id): Path<u64>,
    Json(body): Json<TeamScheduleQueryBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let sq = fleet_types::ScheduledQuery {
        query_id: body.query_id.unwrap_or(0) as u32,
        interval: body.interval.unwrap_or(0) as u32,
        snapshot: body.snapshot,
        removed: body.removed,
        platform: body.platform.unwrap_or_default(),
        shard: body.shard.map(|s| s as u32),
        version: body.version.unwrap_or_default(),
        ..Default::default()
    };
    match state.service.schedule_query(&viewer, sq).await {
        Ok(scheduled) => fleet_ok("scheduled", serde_json::to_value(&scheduled).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/fleets/{fleet_id}/schedule/{report_id}
pub async fn modify_team_schedule(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path((_fleet_id, report_id)): Path<(u64, u64)>,
    Json(body): Json<ModifyScheduledQueryBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let sq = fleet_types::ScheduledQuery {
        id: report_id as u32,
        interval: body.interval.unwrap_or(0) as u32,
        snapshot: body.snapshot,
        removed: body.removed,
        platform: body.platform.unwrap_or_default(),
        shard: body.shard.map(|s| s as u32),
        version: body.version.unwrap_or_default(),
        ..Default::default()
    };
    match state.service.modify_scheduled_query(&viewer, sq).await {
        Ok(scheduled) => fleet_ok("scheduled", serde_json::to_value(&scheduled).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/fleets/{fleet_id}/schedule/{report_id}
pub async fn delete_team_schedule(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path((_fleet_id, report_id)): Path<(u64, u64)>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_scheduled_query(&viewer, report_id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}
