//! User management endpoints.
//!
//! Handles user CRUD, password management, role assignments, and email changes.

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
pub struct ListUsersParams {
    pub query: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
    pub team_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserBody {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub global_role: Option<String>,
    pub teams: Option<Vec<serde_json::Value>>,
    pub invite_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ModifyUserBody {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub global_role: Option<String>,
    pub teams: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordBody {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ApplyUserRoleSpecsBody {
    pub spec: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct RequirePasswordResetBody {
    pub require: bool,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/users
pub async fn list_users(Query(_params): Query<ListUsersParams>) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("users", serde_json::json!([]))
}

/// POST /api/_version_/fleet/users/admin
pub async fn create_user(Json(_body): Json<CreateUserBody>) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("user", serde_json::json!({}))
}

/// GET /api/_version_/fleet/users/{id}
pub async fn get_user(Path(_id): Path<u64>) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("user", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/users/{id}
pub async fn modify_user(Path(_id): Path<u64>, Json(_body): Json<ModifyUserBody>) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("user", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/users/{id}
pub async fn delete_user(Path(_id): Path<u64>) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/users/{id}/require_password_reset
pub async fn require_password_reset(
    Path(_id): Path<u64>,
    Json(_body): Json<RequirePasswordResetBody>,
) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("user", serde_json::json!({}))
}

/// GET /api/_version_/fleet/users/{id}/sessions
pub async fn get_user_sessions(Path(_id): Path<u64>) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("sessions", serde_json::json!([]))
}

/// DELETE /api/_version_/fleet/users/{id}/sessions
pub async fn delete_user_sessions(Path(_id): Path<u64>) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/change_password
pub async fn change_password(Json(_body): Json<ChangePasswordBody>) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/email/change/{token}
pub async fn change_email(Path(_token): Path<String>) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("new_email", serde_json::json!(""))
}

/// POST /api/_version_/fleet/users/roles/spec
pub async fn apply_user_role_specs(Json(_body): Json<ApplyUserRoleSpecsBody>) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/users (unauthenticated, from invite)
pub async fn create_user_from_invite(Json(_body): Json<CreateUserBody>) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("user", serde_json::json!({}))
}
