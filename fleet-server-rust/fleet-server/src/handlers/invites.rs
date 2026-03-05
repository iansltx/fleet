//! Invite management endpoints.
//!
//! Handles invite CRUD and verification.

use axum::extract::{Json, Path, Query, State};
use serde::Deserialize;

use crate::middleware::auth::AuthenticatedUser;
use crate::response::{fleet_error, fleet_ok, encode_service_error, FleetResponse};
use crate::AppState;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListInvitesParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInviteBody {
    pub email: String,
    pub name: Option<String>,
    pub global_role: Option<String>,
    pub teams: Option<Vec<serde_json::Value>>,
    pub sso_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInviteBody {
    pub email: Option<String>,
    pub name: Option<String>,
    pub global_role: Option<String>,
    pub teams: Option<Vec<serde_json::Value>>,
    pub sso_enabled: Option<bool>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/invites
pub async fn create_invite(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<CreateInviteBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_service::invites::InvitePayload {
        email: Some(body.email),
        name: body.name,
        sso_enabled: body.sso_enabled,
        global_role: body.global_role,
        // TODO: convert body.teams from JSON to Vec<UserTeam>
        teams: None,
        ..Default::default()
    };
    match state.service.invite_new_user(&viewer, payload).await {
        Ok(invite) => fleet_ok("invite", serde_json::to_value(&invite).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/invites
pub async fn list_invites(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListInvitesParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let opts = fleet_types::ListOptions {
        page: params.page.unwrap_or(0) as u32,
        per_page: params.per_page.unwrap_or(20) as u32,
        order_key: params.order_key.unwrap_or_default(),
        order_direction: params.order_direction
            .as_deref()
            .map(|d| if d == "desc" { fleet_types::OrderDirection::Descending } else { fleet_types::OrderDirection::Ascending })
            .unwrap_or_default(),
        match_query: params.query.unwrap_or_default(),
        ..Default::default()
    };
    match state.service.list_invites(&viewer, opts).await {
        Ok(invites) => fleet_ok("invites", serde_json::to_value(&invites).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/invites/{id}
pub async fn delete_invite(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_invite(&viewer, id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/invites/{id}
pub async fn update_invite(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<UpdateInviteBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_service::invites::InvitePayload {
        email: body.email,
        name: body.name,
        sso_enabled: body.sso_enabled,
        global_role: body.global_role,
        // TODO: convert body.teams from JSON to Vec<UserTeam>
        teams: None,
        ..Default::default()
    };
    match state.service.update_invite(&viewer, id as u32, payload).await {
        Ok(invite) => fleet_ok("invite", serde_json::to_value(&invite).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/invites/{token} (unauthenticated)
pub async fn verify_invite(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> FleetResponse {
    match state.service.verify_invite(&token).await {
        Ok(invite) => fleet_ok("invite", serde_json::to_value(&invite).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}
