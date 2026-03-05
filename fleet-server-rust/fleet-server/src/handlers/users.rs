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
    // TODO: extract AppState and auth from request context
    // let viewer = auth.to_viewer();
    // let opts = fleet_types::user::UserListOptions {
    //     match_query: params.query,
    //     page: params.page.unwrap_or(0),
    //     per_page: params.per_page.unwrap_or(20),
    //     order_key: params.order_key,
    //     order_direction: params.order_direction,
    //     team_id: params.team_id,
    //     ..Default::default()
    // };
    // match state.service.list_users(&viewer, opts).await {
    //     Ok(users) => fleet_ok("users", serde_json::to_value(&users).unwrap()),
    //     Err(e) => encode_service_error(&e),
    // }
    fleet_ok("users", serde_json::json!([]))
}

/// POST /api/_version_/fleet/users/admin
pub async fn create_user(Json(_body): Json<CreateUserBody>) -> FleetResponse {
    // TODO: extract AppState and auth from request context
    // let viewer = auth.to_viewer();
    // let payload = fleet_service::users::CreateUserPayload {
    //     name: body.name.unwrap_or_default(),
    //     email: body.email.unwrap_or_default(),
    //     password: body.password,
    //     global_role: body.global_role,
    //     teams: None, // TODO: convert body.teams from JSON to Vec<UserTeam>
    //     ..Default::default()
    // };
    // match state.service.create_user(&viewer, payload).await {
    //     Ok((user, api_token)) => {
    //         let mut user_json = serde_json::to_value(&user).unwrap();
    //         if let Some(token) = api_token {
    //             user_json["api_token"] = serde_json::Value::String(token);
    //         }
    //         fleet_ok("user", user_json)
    //     }
    //     Err(e) => encode_error(&e),
    // }
    fleet_ok("user", serde_json::json!({}))
}

/// GET /api/_version_/fleet/users/{id}
pub async fn get_user(Path(_id): Path<u64>) -> FleetResponse {
    // TODO: extract AppState and auth from request context
    // let viewer = auth.to_viewer();
    // match state.service.get_user(&viewer, id as u32).await {
    //     Ok(user) => fleet_ok("user", serde_json::to_value(&user).unwrap()),
    //     Err(e) => encode_error(&e),
    // }
    fleet_ok("user", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/users/{id}
pub async fn modify_user(Path(_id): Path<u64>, Json(_body): Json<ModifyUserBody>) -> FleetResponse {
    // TODO: extract AppState and auth from request context
    // let viewer = auth.to_viewer();
    // let payload = fleet_service::users::ModifyUserPayload {
    //     name: body.name,
    //     email: body.email,
    //     global_role: body.global_role.map(Some),
    //     teams: None, // TODO: convert body.teams from JSON to Vec<UserTeam>
    //     ..Default::default()
    // };
    // match state.service.modify_user(&viewer, id as u32, payload).await {
    //     Ok(user) => fleet_ok("user", serde_json::to_value(&user).unwrap()),
    //     Err(e) => encode_error(&e),
    // }
    fleet_ok("user", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/users/{id}
pub async fn delete_user(Path(_id): Path<u64>) -> FleetResponse {
    // TODO: extract AppState and auth from request context
    // let viewer = auth.to_viewer();
    // match state.service.delete_user(&viewer, id as u32).await {
    //     Ok(_user) => fleet_ok("", serde_json::json!({})),
    //     Err(e) => encode_error(&e),
    // }
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/users/{id}/require_password_reset
pub async fn require_password_reset(
    Path(_id): Path<u64>,
    Json(_body): Json<RequirePasswordResetBody>,
) -> FleetResponse {
    // TODO: extract AppState and auth from request context
    // let viewer = auth.to_viewer();
    // match state.service.require_password_reset(&viewer, id as u32, body.require).await {
    //     Ok(user) => fleet_ok("user", serde_json::to_value(&user).unwrap()),
    //     Err(e) => encode_error(&e),
    // }
    fleet_ok("user", serde_json::json!({}))
}

/// GET /api/_version_/fleet/users/{id}/sessions
pub async fn get_user_sessions(Path(_id): Path<u64>) -> FleetResponse {
    // TODO: extract AppState and auth from request context
    // let viewer = auth.to_viewer();
    // match state.service.get_info_about_sessions_for_user(&viewer, id as u32).await {
    //     Ok(sessions) => fleet_ok("sessions", serde_json::to_value(&sessions).unwrap()),
    //     Err(e) => encode_error(&e),
    // }
    fleet_ok("sessions", serde_json::json!([]))
}

/// DELETE /api/_version_/fleet/users/{id}/sessions
pub async fn delete_user_sessions(Path(_id): Path<u64>) -> FleetResponse {
    // TODO: extract AppState and auth from request context
    // let viewer = auth.to_viewer();
    // match state.service.delete_sessions_for_user(&viewer, id as u32).await {
    //     Ok(()) => fleet_ok("", serde_json::json!({})),
    //     Err(e) => encode_error(&e),
    // }
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/change_password
pub async fn change_password(Json(_body): Json<ChangePasswordBody>) -> FleetResponse {
    // TODO: extract AppState and auth from request context
    // let viewer = auth.to_viewer();
    // match state.service.change_password(&viewer, &body.old_password, &body.new_password).await {
    //     Ok(()) => fleet_ok("", serde_json::json!({})),
    //     Err(e) => encode_error(&e),
    // }
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/email/change/{token}
pub async fn change_email(Path(_token): Path<String>) -> FleetResponse {
    // TODO: extract AppState from request context
    // match state.service.change_email(&token).await {
    //     Ok(new_email) => fleet_ok("new_email", serde_json::json!(new_email)),
    //     Err(e) => encode_error(&e),
    // }
    fleet_ok("new_email", serde_json::json!(""))
}

/// POST /api/_version_/fleet/users/roles/spec
pub async fn apply_user_role_specs(Json(_body): Json<ApplyUserRoleSpecsBody>) -> FleetResponse {
    // TODO: extract AppState and auth from request context
    // let viewer = auth.to_viewer();
    // match state.service.apply_user_role_specs(&viewer, body.spec).await {
    //     Ok(()) => fleet_ok("", serde_json::json!({})),
    //     Err(e) => encode_error(&e),
    // }
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/users (unauthenticated, from invite)
pub async fn create_user_from_invite(Json(_body): Json<CreateUserBody>) -> FleetResponse {
    // TODO: extract AppState from request context (no auth required)
    // let payload = fleet_service::users::CreateUserFromInvitePayload {
    //     name: body.name.unwrap_or_default(),
    //     email: body.email.unwrap_or_default(),
    //     password: body.password.unwrap_or_default(),
    //     invite_token: body.invite_token.unwrap_or_default(),
    // };
    // match state.service.create_user_from_invite(payload).await {
    //     Ok(user) => fleet_ok("user", serde_json::to_value(&user).unwrap()),
    //     Err(e) => encode_error(&e),
    // }
    fleet_ok("user", serde_json::json!({}))
}
