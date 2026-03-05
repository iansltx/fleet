//! Invite management endpoints.
//!
//! Handles invite CRUD and verification.

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
pub async fn create_invite(Json(_body): Json<CreateInviteBody>) -> FleetResponse {
    fleet_ok("invite", serde_json::json!({}))
}

/// GET /api/_version_/fleet/invites
pub async fn list_invites(Query(_params): Query<ListInvitesParams>) -> FleetResponse {
    fleet_ok("invites", serde_json::json!([]))
}

/// DELETE /api/_version_/fleet/invites/{id}
pub async fn delete_invite(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/invites/{id}
pub async fn update_invite(
    Path(_id): Path<u64>,
    Json(_body): Json<UpdateInviteBody>,
) -> FleetResponse {
    fleet_ok("invite", serde_json::json!({}))
}

/// GET /api/_version_/fleet/invites/{token} (unauthenticated)
pub async fn verify_invite(Path(_token): Path<String>) -> FleetResponse {
    fleet_ok("invite", serde_json::json!({}))
}
