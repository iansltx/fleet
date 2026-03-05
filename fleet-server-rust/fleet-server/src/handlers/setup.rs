//! Setup and setup experience endpoints.
//!
//! Handles initial Fleet setup and macOS setup experience configuration.

use axum::extract::{Json, State};
use serde::Deserialize;

use crate::middleware::auth::AuthenticatedUser;
use crate::response::{encode_service_error, fleet_error, fleet_ok, FleetResponse};
use crate::AppState;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SetupBody {
    pub admin: SetupAdmin,
    pub org_info: SetupOrgInfo,
    pub server_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetupAdmin {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SetupOrgInfo {
    pub org_name: String,
    pub org_logo_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PutSetupExperienceSoftwareBody {
    pub team_id: Option<u64>,
    pub software_title_ids: Option<Vec<u64>>,
}

#[derive(Debug, Deserialize)]
pub struct GetSetupExperienceSoftwareParams {
    pub team_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct GetSetupExperienceScriptParams {
    pub team_id: Option<u64>,
    pub alt: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteSetupExperienceScriptParams {
    pub team_id: Option<u64>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/v1/setup
/// POST /api/setup
///
/// Creates the initial admin user and configures the Fleet server.
/// This endpoint is only available before setup is complete.
pub async fn setup(
    State(state): State<AppState>,
    Json(body): Json<SetupBody>,
) -> FleetResponse {
    let payload = fleet_service::users::CreateUserPayload {
        name: body.admin.name,
        email: body.admin.email,
        password: Some(body.admin.password),
        global_role: Some("admin".to_string()),
        ..Default::default()
    };
    match state.service.create_initial_user(payload).await {
        Ok(user) => fleet_ok("admin", serde_json::to_value(&user).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// PUT /api/_version_/fleet/setup_experience/software
pub async fn put_setup_experience_software(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<PutSetupExperienceSoftwareBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &body);
    // Stub: setup experience software deferred
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/setup_experience/software
pub async fn get_setup_experience_software(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    axum::extract::Query(params): axum::extract::Query<GetSetupExperienceSoftwareParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &params);
    // Stub: setup experience software listing deferred
    fleet_ok("software_titles", serde_json::json!([]))
}

/// GET /api/_version_/fleet/setup_experience/script
pub async fn get_setup_experience_script(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    axum::extract::Query(params): axum::extract::Query<GetSetupExperienceScriptParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &params);
    // Stub: setup experience script deferred
    fleet_ok("script", serde_json::json!({}))
}

/// POST /api/_version_/fleet/setup_experience/script
pub async fn set_setup_experience_script(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    // Stub: multipart upload deferred
    fleet_ok("script_id", serde_json::json!(0))
}

/// DELETE /api/_version_/fleet/setup_experience/script
pub async fn delete_setup_experience_script(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    axum::extract::Query(params): axum::extract::Query<DeleteSetupExperienceScriptParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &params);
    // Stub: setup experience script deletion deferred
    fleet_ok("", serde_json::json!({}))
}
