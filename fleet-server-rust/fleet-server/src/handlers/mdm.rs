//! MDM (Mobile Device Management) endpoints.
//!
//! Handles Apple MDM, Windows MDM, configuration profiles, bootstrap packages,
//! EULA, enrollment, ABM/VPP tokens, disk encryption, and device management.
//! Many of these endpoints have deprecated paths that are maintained for
//! backwards compatibility.

use axum::extract::{Json, Path, Query, State};
use serde::Deserialize;

use crate::middleware::auth::AuthenticatedUser;
use crate::response::{encode_service_error, fleet_error, fleet_ok, FleetResponse};
use crate::AppState;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct UpdateMDMAppleSetupBody {
    pub team_id: Option<u64>,
    pub enable_end_user_authentication: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct EnqueueMDMAppleCommandBody {
    pub command: serde_json::Value,
    pub device_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetMDMAppleCommandResultsParams {
    pub command_uuid: String,
}

#[derive(Debug, Deserialize)]
pub struct ListMDMCommandsParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RunMDMCommandBody {
    pub command: String,
    pub host_uuids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetMDMCommandResultsParams {
    pub command_uuid: String,
}

#[derive(Debug, Deserialize)]
pub struct NewMDMConfigProfileBody {
    // Multipart: team_id + profile file
    pub team_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ListMDMConfigProfilesParams {
    pub team_id: Option<u64>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct BatchModifyMDMConfigProfilesBody {
    pub profiles: Option<Vec<serde_json::Value>>,
    pub team_id: Option<u64>,
    pub dry_run: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct BatchSetMDMProfilesBody {
    pub profiles: Option<Vec<serde_json::Value>>,
    pub team_id: Option<u64>,
    pub dry_run: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMDMAppleSettingsBody {
    pub enable_disk_encryption: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDiskEncryptionBody {
    pub team_id: Option<u64>,
    pub enable_disk_encryption: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RequestMDMAppleCSRBody {
    pub email_address: String,
    pub organization: String,
}

#[derive(Debug, Deserialize)]
pub struct UploadABMTokenBody {
    // Multipart upload
}

#[derive(Debug, Deserialize)]
pub struct UpdateABMTokenTeamsBody {
    pub team_ids: Option<Vec<u64>>,
    pub ios_team_id: Option<u64>,
    pub ipados_team_id: Option<u64>,
    pub macos_team_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct PatchVPPTokensTeamsBody {
    pub team_ids: Option<Vec<u64>>,
}

#[derive(Debug, Deserialize)]
pub struct GetMDMDiskEncryptionSummaryParams {
    pub team_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct GetMDMProfilesSummaryParams {
    pub team_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct GetMDMAppleFileVaultSummaryParams {
    pub team_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct GetMDMAppleProfilesSummaryParams {
    pub team_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct GetMDMAppleBootstrapPackageSummaryParams {
    pub team_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct PreassignMDMAppleProfileBody {
    // Multipart: external host identifier + profile
    pub external_host_identifier: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MatchMDMApplePreassignmentBody {
    pub external_host_identifier: String,
}

#[derive(Debug, Deserialize)]
pub struct InitiateMDMSSOBody {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CallbackMDMSSOBody {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct BatchResendMDMProfileToHostsBody {
    pub profile_uuid: String,
    pub host_ids: Vec<u64>,
}

#[derive(Debug, Deserialize)]
pub struct GetMDMConfigProfileStatusParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ResendHostMDMProfileParams {}

#[derive(Debug, Deserialize)]
pub struct MdmAppleEnrollParams {
    pub token: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers - Apple MDM Setup
// ---------------------------------------------------------------------------

/// PATCH /api/_version_/fleet/mdm/apple/setup
/// PATCH /api/_version_/fleet/setup_experience
pub async fn update_mdm_apple_setup(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(_body): Json<UpdateMDMAppleSetupBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

// ---------------------------------------------------------------------------
// Handlers - Apple MDM Commands (deprecated Apple-specific paths)
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/apple/enqueue (deprecated)
pub async fn enqueue_mdm_apple_command(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<EnqueueMDMAppleCommandBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/mdm/apple/commandresults (deprecated)
pub async fn get_mdm_apple_command_results(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<GetMDMAppleCommandResultsParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_mdm_command_results(&viewer, &params.command_uuid).await {
        Ok(results) => fleet_ok("results", serde_json::to_value(&results).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/mdm/apple/commands (deprecated)
pub async fn list_mdm_apple_commands(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListMDMCommandsParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let page = params.page.unwrap_or(0) as u32;
    let per_page = params.per_page.unwrap_or(20) as u32;
    match state.service.list_mdm_commands(&viewer, page, per_page).await {
        Ok(commands) => fleet_ok("commands", serde_json::to_value(&commands).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

// ---------------------------------------------------------------------------
// Handlers - Apple MDM Config Profiles (deprecated Apple-specific paths)
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/mdm/apple/profiles/{profile_id} (deprecated)
pub async fn get_mdm_apple_config_profile(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(profile_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Legacy endpoint used numeric IDs; convert to UUID format
    let profile_uuid = format!("a{}", profile_id);
    match state.service.get_mdm_config_profile(&viewer, &profile_uuid).await {
        Ok(profile) => fleet_ok("profile", serde_json::to_value(&profile).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/mdm/apple/profiles/{profile_id} (deprecated)
pub async fn delete_mdm_apple_config_profile(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(profile_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let profile_uuid = format!("a{}", profile_id);
    match state.service.delete_mdm_config_profile(&viewer, &profile_uuid).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/mdm/apple/profiles (deprecated)
pub async fn new_mdm_apple_config_profile(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/mdm/apple/profiles (deprecated)
pub async fn list_mdm_apple_config_profiles(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListMDMConfigProfilesParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = params.team_id.map(|id| id as u32);
    let page = params.page.unwrap_or(0) as u32;
    let per_page = params.per_page.unwrap_or(20) as u32;
    match state.service.list_mdm_config_profiles(&viewer, team_id, page, per_page).await {
        Ok(profiles) => fleet_ok("profiles", serde_json::to_value(&profiles).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/mdm/apple/filevault/summary (deprecated)
pub async fn get_mdm_apple_filevault_summary(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<GetMDMAppleFileVaultSummaryParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = params.team_id.map(|id| id as u32);
    match state.service.get_mdm_apple_filevault_summary(&viewer, team_id).await {
        Ok(summary) => fleet_ok("", serde_json::to_value(&summary).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/mdm/apple/profiles/summary (deprecated)
pub async fn get_mdm_apple_profiles_summary(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<GetMDMAppleProfilesSummaryParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = params.team_id.map(|id| id as u32);
    match state.service.get_mdm_apple_profiles_summary(&viewer, team_id).await {
        Ok(summary) => fleet_ok("", serde_json::to_value(&summary).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

// ---------------------------------------------------------------------------
// Handlers - Setup Assistant / Enrollment Profiles
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/apple/enrollment_profile
/// POST /api/_version_/fleet/enrollment_profiles/automatic
pub async fn create_mdm_apple_setup_assistant(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/mdm/apple/enrollment_profile
/// GET /api/_version_/fleet/enrollment_profiles/automatic
pub async fn get_mdm_apple_setup_assistant(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// DELETE /api/_version_/fleet/mdm/apple/enrollment_profile
/// DELETE /api/_version_/fleet/enrollment_profiles/automatic
pub async fn delete_mdm_apple_setup_assistant(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

// ---------------------------------------------------------------------------
// Handlers - Apple Installers (old endpoints)
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/apple/installers
pub async fn upload_apple_installer(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/mdm/apple/installers/{installer_id}
pub async fn get_apple_installer(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(installer_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &installer_id);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// DELETE /api/_version_/fleet/mdm/apple/installers/{installer_id}
pub async fn delete_apple_installer(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(installer_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &installer_id);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/mdm/apple/installers
pub async fn list_mdm_apple_installers(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/mdm/apple/devices
pub async fn list_mdm_apple_devices(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

// ---------------------------------------------------------------------------
// Handlers - Manual Enrollment Profile
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/mdm/manual_enrollment_profile
/// GET /api/_version_/fleet/enrollment_profiles/manual
pub async fn get_manual_enrollment_profile(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

// ---------------------------------------------------------------------------
// Handlers - Bootstrap Package
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/bootstrap
/// POST /api/_version_/fleet/bootstrap
/// POST /api/_version_/fleet/mdm/apple/bootstrap
pub async fn upload_bootstrap_package(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/mdm/bootstrap/{fleet_id}/metadata
/// GET /api/_version_/fleet/bootstrap/{fleet_id}/metadata
/// GET /api/_version_/fleet/mdm/apple/bootstrap/{fleet_id}/metadata
pub async fn bootstrap_package_metadata(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(fleet_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &fleet_id);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// DELETE /api/_version_/fleet/mdm/bootstrap/{fleet_id}
/// DELETE /api/_version_/fleet/bootstrap/{fleet_id}
/// DELETE /api/_version_/fleet/mdm/apple/bootstrap/{fleet_id}
pub async fn delete_bootstrap_package(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(fleet_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &fleet_id);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/mdm/bootstrap/summary
/// GET /api/_version_/fleet/bootstrap/summary
/// GET /api/_version_/fleet/mdm/apple/bootstrap/summary
pub async fn get_bootstrap_package_summary(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<GetMDMAppleBootstrapPackageSummaryParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &params);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/mdm/bootstrap (unauthenticated download)
/// GET /api/_version_/fleet/bootstrap (unauthenticated download)
/// GET /api/_version_/fleet/mdm/apple/bootstrap (unauthenticated download)
pub async fn download_bootstrap_package(
    State(state): State<AppState>,
) -> FleetResponse {
    let _ = &state;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

// ---------------------------------------------------------------------------
// Handlers - Host MDM Actions
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/hosts/{id}/lock (deprecated)
pub async fn device_lock(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &id);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// POST /api/_version_/fleet/mdm/hosts/{id}/wipe
pub async fn device_wipe(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &id);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/mdm/hosts/{id}/profiles (deprecated)
/// GET /api/_version_/fleet/hosts/{id}/configuration_profiles
pub async fn get_host_profiles(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Need host UUID - look up the host first
    match state.service.get_host(&viewer, id as u32).await {
        Ok(host_detail) => {
            match state.service.get_host_mdm_profiles(&viewer, &host_detail.host.uuid).await {
                Ok(profiles) => fleet_ok("profiles", serde_json::to_value(&profiles).unwrap_or_default()),
                Err(e) => encode_service_error(&e),
            }
        }
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/mdm/apple (deprecated)
/// GET /api/_version_/fleet/apns
pub async fn get_apple_mdm(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

// ---------------------------------------------------------------------------
// Handlers - EULA
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/setup/eula
/// POST /api/_version_/fleet/setup_experience/eula
/// POST /api/_version_/fleet/mdm/apple/setup/eula (deprecated)
pub async fn create_mdm_eula(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Stub: MDM operations deferred
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/mdm/setup/eula/metadata
/// GET /api/_version_/fleet/setup_experience/eula/metadata
/// GET /api/_version_/fleet/mdm/apple/setup/eula/metadata (deprecated)
pub async fn get_mdm_eula_metadata(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// DELETE /api/_version_/fleet/mdm/setup/eula/{token}
/// DELETE /api/_version_/fleet/setup_experience/eula/{token}
/// DELETE /api/_version_/fleet/mdm/apple/setup/eula/{token} (deprecated)
pub async fn delete_mdm_eula(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(token): Path<String>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &token);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/mdm/setup/eula/{token} (unauthenticated)
/// GET /api/_version_/fleet/setup_experience/eula/{token} (unauthenticated)
/// GET /api/_version_/fleet/mdm/apple/setup/eula/{token} (unauthenticated, deprecated)
pub async fn get_mdm_eula(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> FleetResponse {
    // Stub: MDM operations deferred
    let _ = (&state, &token);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

// ---------------------------------------------------------------------------
// Handlers - MDM Preassign
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/apple/profiles/preassign
pub async fn preassign_mdm_apple_profile(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<PreassignMDMAppleProfileBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// POST /api/_version_/fleet/mdm/apple/profiles/match
pub async fn match_mdm_apple_preassignment(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<MatchMDMApplePreassignmentBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

// ---------------------------------------------------------------------------
// Handlers - Platform-Agnostic MDM Commands
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/commands/run (deprecated)
/// POST /api/_version_/fleet/commands/run
pub async fn run_mdm_command(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<RunMDMCommandBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/mdm/commandresults (deprecated)
/// GET /api/_version_/fleet/commands/results
pub async fn get_mdm_command_results(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<GetMDMCommandResultsParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_mdm_command_results(&viewer, &params.command_uuid).await {
        Ok(results) => fleet_ok("results", serde_json::to_value(&results).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/mdm/commands (deprecated)
/// GET /api/_version_/fleet/commands
pub async fn list_mdm_commands(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListMDMCommandsParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let page = params.page.unwrap_or(0) as u32;
    let per_page = params.per_page.unwrap_or(20) as u32;
    match state.service.list_mdm_commands(&viewer, page, per_page).await {
        Ok(commands) => fleet_ok("commands", serde_json::to_value(&commands).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/mdm/hosts/{id}/unenroll (deprecated)
/// DELETE /api/_version_/fleet/hosts/{id}/mdm
pub async fn mdm_unenroll(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &id);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

// ---------------------------------------------------------------------------
// Handlers - Disk Encryption
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/mdm/disk_encryption/summary (deprecated)
/// GET /api/_version_/fleet/disk_encryption
pub async fn get_mdm_disk_encryption_summary(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<GetMDMDiskEncryptionSummaryParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = params.team_id.map(|id| id as u32);
    match state.service.get_mdm_disk_encryption_summary(&viewer, team_id).await {
        Ok(summary) => fleet_ok("", serde_json::to_value(&summary).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/mdm/hosts/{id}/encryption_key (deprecated)
/// GET /api/_version_/fleet/hosts/{id}/encryption_key
pub async fn get_host_encryption_key(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &id);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// PATCH /api/_version_/fleet/mdm/apple/settings (deprecated)
pub async fn update_mdm_apple_settings(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<UpdateMDMAppleSettingsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// POST /api/_version_/fleet/disk_encryption
pub async fn update_disk_encryption(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<UpdateDiskEncryptionBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

// ---------------------------------------------------------------------------
// Handlers - Profile Summary
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/mdm/profiles/summary (deprecated)
/// GET /api/_version_/fleet/configuration_profiles/summary
pub async fn get_mdm_profiles_summary(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<GetMDMProfilesSummaryParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = params.team_id.map(|id| id as u32);
    match state.service.get_mdm_profiles_summary(&viewer, team_id).await {
        Ok(summary) => fleet_ok("", serde_json::to_value(&summary).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

// ---------------------------------------------------------------------------
// Handlers - Platform-Agnostic Configuration Profiles
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/mdm/profiles/{profile_uuid} (deprecated)
/// GET /api/_version_/fleet/configuration_profiles/{profile_uuid}
pub async fn get_mdm_config_profile(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(profile_uuid): Path<String>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_mdm_config_profile(&viewer, &profile_uuid).await {
        Ok(profile) => fleet_ok("profile", serde_json::to_value(&profile).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/mdm/profiles/{profile_uuid} (deprecated)
/// DELETE /api/_version_/fleet/configuration_profiles/{profile_uuid}
pub async fn delete_mdm_config_profile(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(profile_uuid): Path<String>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_mdm_config_profile(&viewer, &profile_uuid).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/mdm/profiles (deprecated)
/// GET /api/_version_/fleet/configuration_profiles
pub async fn list_mdm_config_profiles(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListMDMConfigProfilesParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = params.team_id.map(|id| id as u32);
    let page = params.page.unwrap_or(0) as u32;
    let per_page = params.per_page.unwrap_or(20) as u32;
    match state.service.list_mdm_config_profiles(&viewer, team_id, page, per_page).await {
        Ok(profiles) => fleet_ok("profiles", serde_json::to_value(&profiles).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/mdm/profiles (deprecated)
/// POST /api/_version_/fleet/configuration_profiles
pub async fn new_mdm_config_profile(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// POST /api/_version_/fleet/configuration_profiles/batch
pub async fn batch_modify_mdm_config_profiles(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<BatchModifyMDMConfigProfilesBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// POST /api/_version_/fleet/hosts/{host_id}/configuration_profiles/resend/{profile_uuid} (deprecated)
/// POST /api/_version_/fleet/hosts/{host_id}/configuration_profiles/{profile_uuid}/resend
pub async fn resend_host_mdm_profile(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path((host_id, profile_uuid)): Path<(u64, String)>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &host_id, &profile_uuid);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// POST /api/_version_/fleet/configuration_profiles/resend/batch
pub async fn batch_resend_mdm_profile_to_hosts(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<BatchResendMDMProfileToHostsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/configuration_profiles/{profile_uuid}/status
pub async fn get_mdm_config_profile_status(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(profile_uuid): Path<String>,
    Query(params): Query<GetMDMConfigProfileStatusParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let page = params.page.unwrap_or(0) as u32;
    let per_page = params.per_page.unwrap_or(20) as u32;
    match state.service.get_mdm_config_profile_status(&viewer, &profile_uuid, page, per_page).await {
        Ok(status) => fleet_ok("", serde_json::to_value(&status).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

// ---------------------------------------------------------------------------
// Handlers - CSR / APNs / DEP
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/apple/request_csr (deprecated)
pub async fn request_mdm_apple_csr(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<RequestMDMAppleCSRBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/mdm/apple/request_csr
pub async fn get_mdm_apple_csr(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// POST /api/_version_/fleet/mdm/apple/dep/key_pair (deprecated)
pub async fn new_mdm_apple_dep_key_pair(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/mdm/apple/abm_public_key
pub async fn generate_abm_key_pair(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// POST /api/_version_/fleet/mdm/apple/apns_certificate
pub async fn upload_mdm_apple_apns_cert(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Stub: multipart upload deferred
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// DELETE /api/_version_/fleet/mdm/apple/apns_certificate
pub async fn delete_mdm_apple_apns_cert(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

// ---------------------------------------------------------------------------
// Handlers - ABM Tokens
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/abm_tokens
pub async fn upload_abm_token(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Stub: multipart upload deferred
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// DELETE /api/_version_/fleet/abm_tokens/{id}
pub async fn delete_abm_token(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &id);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/abm_tokens
pub async fn list_abm_tokens(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/abm_tokens/count
pub async fn count_abm_tokens(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// PATCH /api/_version_/fleet/abm_tokens/{id}/fleets
pub async fn update_abm_token_teams(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<UpdateABMTokenTeamsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &id, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// PATCH /api/_version_/fleet/abm_tokens/{id}/renew
pub async fn renew_abm_token(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &id);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

// ---------------------------------------------------------------------------
// Handlers - VPP Tokens
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/vpp_tokens
pub async fn get_vpp_tokens(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// POST /api/_version_/fleet/vpp_tokens
pub async fn upload_vpp_token(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Stub: multipart upload deferred
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// PATCH /api/_version_/fleet/vpp_tokens/{id}/fleets
pub async fn patch_vpp_tokens_teams(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<PatchVPPTokensTeamsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &id, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// PATCH /api/_version_/fleet/vpp_tokens/{id}/renew
pub async fn patch_vpp_token_renew(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &id);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// DELETE /api/_version_/fleet/vpp_tokens/{id}
pub async fn delete_vpp_token(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &id);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

// ---------------------------------------------------------------------------
// Handlers - ABM (deprecated)
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/mdm/apple_bm (deprecated)
/// GET /api/_version_/fleet/abm (deprecated)
pub async fn get_apple_bm(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

// ---------------------------------------------------------------------------
// Handlers - Batch profile operations (deprecated Apple-specific)
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/apple/profiles/batch (deprecated)
pub async fn batch_set_mdm_apple_profiles(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<BatchSetMDMProfilesBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// POST /api/_version_/fleet/mdm/profiles/batch
pub async fn batch_set_mdm_profiles(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<BatchSetMDMProfilesBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

// ---------------------------------------------------------------------------
// Handlers - MDM SSO
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/sso
pub async fn initiate_mdm_sso(
    State(state): State<AppState>,
    Json(body): Json<InitiateMDMSSOBody>,
) -> FleetResponse {
    let _ = (&state, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// POST /api/_version_/fleet/mdm/sso/callback
pub async fn callback_mdm_sso(
    State(state): State<AppState>,
    Json(body): Json<CallbackMDMSSOBody>,
) -> FleetResponse {
    let _ = (&state, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

// ---------------------------------------------------------------------------
// Handlers - Apple MDM Enrollment (unauthenticated/token-auth)
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/ota_enrollment
pub async fn mdm_apple_ota(
    State(state): State<AppState>,
) -> FleetResponse {
    let _ = &state;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// GET /api/_version_/fleet/enrollment_profiles/ota
pub async fn get_ota_profile(
    State(state): State<AppState>,
) -> FleetResponse {
    let _ = &state;
    // Stub: mobileconfig binary deferred
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}
