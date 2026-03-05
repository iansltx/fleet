//! Orbit-authenticated endpoints.
//!
//! Handles orbit device token management, config, scripts, software installs,
//! disk encryption key escrow, and setup experience.

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::response::{encode_service_error, fleet_ok, FleetResponse};
use crate::AppState;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct EnrollOrbitBody {
    pub enroll_secret: String,
    pub hardware_uuid: Option<String>,
    pub hardware_serial: Option<String>,
    pub os_version: Option<String>,
    pub orbit_version: Option<String>,
    pub platform: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetOrUpdateDeviceTokenBody {
    pub orbit_node_key: String,
    pub device_auth_token: String,
}

#[derive(Debug, Deserialize)]
pub struct OrbitGetConfigBody {
    pub orbit_node_key: String,
}

#[derive(Debug, Deserialize)]
pub struct OrbitGetScriptBody {
    pub orbit_node_key: String,
    pub execution_id: String,
}

#[derive(Debug, Deserialize)]
pub struct OrbitPostScriptResultBody {
    pub orbit_node_key: String,
    pub execution_id: String,
    pub exit_code: i32,
    pub output: String,
    pub runtime: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct OrbitPutDeviceMappingBody {
    pub orbit_node_key: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OrbitPostSoftwareInstallResultBody {
    pub orbit_node_key: String,
    pub install_uuid: String,
    #[serde(flatten)]
    pub rest: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct OrbitDownloadSoftwareInstallerBody {
    pub orbit_node_key: String,
    pub install_uuid: String,
}

#[derive(Debug, Deserialize)]
pub struct OrbitGetSoftwareInstallBody {
    pub orbit_node_key: String,
    pub install_uuid: String,
}

#[derive(Debug, Deserialize)]
pub struct OrbitSetupExperienceInitBody {
    pub orbit_node_key: String,
}

#[derive(Debug, Deserialize)]
pub struct GetOrbitSetupExperienceStatusBody {
    pub orbit_node_key: String,
}

#[derive(Debug, Deserialize)]
pub struct OrbitPostDiskEncryptionKeyBody {
    pub orbit_node_key: String,
    pub encryption_key: Vec<u8>,
    pub client_error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OrbitPostLUKSBody {
    pub orbit_node_key: String,
    pub passphrase: String,
    pub slot_key: Option<String>,
    pub client_error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetDeviceCertificateTemplateParams {
    // Authentication header parsed by middleware
}

#[derive(Debug, Deserialize)]
pub struct UpdateCertificateStatusBody {
    pub status: String,
    #[serde(flatten)]
    pub rest: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/fleet/orbit/enroll (unauthenticated)
pub async fn enroll_orbit(
    State(state): State<AppState>,
    Json(body): Json<EnrollOrbitBody>,
) -> FleetResponse {
    match state
        .service
        .enroll_orbit(
            &body.enroll_secret,
            body.hardware_uuid.as_deref(),
            body.hardware_serial.as_deref(),
        )
        .await
    {
        Ok(orbit_node_key) => fleet_ok("orbit_node_key", serde_json::json!(orbit_node_key)),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/fleet/orbit/device_token
pub async fn set_or_update_device_token(
    State(state): State<AppState>,
    Json(body): Json<SetOrUpdateDeviceTokenBody>,
) -> FleetResponse {
    match state
        .service
        .set_or_update_device_token(&body.orbit_node_key, &body.device_auth_token)
        .await
    {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/fleet/orbit/config
pub async fn get_orbit_config(
    State(state): State<AppState>,
    Json(body): Json<OrbitGetConfigBody>,
) -> FleetResponse {
    match state.service.get_orbit_config(&body.orbit_node_key).await {
        Ok(config) => (StatusCode::OK, axum::extract::Json(config)),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/fleet/orbit/scripts/request
pub async fn get_orbit_script(
    State(state): State<AppState>,
    Json(body): Json<OrbitGetScriptBody>,
) -> FleetResponse {
    match state
        .service
        .get_orbit_script(&body.orbit_node_key, &body.execution_id)
        .await
    {
        Ok(result) => {
            fleet_ok("", serde_json::to_value(&result).unwrap_or_default())
        }
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/fleet/orbit/scripts/result
pub async fn post_orbit_script_result(
    State(state): State<AppState>,
    Json(body): Json<OrbitPostScriptResultBody>,
) -> FleetResponse {
    match state
        .service
        .post_orbit_script_result(
            &body.orbit_node_key,
            &body.execution_id,
            body.exit_code,
            &body.output,
            body.runtime,
        )
        .await
    {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// PUT /api/fleet/orbit/device_mapping
pub async fn put_orbit_device_mapping(
    State(state): State<AppState>,
    Json(body): Json<OrbitPutDeviceMappingBody>,
) -> FleetResponse {
    match state
        .service
        .put_orbit_device_mapping(&body.orbit_node_key, body.email.as_deref())
        .await
    {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/fleet/orbit/software_install/result
pub async fn post_orbit_software_install_result(
    State(state): State<AppState>,
    Json(body): Json<OrbitPostSoftwareInstallResultBody>,
) -> FleetResponse {
    // Authenticate the orbit agent. Full implementation would record the install result.
    match state.service.authenticate_orbit(&body.orbit_node_key).await {
        Ok(_host) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/fleet/orbit/software_install/package
pub async fn orbit_download_software_installer(
    State(state): State<AppState>,
    Json(body): Json<OrbitDownloadSoftwareInstallerBody>,
) -> FleetResponse {
    // Authenticate the orbit agent. Full implementation would return the installer binary.
    match state.service.authenticate_orbit(&body.orbit_node_key).await {
        Ok(_host) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/fleet/orbit/software_install/details
pub async fn get_orbit_software_install_details(
    State(state): State<AppState>,
    Json(body): Json<OrbitGetSoftwareInstallBody>,
) -> FleetResponse {
    match state.service.authenticate_orbit(&body.orbit_node_key).await {
        Ok(_host) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/fleet/orbit/setup_experience/init
pub async fn orbit_setup_experience_init(
    State(state): State<AppState>,
    Json(body): Json<OrbitSetupExperienceInitBody>,
) -> FleetResponse {
    match state.service.authenticate_orbit(&body.orbit_node_key).await {
        Ok(_host) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/fleet/orbit/setup_experience/status
pub async fn get_orbit_setup_experience_status(
    State(state): State<AppState>,
    Json(body): Json<GetOrbitSetupExperienceStatusBody>,
) -> FleetResponse {
    match state.service.authenticate_orbit(&body.orbit_node_key).await {
        Ok(_host) => fleet_ok("status", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/fleet/orbit/disk_encryption_key
pub async fn post_orbit_disk_encryption_key(
    State(state): State<AppState>,
    Json(body): Json<OrbitPostDiskEncryptionKeyBody>,
) -> FleetResponse {
    match state
        .service
        .post_orbit_disk_encryption_key(
            &body.orbit_node_key,
            &body.encryption_key,
            body.client_error.as_deref(),
        )
        .await
    {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/fleet/orbit/luks_data
pub async fn post_orbit_luks(
    State(state): State<AppState>,
    Json(body): Json<OrbitPostLUKSBody>,
) -> FleetResponse {
    match state
        .service
        .post_orbit_luks_data(
            &body.orbit_node_key,
            &body.passphrase,
            body.slot_key.as_deref(),
            body.client_error.as_deref(),
        )
        .await
    {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// HEAD /api/fleet/orbit/ping (unauthenticated)
pub async fn orbit_ping(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let _ = &state;
    StatusCode::OK
}

/// GET /api/fleetd/certificates/{id}
pub async fn get_device_certificate_template(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> FleetResponse {
    let _ = (&state, id);
    fleet_ok("certificate_template", serde_json::json!({}))
}

/// PUT /api/fleetd/certificates/{id}/status
pub async fn update_certificate_status(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(body): Json<UpdateCertificateStatusBody>,
) -> FleetResponse {
    let _ = (&state, id, &body);
    fleet_ok("", serde_json::json!({}))
}
