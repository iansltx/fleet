//! Team management endpoints.
//!
//! Handles team CRUD, specs, agent options, team users, and enroll secrets.

use axum::extract::{Json, Path, Query, State};
use serde::Deserialize;

use crate::middleware::auth::AuthenticatedUser;
use crate::response::{encode_service_error, fleet_error, fleet_ok, FleetResponse};
use crate::AppState;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListTeamsParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTeamBody {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ModifyTeamBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub webhook_settings: Option<serde_json::Value>,
    pub integrations: Option<serde_json::Value>,
    pub mdm: Option<serde_json::Value>,
    pub host_expiry_settings: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ApplyTeamSpecsBody {
    pub specs: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ModifyTeamAgentOptionsBody {
    pub agent_options: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ModifyTeamUsersBody {
    pub users: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ModifyTeamEnrollSecretsBody {
    pub secrets: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListTeamUsersParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
    pub query: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/spec/fleets
pub async fn apply_team_specs(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<ApplyTeamSpecsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let specs: Vec<fleet_service::teams::TeamSpec> = body
        .specs
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();
    match state.service.apply_team_specs(&viewer, specs).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/fleets/{fleet_id}/secrets
pub async fn modify_team_enroll_secrets(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(fleet_id): Path<u64>,
    Json(body): Json<ModifyTeamEnrollSecretsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Extract secret strings from the JSON values
    let secrets: Vec<String> = body.secrets.iter().filter_map(|v| {
        v.get("secret").and_then(|s| s.as_str()).map(|s| s.to_string())
    }).collect();
    match state.service.modify_team_enroll_secrets(&viewer, fleet_id as u32, secrets).await {
        Ok(result) => fleet_ok("secrets", serde_json::to_value(&result).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/fleets
pub async fn create_team(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<CreateTeamBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_types::team::TeamPayload {
        name: Some(body.name),
        description: body.description,
        ..Default::default()
    };
    match state.service.new_team(&viewer, payload).await {
        Ok(team) => fleet_ok("team", serde_json::to_value(&team).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/fleets
pub async fn list_teams(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListTeamsParams>,
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
    match state.service.list_teams(&viewer, opts).await {
        Ok(teams) => fleet_ok("teams", serde_json::to_value(&teams).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/fleets/{id}
pub async fn get_team(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_team(&viewer, id as u32).await {
        Ok(team) => fleet_ok("team", serde_json::to_value(&team).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/fleets/{id}
pub async fn modify_team(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<ModifyTeamBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_types::team::TeamPayload {
        name: body.name,
        description: body.description,
        ..Default::default()
    };
    match state.service.modify_team(&viewer, id as u32, payload).await {
        Ok(team) => fleet_ok("team", serde_json::to_value(&team).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/fleets/{id}
pub async fn delete_team(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_team(&viewer, id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/fleets/{id}/agent_options
pub async fn modify_team_agent_options(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<ModifyTeamAgentOptionsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.modify_team_agent_options(&viewer, id as u32, body.agent_options).await {
        Ok(team) => fleet_ok("team", serde_json::to_value(&team).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/fleets/{id}/users
pub async fn list_team_users(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Query(params): Query<ListTeamUsersParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &params;
    match state.service.list_team_users(&viewer, id as u32).await {
        Ok(users) => fleet_ok("users", serde_json::to_value(&users).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/fleets/{id}/users
pub async fn add_team_users(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<ModifyTeamUsersBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let users: Vec<(u32, String)> = body.users.iter().filter_map(|v| {
        let id = v.get("id")?.as_u64()? as u32;
        let role = v.get("role").and_then(|r| r.as_str()).unwrap_or("observer").to_string();
        Some((id, role))
    }).collect();
    match state.service.add_team_users(&viewer, id as u32, &users).await {
        Ok(team) => fleet_ok("team", serde_json::to_value(&team).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/fleets/{id}/users
pub async fn delete_team_users(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<ModifyTeamUsersBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let user_ids: Vec<u32> = body.users.iter().filter_map(|v| {
        v.get("id")?.as_u64().map(|id| id as u32)
    }).collect();
    match state.service.remove_team_users(&viewer, id as u32, &user_ids).await {
        Ok(team) => fleet_ok("team", serde_json::to_value(&team).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/fleets/{id}/secrets
pub async fn team_enroll_secrets(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.team_enroll_secrets(&viewer, id as u32).await {
        Ok(secrets) => fleet_ok("secrets", serde_json::to_value(&secrets).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}
