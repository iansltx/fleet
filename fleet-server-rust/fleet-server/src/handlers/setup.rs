//! Setup and setup experience endpoints.
//!
//! Handles initial Fleet setup and macOS setup experience configuration.

use axum::extract::{Json, State};
use serde::Deserialize;

use crate::response::{encode_service_error, fleet_ok, FleetResponse};
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
    State(_state): State<AppState>,
    Json(_body): Json<PutSetupExperienceSoftwareBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/setup_experience/software
pub async fn get_setup_experience_software(
    State(_state): State<AppState>,
    axum::extract::Query(_params): axum::extract::Query<GetSetupExperienceSoftwareParams>,
) -> FleetResponse {
    fleet_ok("software_titles", serde_json::json!([]))
}

/// GET /api/_version_/fleet/setup_experience/script
pub async fn get_setup_experience_script(
    State(_state): State<AppState>,
    axum::extract::Query(_params): axum::extract::Query<GetSetupExperienceScriptParams>,
) -> FleetResponse {
    fleet_ok("script", serde_json::json!({}))
}

/// POST /api/_version_/fleet/setup_experience/script
pub async fn set_setup_experience_script(
    State(_state): State<AppState>,
) -> FleetResponse {
    // Multipart form upload of script content
    fleet_ok("script_id", serde_json::json!(0))
}

/// DELETE /api/_version_/fleet/setup_experience/script
pub async fn delete_setup_experience_script(
    State(_state): State<AppState>,
    axum::extract::Query(_params): axum::extract::Query<DeleteSetupExperienceScriptParams>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}
