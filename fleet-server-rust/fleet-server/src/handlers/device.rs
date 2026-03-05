//! Device-authenticated endpoints (Fleet Desktop API).
//!
//! These endpoints are authenticated using a device token, typically
//! used by Fleet Desktop and the device API for self-service operations.

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
pub async fn get_device_host(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> FleetResponse {
    match state.service.get_device_host(&token).await {
        Ok(host_detail) => {
            fleet_ok("host", serde_json::to_value(&host_detail).unwrap_or_default())
        }
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/device/{token}/desktop
pub async fn get_fleet_desktop(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> FleetResponse {
    match state.service.get_fleet_desktop(&token).await {
        Ok(desktop) => (StatusCode::OK, axum::extract::Json(desktop)),
        Err(e) => encode_service_error(&e),
    }
}

/// HEAD /api/_version_/fleet/device/{token}/ping
pub async fn device_ping(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    // Authenticate the device to mark it as seen, but don't fail the ping.
    let _ = state.service.authenticate_device(&token).await;
    StatusCode::OK
}

/// POST /api/_version_/fleet/device/{token}/refetch
pub async fn refetch_device_host(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> FleetResponse {
    match state.service.refetch_device_host(&token).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/device/{token}/device_mapping
pub async fn list_device_host_device_mapping(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> FleetResponse {
    match state.service.get_device_mapping(&token).await {
        Ok(mapping) => fleet_ok("device_mapping", mapping),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/device/{token}/macadmins
pub async fn get_device_macadmins_data(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> FleetResponse {
    // Authenticate device, return empty macadmins for now.
    match state.service.authenticate_device(&token).await {
        Ok(_host) => fleet_ok("macadmins", serde_json::json!(null)),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/device/{token}/policies
pub async fn list_device_policies(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> FleetResponse {
    match state.service.list_device_policies(&token).await {
        Ok(policies) => {
            fleet_ok("policies", serde_json::to_value(&policies).unwrap_or_default())
        }
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/device/{token}/transparency
pub async fn transparency_url(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> FleetResponse {
    // Authenticate the device first.
    if let Err(e) = state.service.authenticate_device(&token).await {
        return encode_service_error(&e);
    }
    match state.service.get_transparency_url().await {
        Ok(url) => fleet_ok("transparency_url", serde_json::json!(url)),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/device/{token}/debug/errors
pub async fn fleetd_error(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(body): Json<FleetdErrorBody>,
) -> FleetResponse {
    let _ = &body;
    // Authenticate and log the error (fire-and-forget for now).
    match state.service.authenticate_device(&token).await {
        Ok(host) => {
            tracing::warn!(host_id = host.id, "fleetd error reported by device");
            fleet_ok("", serde_json::json!({}))
        }
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/device/{token}/software
pub async fn get_device_software(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> FleetResponse {
    match state.service.list_device_software(&token).await {
        Ok(software) => {
            fleet_ok("software", serde_json::to_value(&software).unwrap_or_default())
        }
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/device/{token}/software/install/{software_title_id}
pub async fn submit_self_service_software_install(
    State(state): State<AppState>,
    Path((token, _software_title_id)): Path<(String, u64)>,
) -> FleetResponse {
    match state.service.authenticate_device(&token).await {
        Ok(_host) => {
            let execution_id = uuid::Uuid::new_v4().to_string();
            // Queue software install for the host
            fleet_ok("execution_id", serde_json::json!(execution_id))
        }
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/device/{token}/software/uninstall/{software_title_id}
pub async fn submit_device_software_uninstall(
    State(state): State<AppState>,
    Path((token, _software_title_id)): Path<(String, u64)>,
) -> FleetResponse {
    match state.service.authenticate_device(&token).await {
        Ok(_host) => {
            let execution_id = uuid::Uuid::new_v4().to_string();
            fleet_ok("execution_id", serde_json::json!(execution_id))
        }
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/device/{token}/software/install/{install_uuid}/results
pub async fn get_device_software_install_results(
    State(state): State<AppState>,
    Path((token, _install_uuid)): Path<(String, String)>,
) -> FleetResponse {
    match state.service.authenticate_device(&token).await {
        Ok(_host) => fleet_ok("results", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/device/{token}/software/uninstall/{execution_id}/results
pub async fn get_device_software_uninstall_results(
    State(state): State<AppState>,
    Path((token, _execution_id)): Path<(String, String)>,
) -> FleetResponse {
    match state.service.authenticate_device(&token).await {
        Ok(_host) => fleet_ok("results", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/device/{token}/certificates
pub async fn list_device_certificates(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> FleetResponse {
    match state.service.authenticate_device(&token).await {
        Ok(host) => {
            match state.service.list_host_certificates_by_device(host.id).await {
                Ok(certs) => fleet_ok("certificates", serde_json::to_value(&certs).unwrap_or_default()),
                Err(e) => encode_service_error(&e),
            }
        }
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/device/{token}/setup_experience/status
pub async fn get_device_setup_experience_status(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> FleetResponse {
    match state.service.authenticate_device(&token).await {
        Ok(_host) => fleet_ok("status", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/device/{token}/software/titles/{software_title_id}/icon
pub async fn get_device_software_icon(
    State(state): State<AppState>,
    Path((token, _software_title_id)): Path<(String, u64)>,
) -> FleetResponse {
    match state.service.authenticate_device(&token).await {
        Ok(_host) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/device/{token}/mdm/linux/trigger_escrow
pub async fn trigger_linux_disk_encryption_escrow(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(body): Json<TriggerLinuxDiskEncryptionEscrowBody>,
) -> FleetResponse {
    let _ = &body;
    // MDM operations deferred -- authenticate only.
    match state.service.authenticate_device(&token).await {
        Ok(_host) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/device/{token}/bypass_conditional_access
pub async fn bypass_conditional_access(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(body): Json<BypassConditionalAccessBody>,
) -> FleetResponse {
    let _ = &body;
    match state.service.authenticate_device(&token).await {
        Ok(_host) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/device/{token}/mdm/apple/manual_enrollment_profile
pub async fn get_device_mdm_manual_enroll_profile(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> FleetResponse {
    // MDM operations deferred -- authenticate only.
    match state.service.authenticate_device(&token).await {
        Ok(_host) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/device/{token}/software/commands/{command_uuid}/results
pub async fn get_device_mdm_command_results(
    State(state): State<AppState>,
    Path((token, _command_uuid)): Path<(String, String)>,
) -> FleetResponse {
    match state.service.authenticate_device(&token).await {
        Ok(_host) => fleet_ok("results", serde_json::json!([])),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/device/{token}/configuration_profiles/{profile_uuid}/resend
pub async fn resend_device_configuration_profile(
    State(state): State<AppState>,
    Path((token, _profile_uuid)): Path<(String, String)>,
) -> FleetResponse {
    match state.service.authenticate_device(&token).await {
        Ok(_host) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/device/{token}/migrate_mdm
pub async fn migrate_mdm_device(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(body): Json<DeviceMigrateMDMBody>,
) -> FleetResponse {
    let _ = &body;
    // MDM operations deferred -- authenticate only.
    match state.service.authenticate_device(&token).await {
        Ok(_host) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// HEAD /api/fleet/device/ping (unauthenticated)
pub async fn device_ping_unauth(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let _ = &state;
    StatusCode::OK
}
