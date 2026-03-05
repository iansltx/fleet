//! Orbit-authenticated endpoints.
//!
//! Handles orbit device token management, config, scripts, software installs,
//! disk encryption key escrow, and setup experience.

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::response::{fleet_error, fleet_ok, FleetResponse};

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
pub async fn enroll_orbit(Json(_body): Json<EnrollOrbitBody>) -> FleetResponse {
    fleet_ok("orbit_node_key", serde_json::json!(""))
}

/// POST /api/fleet/orbit/device_token
pub async fn set_or_update_device_token(
    Json(_body): Json<SetOrUpdateDeviceTokenBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/fleet/orbit/config
pub async fn get_orbit_config(Json(_body): Json<OrbitGetConfigBody>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/fleet/orbit/scripts/request
pub async fn get_orbit_script(Json(_body): Json<OrbitGetScriptBody>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/fleet/orbit/scripts/result
pub async fn post_orbit_script_result(
    Json(_body): Json<OrbitPostScriptResultBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// PUT /api/fleet/orbit/device_mapping
pub async fn put_orbit_device_mapping(
    Json(_body): Json<OrbitPutDeviceMappingBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/fleet/orbit/software_install/result
pub async fn post_orbit_software_install_result(
    Json(_body): Json<OrbitPostSoftwareInstallResultBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/fleet/orbit/software_install/package
pub async fn orbit_download_software_installer(
    Json(_body): Json<OrbitDownloadSoftwareInstallerBody>,
) -> FleetResponse {
    // TODO: return the actual software installer binary
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/fleet/orbit/software_install/details
pub async fn get_orbit_software_install_details(
    Json(_body): Json<OrbitGetSoftwareInstallBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/fleet/orbit/setup_experience/init
pub async fn orbit_setup_experience_init(
    Json(_body): Json<OrbitSetupExperienceInitBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/fleet/orbit/setup_experience/status
pub async fn get_orbit_setup_experience_status(
    Json(_body): Json<GetOrbitSetupExperienceStatusBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/fleet/orbit/disk_encryption_key
pub async fn post_orbit_disk_encryption_key(
    Json(_body): Json<OrbitPostDiskEncryptionKeyBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/fleet/orbit/luks_data
pub async fn post_orbit_luks(Json(_body): Json<OrbitPostLUKSBody>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// HEAD /api/fleet/orbit/ping (unauthenticated)
pub async fn orbit_ping() -> impl IntoResponse {
    StatusCode::OK
}

/// GET /api/fleetd/certificates/{id}
pub async fn get_device_certificate_template(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("certificate_template", serde_json::json!({}))
}

/// PUT /api/fleetd/certificates/{id}/status
pub async fn update_certificate_status(
    Path(_id): Path<u64>,
    Json(_body): Json<UpdateCertificateStatusBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}
