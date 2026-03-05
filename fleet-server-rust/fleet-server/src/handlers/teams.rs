//! Team management endpoints.
//!
//! Handles team CRUD, specs, agent options, team users, and enroll secrets.

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
pub async fn apply_team_specs(Json(_body): Json<ApplyTeamSpecsBody>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/fleets/{fleet_id}/secrets
pub async fn modify_team_enroll_secrets(
    Path(_fleet_id): Path<u64>,
    Json(_body): Json<ModifyTeamEnrollSecretsBody>,
) -> FleetResponse {
    fleet_ok("secrets", serde_json::json!([]))
}

/// POST /api/_version_/fleet/fleets
pub async fn create_team(Json(_body): Json<CreateTeamBody>) -> FleetResponse {
    fleet_ok("team", serde_json::json!({}))
}

/// GET /api/_version_/fleet/fleets
pub async fn list_teams(Query(_params): Query<ListTeamsParams>) -> FleetResponse {
    fleet_ok("teams", serde_json::json!([]))
}

/// GET /api/_version_/fleet/fleets/{id}
pub async fn get_team(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("team", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/fleets/{id}
pub async fn modify_team(
    Path(_id): Path<u64>,
    Json(_body): Json<ModifyTeamBody>,
) -> FleetResponse {
    fleet_ok("team", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/fleets/{id}
pub async fn delete_team(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/fleets/{id}/agent_options
pub async fn modify_team_agent_options(
    Path(_id): Path<u64>,
    Json(_body): Json<ModifyTeamAgentOptionsBody>,
) -> FleetResponse {
    fleet_ok("team", serde_json::json!({}))
}

/// GET /api/_version_/fleet/fleets/{id}/users
pub async fn list_team_users(
    Path(_id): Path<u64>,
    Query(_params): Query<ListTeamUsersParams>,
) -> FleetResponse {
    fleet_ok("users", serde_json::json!([]))
}

/// PATCH /api/_version_/fleet/fleets/{id}/users
pub async fn add_team_users(
    Path(_id): Path<u64>,
    Json(_body): Json<ModifyTeamUsersBody>,
) -> FleetResponse {
    fleet_ok("team", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/fleets/{id}/users
pub async fn delete_team_users(
    Path(_id): Path<u64>,
    Json(_body): Json<ModifyTeamUsersBody>,
) -> FleetResponse {
    fleet_ok("team", serde_json::json!({}))
}

/// GET /api/_version_/fleet/fleets/{id}/secrets
pub async fn team_enroll_secrets(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("secrets", serde_json::json!([]))
}
