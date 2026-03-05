//! Activity log endpoints.
//!
//! Handles listing activities (audit log).

use axum::extract::{Query, State};
use serde::Deserialize;

use crate::middleware::auth::AuthenticatedUser;
use crate::response::{encode_service_error, fleet_error, fleet_ok, FleetResponse};
use crate::AppState;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListActivitiesParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/activities
pub async fn list_activities(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListActivitiesParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let page = params.page.unwrap_or(0) as u32;
    let per_page = params.per_page.unwrap_or(20) as u32;
    match state.service.list_activities(&viewer, page, per_page).await {
        Ok(activities) => fleet_ok("activities", serde_json::to_value(&activities).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}
