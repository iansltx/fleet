//! Setup and setup experience endpoints.
//!
//! Handles initial Fleet setup and macOS setup experience configuration.

use axum::{
    extract::Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::response::{fleet_error, fleet_ok, FleetResponse};

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
pub async fn setup(Json(body): Json<SetupBody>) -> FleetResponse {
    // When AppState is available:
    // let payload = CreateUserPayload {
    //     name: body.admin.name,
    //     email: body.admin.email,
    //     password: Some(body.admin.password),
    //     global_role: Some("admin".to_string()),
    //     ..Default::default()
    // };
    // match state.service.create_initial_user(payload).await {
    //     Ok(user) => {
    //         // Also save org_info to app config
    //         fleet_ok("admin", serde_json::to_value(&user).unwrap())
    //     }
    //     Err(e) => encode_service_error(&e),
    // }
    fleet_ok("admin", serde_json::json!({}))
}

/// PUT /api/_version_/fleet/setup_experience/software
pub async fn put_setup_experience_software(
    Json(_body): Json<PutSetupExperienceSoftwareBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/setup_experience/software
pub async fn get_setup_experience_software(
    axum::extract::Query(_params): axum::extract::Query<GetSetupExperienceSoftwareParams>,
) -> FleetResponse {
    fleet_ok("software_titles", serde_json::json!([]))
}

/// GET /api/_version_/fleet/setup_experience/script
pub async fn get_setup_experience_script(
    axum::extract::Query(_params): axum::extract::Query<GetSetupExperienceScriptParams>,
) -> FleetResponse {
    fleet_ok("script", serde_json::json!({}))
}

/// POST /api/_version_/fleet/setup_experience/script
pub async fn set_setup_experience_script() -> FleetResponse {
    // Multipart form upload of script content
    fleet_ok("script_id", serde_json::json!(0))
}

/// DELETE /api/_version_/fleet/setup_experience/script
pub async fn delete_setup_experience_script(
    axum::extract::Query(_params): axum::extract::Query<DeleteSetupExperienceScriptParams>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}
