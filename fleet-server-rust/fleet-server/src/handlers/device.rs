//! Device-authenticated endpoints (Fleet Desktop API).
//!
//! These endpoints are authenticated using a device token, typically
//! used by Fleet Desktop and the device API for self-service operations.

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
pub struct FleetdErrorBody {
    pub error: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct BypassConditionalAccessBody {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct DeviceMigrateMDMBody {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ResendDeviceConfigurationProfileBody {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct TriggerLinuxDiskEncryptionEscrowBody {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/device/{token}
pub async fn get_device_host(Path(_token): Path<String>) -> FleetResponse {
    fleet_ok("host", serde_json::json!({}))
}

/// GET /api/_version_/fleet/device/{token}/desktop
pub async fn get_fleet_desktop(Path(_token): Path<String>) -> FleetResponse {
    fleet_ok("desktop", serde_json::json!({}))
}

/// HEAD /api/_version_/fleet/device/{token}/ping
pub async fn device_ping(Path(_token): Path<String>) -> impl IntoResponse {
    StatusCode::OK
}

/// POST /api/_version_/fleet/device/{token}/refetch
pub async fn refetch_device_host(Path(_token): Path<String>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/device/{token}/device_mapping
pub async fn list_device_host_device_mapping(Path(_token): Path<String>) -> FleetResponse {
    fleet_ok("device_mapping", serde_json::json!([]))
}

/// GET /api/_version_/fleet/device/{token}/macadmins
pub async fn get_device_macadmins_data(Path(_token): Path<String>) -> FleetResponse {
    fleet_ok("macadmins", serde_json::json!({}))
}

/// GET /api/_version_/fleet/device/{token}/policies
pub async fn list_device_policies(Path(_token): Path<String>) -> FleetResponse {
    fleet_ok("policies", serde_json::json!([]))
}

/// GET /api/_version_/fleet/device/{token}/transparency
pub async fn transparency_url(Path(_token): Path<String>) -> FleetResponse {
    fleet_ok("transparency_url", serde_json::json!(""))
}

/// POST /api/_version_/fleet/device/{token}/debug/errors
pub async fn fleetd_error(
    Path(_token): Path<String>,
    Json(_body): Json<FleetdErrorBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/device/{token}/software
pub async fn get_device_software(Path(_token): Path<String>) -> FleetResponse {
    fleet_ok("software", serde_json::json!([]))
}

/// POST /api/_version_/fleet/device/{token}/software/install/{software_title_id}
pub async fn submit_self_service_software_install(
    Path((_token, _software_title_id)): Path<(String, u64)>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/device/{token}/software/uninstall/{software_title_id}
pub async fn submit_device_software_uninstall(
    Path((_token, _software_title_id)): Path<(String, u64)>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/device/{token}/software/install/{install_uuid}/results
pub async fn get_device_software_install_results(
    Path((_token, _install_uuid)): Path<(String, String)>,
) -> FleetResponse {
    fleet_ok("results", serde_json::json!({}))
}

/// GET /api/_version_/fleet/device/{token}/software/uninstall/{execution_id}/results
pub async fn get_device_software_uninstall_results(
    Path((_token, _execution_id)): Path<(String, String)>,
) -> FleetResponse {
    fleet_ok("results", serde_json::json!({}))
}

/// GET /api/_version_/fleet/device/{token}/certificates
pub async fn list_device_certificates(Path(_token): Path<String>) -> FleetResponse {
    fleet_ok("certificates", serde_json::json!([]))
}

/// POST /api/_version_/fleet/device/{token}/setup_experience/status
pub async fn get_device_setup_experience_status(Path(_token): Path<String>) -> FleetResponse {
    fleet_ok("status", serde_json::json!({}))
}

/// GET /api/_version_/fleet/device/{token}/software/titles/{software_title_id}/icon
pub async fn get_device_software_icon(
    Path((_token, _software_title_id)): Path<(String, u64)>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/device/{token}/mdm/linux/trigger_escrow
pub async fn trigger_linux_disk_encryption_escrow(
    Path(_token): Path<String>,
    Json(_body): Json<TriggerLinuxDiskEncryptionEscrowBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/device/{token}/bypass_conditional_access
pub async fn bypass_conditional_access(
    Path(_token): Path<String>,
    Json(_body): Json<BypassConditionalAccessBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/device/{token}/mdm/apple/manual_enrollment_profile
pub async fn get_device_mdm_manual_enroll_profile(Path(_token): Path<String>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/device/{token}/software/commands/{command_uuid}/results
pub async fn get_device_mdm_command_results(
    Path((_token, _command_uuid)): Path<(String, String)>,
) -> FleetResponse {
    fleet_ok("results", serde_json::json!([]))
}

/// POST /api/_version_/fleet/device/{token}/configuration_profiles/{profile_uuid}/resend
pub async fn resend_device_configuration_profile(
    Path((_token, _profile_uuid)): Path<(String, String)>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/device/{token}/migrate_mdm
pub async fn migrate_mdm_device(
    Path(_token): Path<String>,
    Json(_body): Json<DeviceMigrateMDMBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// HEAD /api/fleet/device/ping (unauthenticated)
pub async fn device_ping_unauth() -> impl IntoResponse {
    StatusCode::OK
}
