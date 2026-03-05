//! User management endpoints.
//!
//! Handles user CRUD, password management, role assignments, and email changes.

use axum::extract::{Json, Path, Query, State};
use serde::Deserialize;

use crate::middleware::auth::AuthenticatedUser;
use crate::response::{fleet_error, fleet_ok, encode_service_error, FleetResponse};
use crate::AppState;

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
pub async fn list_users(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListUsersParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let opts = fleet_types::user::UserListOptions {
        list_options: fleet_types::ListOptions {
            match_query: params.query.unwrap_or_default(),
            page: params.page.unwrap_or(0) as u32,
            per_page: params.per_page.unwrap_or(20) as u32,
            order_key: params.order_key.unwrap_or_default(),
            order_direction: params.order_direction
                .as_deref()
                .map(|d| if d == "desc" { fleet_types::OrderDirection::Descending } else { fleet_types::OrderDirection::Ascending })
                .unwrap_or_default(),
            ..Default::default()
        },
        team_id: params.team_id.map(|t| t as u32),
    };
    match state.service.list_users(&viewer, opts).await {
        Ok(users) => fleet_ok("users", serde_json::to_value(&users).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/users/admin
pub async fn create_user(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<CreateUserBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_service::users::CreateUserPayload {
        name: body.name.unwrap_or_default(),
        email: body.email.unwrap_or_default(),
        password: body.password,
        global_role: body.global_role,
        // TODO: convert body.teams from JSON to Vec<UserTeam>
        teams: None,
        ..Default::default()
    };
    match state.service.create_user(&viewer, payload).await {
        Ok((user, api_token)) => {
            let mut user_json = serde_json::to_value(&user).unwrap_or_default();
            if let Some(token) = api_token {
                if let Some(obj) = user_json.as_object_mut() {
                    obj.insert("api_token".to_string(), serde_json::Value::String(token));
                }
            }
            fleet_ok("user", user_json)
        }
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/users/{id}
pub async fn get_user(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_user(&viewer, id as u32).await {
        Ok(user) => fleet_ok("user", serde_json::to_value(&user).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/users/{id}
pub async fn modify_user(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<ModifyUserBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_service::users::ModifyUserPayload {
        name: body.name,
        email: body.email,
        global_role: body.global_role.map(Some),
        // TODO: convert body.teams from JSON to Vec<UserTeam>
        teams: None,
        ..Default::default()
    };
    match state.service.modify_user(&viewer, id as u32, payload).await {
        Ok(user) => fleet_ok("user", serde_json::to_value(&user).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/users/{id}
pub async fn delete_user(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_user(&viewer, id as u32).await {
        Ok(_user) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/users/{id}/require_password_reset
pub async fn require_password_reset(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<RequirePasswordResetBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.require_password_reset(&viewer, id as u32, body.require).await {
        Ok(user) => fleet_ok("user", serde_json::to_value(&user).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/users/{id}/sessions
pub async fn get_user_sessions(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_id): Path<u64>,
) -> FleetResponse {
    // Verify the user is authenticated and authorized
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // TODO: list_sessions_for_user is not yet on the Datastore trait.
    // Once added, call: state.service.datastore().list_sessions_for_user(id as u32)
    fleet_ok("sessions", serde_json::json!([]))
}

/// DELETE /api/_version_/fleet/users/{id}/sessions
pub async fn delete_user_sessions(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.datastore().destroy_all_sessions_for_user(id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/change_password
pub async fn change_password(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<ChangePasswordBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.change_password(&viewer, &body.old_password, &body.new_password).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/email/change/{token}
pub async fn change_email(
    State(_state): State<AppState>,
    Path(_token): Path<String>,
) -> FleetResponse {
    // TODO: add change_email to FleetService
    fleet_ok("new_email", serde_json::json!(""))
}

/// POST /api/_version_/fleet/users/roles/spec
pub async fn apply_user_role_specs(
    State(_state): State<AppState>,
    auth: AuthenticatedUser,
    Json(_body): Json<ApplyUserRoleSpecsBody>,
) -> FleetResponse {
    // Verify authentication
    let _viewer = match auth.viewer(&_state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // TODO: add apply_user_role_specs to FleetService
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/users (unauthenticated, from invite)
pub async fn create_user_from_invite(
    State(state): State<AppState>,
    Json(body): Json<CreateUserBody>,
) -> FleetResponse {
    let payload = fleet_service::users::CreateUserFromInvitePayload {
        name: body.name.unwrap_or_default(),
        email: body.email.unwrap_or_default(),
        password: body.password.unwrap_or_default(),
        invite_token: body.invite_token.unwrap_or_default(),
    };
    match state.service.create_user_from_invite(payload).await {
        Ok(user) => fleet_ok("user", serde_json::to_value(&user).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}
