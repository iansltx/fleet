//! MDM (Mobile Device Management) endpoints.
//!
//! Handles Apple MDM, Windows MDM, configuration profiles, bootstrap packages,
//! EULA, enrollment, ABM/VPP tokens, disk encryption, and device management.
//! Many of these endpoints have deprecated paths that are maintained for
//! backwards compatibility.

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::response::{fleet_error, fleet_ok, FleetResponse};
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
    State(_state): State<AppState>,
    Json(_body): Json<UpdateMDMAppleSetupBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - Apple MDM Commands (deprecated Apple-specific paths)
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/apple/enqueue (deprecated)
pub async fn enqueue_mdm_apple_command(
    State(_state): State<AppState>,
    Json(_body): Json<EnqueueMDMAppleCommandBody>,
) -> FleetResponse {
    fleet_ok("command_uuid", serde_json::json!(""))
}

/// GET /api/_version_/fleet/mdm/apple/commandresults (deprecated)
pub async fn get_mdm_apple_command_results(
    State(_state): State<AppState>,
    Query(_params): Query<GetMDMAppleCommandResultsParams>,
) -> FleetResponse {
    fleet_ok("results", serde_json::json!([]))
}

/// GET /api/_version_/fleet/mdm/apple/commands (deprecated)
pub async fn list_mdm_apple_commands(
    State(_state): State<AppState>,
    Query(_params): Query<ListMDMCommandsParams>,
) -> FleetResponse {
    fleet_ok("commands", serde_json::json!([]))
}

// ---------------------------------------------------------------------------
// Handlers - Apple MDM Config Profiles (deprecated Apple-specific paths)
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/mdm/apple/profiles/{profile_id} (deprecated)
pub async fn get_mdm_apple_config_profile(
    State(_state): State<AppState>,
    Path(_profile_id): Path<u64>,
) -> FleetResponse {
    // TODO: return profile binary
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/mdm/apple/profiles/{profile_id} (deprecated)
pub async fn delete_mdm_apple_config_profile(
    State(_state): State<AppState>,
    Path(_profile_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/mdm/apple/profiles (deprecated)
pub async fn new_mdm_apple_config_profile(
    State(_state): State<AppState>,
) -> FleetResponse {
    // TODO: multipart upload
    fleet_ok("profile_id", serde_json::json!(0))
}

/// GET /api/_version_/fleet/mdm/apple/profiles (deprecated)
pub async fn list_mdm_apple_config_profiles(
    State(_state): State<AppState>,
    Query(_params): Query<ListMDMConfigProfilesParams>,
) -> FleetResponse {
    fleet_ok("profiles", serde_json::json!([]))
}

/// GET /api/_version_/fleet/mdm/apple/filevault/summary (deprecated)
pub async fn get_mdm_apple_filevault_summary(
    State(_state): State<AppState>,
    Query(_params): Query<GetMDMAppleFileVaultSummaryParams>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/mdm/apple/profiles/summary (deprecated)
pub async fn get_mdm_apple_profiles_summary(
    State(_state): State<AppState>,
    Query(_params): Query<GetMDMAppleProfilesSummaryParams>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - Setup Assistant / Enrollment Profiles
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/apple/enrollment_profile
/// POST /api/_version_/fleet/enrollment_profiles/automatic
pub async fn create_mdm_apple_setup_assistant(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/mdm/apple/enrollment_profile
/// GET /api/_version_/fleet/enrollment_profiles/automatic
pub async fn get_mdm_apple_setup_assistant(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/mdm/apple/enrollment_profile
/// DELETE /api/_version_/fleet/enrollment_profiles/automatic
pub async fn delete_mdm_apple_setup_assistant(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - Apple Installers (old endpoints)
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/apple/installers
pub async fn upload_apple_installer(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("installer_id", serde_json::json!(0))
}

/// GET /api/_version_/fleet/mdm/apple/installers/{installer_id}
pub async fn get_apple_installer(
    State(_state): State<AppState>,
    Path(_installer_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/mdm/apple/installers/{installer_id}
pub async fn delete_apple_installer(
    State(_state): State<AppState>,
    Path(_installer_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/mdm/apple/installers
pub async fn list_mdm_apple_installers(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("installers", serde_json::json!([]))
}

/// GET /api/_version_/fleet/mdm/apple/devices
pub async fn list_mdm_apple_devices(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("devices", serde_json::json!([]))
}

// ---------------------------------------------------------------------------
// Handlers - Manual Enrollment Profile
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/mdm/manual_enrollment_profile
/// GET /api/_version_/fleet/enrollment_profiles/manual
pub async fn get_manual_enrollment_profile(
    State(_state): State<AppState>,
) -> FleetResponse {
    // TODO: return .mobileconfig binary
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - Bootstrap Package
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/bootstrap
/// POST /api/_version_/fleet/bootstrap
/// POST /api/_version_/fleet/mdm/apple/bootstrap
pub async fn upload_bootstrap_package(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/mdm/bootstrap/{fleet_id}/metadata
/// GET /api/_version_/fleet/bootstrap/{fleet_id}/metadata
/// GET /api/_version_/fleet/mdm/apple/bootstrap/{fleet_id}/metadata
pub async fn bootstrap_package_metadata(
    State(_state): State<AppState>,
    Path(_fleet_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/mdm/bootstrap/{fleet_id}
/// DELETE /api/_version_/fleet/bootstrap/{fleet_id}
/// DELETE /api/_version_/fleet/mdm/apple/bootstrap/{fleet_id}
pub async fn delete_bootstrap_package(
    State(_state): State<AppState>,
    Path(_fleet_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/mdm/bootstrap/summary
/// GET /api/_version_/fleet/bootstrap/summary
/// GET /api/_version_/fleet/mdm/apple/bootstrap/summary
pub async fn get_bootstrap_package_summary(
    State(_state): State<AppState>,
    Query(_params): Query<GetMDMAppleBootstrapPackageSummaryParams>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/mdm/bootstrap (unauthenticated download)
/// GET /api/_version_/fleet/bootstrap (unauthenticated download)
/// GET /api/_version_/fleet/mdm/apple/bootstrap (unauthenticated download)
pub async fn download_bootstrap_package(
    State(_state): State<AppState>,
) -> FleetResponse {
    // TODO: return binary
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - Host MDM Actions
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/hosts/{id}/lock (deprecated)
pub async fn device_lock(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/mdm/hosts/{id}/wipe
pub async fn device_wipe(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/mdm/hosts/{id}/profiles (deprecated)
/// GET /api/_version_/fleet/hosts/{id}/configuration_profiles
pub async fn get_host_profiles(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("profiles", serde_json::json!([]))
}

/// GET /api/_version_/fleet/mdm/apple (deprecated)
/// GET /api/_version_/fleet/apns
pub async fn get_apple_mdm(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - EULA
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/setup/eula
/// POST /api/_version_/fleet/setup_experience/eula
/// POST /api/_version_/fleet/mdm/apple/setup/eula (deprecated)
pub async fn create_mdm_eula(
    State(_state): State<AppState>,
) -> FleetResponse {
    // TODO: multipart upload
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/mdm/setup/eula/metadata
/// GET /api/_version_/fleet/setup_experience/eula/metadata
/// GET /api/_version_/fleet/mdm/apple/setup/eula/metadata (deprecated)
pub async fn get_mdm_eula_metadata(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/mdm/setup/eula/{token}
/// DELETE /api/_version_/fleet/setup_experience/eula/{token}
/// DELETE /api/_version_/fleet/mdm/apple/setup/eula/{token} (deprecated)
pub async fn delete_mdm_eula(
    State(_state): State<AppState>,
    Path(_token): Path<String>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/mdm/setup/eula/{token} (unauthenticated)
/// GET /api/_version_/fleet/setup_experience/eula/{token} (unauthenticated)
/// GET /api/_version_/fleet/mdm/apple/setup/eula/{token} (unauthenticated, deprecated)
pub async fn get_mdm_eula(
    State(_state): State<AppState>,
    Path(_token): Path<String>,
) -> FleetResponse {
    // TODO: return EULA binary
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - MDM Preassign
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/apple/profiles/preassign
pub async fn preassign_mdm_apple_profile(
    State(_state): State<AppState>,
    Json(_body): Json<PreassignMDMAppleProfileBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/mdm/apple/profiles/match
pub async fn match_mdm_apple_preassignment(
    State(_state): State<AppState>,
    Json(_body): Json<MatchMDMApplePreassignmentBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - Platform-Agnostic MDM Commands
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/commands/run (deprecated)
/// POST /api/_version_/fleet/commands/run
pub async fn run_mdm_command(
    State(_state): State<AppState>,
    Json(_body): Json<RunMDMCommandBody>,
) -> FleetResponse {
    fleet_ok("command_uuid", serde_json::json!(""))
}

/// GET /api/_version_/fleet/mdm/commandresults (deprecated)
/// GET /api/_version_/fleet/commands/results
pub async fn get_mdm_command_results(
    State(_state): State<AppState>,
    Query(_params): Query<GetMDMCommandResultsParams>,
) -> FleetResponse {
    fleet_ok("results", serde_json::json!([]))
}

/// GET /api/_version_/fleet/mdm/commands (deprecated)
/// GET /api/_version_/fleet/commands
pub async fn list_mdm_commands(
    State(_state): State<AppState>,
    Query(_params): Query<ListMDMCommandsParams>,
) -> FleetResponse {
    fleet_ok("commands", serde_json::json!([]))
}

/// PATCH /api/_version_/fleet/mdm/hosts/{id}/unenroll (deprecated)
/// DELETE /api/_version_/fleet/hosts/{id}/mdm
pub async fn mdm_unenroll(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - Disk Encryption
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/mdm/disk_encryption/summary (deprecated)
/// GET /api/_version_/fleet/disk_encryption
pub async fn get_mdm_disk_encryption_summary(
    State(_state): State<AppState>,
    Query(_params): Query<GetMDMDiskEncryptionSummaryParams>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/mdm/hosts/{id}/encryption_key (deprecated)
/// GET /api/_version_/fleet/hosts/{id}/encryption_key
pub async fn get_host_encryption_key(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("encryption_key", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/mdm/apple/settings (deprecated)
pub async fn update_mdm_apple_settings(
    State(_state): State<AppState>,
    Json(_body): Json<UpdateMDMAppleSettingsBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/disk_encryption
pub async fn update_disk_encryption(
    State(_state): State<AppState>,
    Json(_body): Json<UpdateDiskEncryptionBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - Profile Summary
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/mdm/profiles/summary (deprecated)
/// GET /api/_version_/fleet/configuration_profiles/summary
pub async fn get_mdm_profiles_summary(
    State(_state): State<AppState>,
    Query(_params): Query<GetMDMProfilesSummaryParams>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - Platform-Agnostic Configuration Profiles
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/mdm/profiles/{profile_uuid} (deprecated)
/// GET /api/_version_/fleet/configuration_profiles/{profile_uuid}
pub async fn get_mdm_config_profile(
    State(_state): State<AppState>,
    Path(_profile_uuid): Path<String>,
) -> FleetResponse {
    // TODO: return profile binary
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/mdm/profiles/{profile_uuid} (deprecated)
/// DELETE /api/_version_/fleet/configuration_profiles/{profile_uuid}
pub async fn delete_mdm_config_profile(
    State(_state): State<AppState>,
    Path(_profile_uuid): Path<String>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/mdm/profiles (deprecated)
/// GET /api/_version_/fleet/configuration_profiles
pub async fn list_mdm_config_profiles(
    State(_state): State<AppState>,
    Query(_params): Query<ListMDMConfigProfilesParams>,
) -> FleetResponse {
    fleet_ok("profiles", serde_json::json!([]))
}

/// POST /api/_version_/fleet/mdm/profiles (deprecated)
/// POST /api/_version_/fleet/configuration_profiles
pub async fn new_mdm_config_profile(
    State(_state): State<AppState>,
) -> FleetResponse {
    // TODO: multipart upload
    fleet_ok("profile_uuid", serde_json::json!(""))
}

/// POST /api/_version_/fleet/configuration_profiles/batch
pub async fn batch_modify_mdm_config_profiles(
    State(_state): State<AppState>,
    Json(_body): Json<BatchModifyMDMConfigProfilesBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/hosts/{host_id}/configuration_profiles/resend/{profile_uuid} (deprecated)
/// POST /api/_version_/fleet/hosts/{host_id}/configuration_profiles/{profile_uuid}/resend
pub async fn resend_host_mdm_profile(
    State(_state): State<AppState>,
    Path((_host_id, _profile_uuid)): Path<(u64, String)>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/configuration_profiles/resend/batch
pub async fn batch_resend_mdm_profile_to_hosts(
    State(_state): State<AppState>,
    Json(_body): Json<BatchResendMDMProfileToHostsBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/configuration_profiles/{profile_uuid}/status
pub async fn get_mdm_config_profile_status(
    State(_state): State<AppState>,
    Path(_profile_uuid): Path<String>,
    Query(_params): Query<GetMDMConfigProfileStatusParams>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - CSR / APNs / DEP
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/apple/request_csr (deprecated)
pub async fn request_mdm_apple_csr(
    State(_state): State<AppState>,
    Json(_body): Json<RequestMDMAppleCSRBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/mdm/apple/request_csr
pub async fn get_mdm_apple_csr(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/mdm/apple/dep/key_pair (deprecated)
pub async fn new_mdm_apple_dep_key_pair(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/mdm/apple/abm_public_key
pub async fn generate_abm_key_pair(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/mdm/apple/apns_certificate
pub async fn upload_mdm_apple_apns_cert(
    State(_state): State<AppState>,
) -> FleetResponse {
    // TODO: multipart upload
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/mdm/apple/apns_certificate
pub async fn delete_mdm_apple_apns_cert(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - ABM Tokens
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/abm_tokens
pub async fn upload_abm_token(
    State(_state): State<AppState>,
) -> FleetResponse {
    // TODO: multipart upload
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/abm_tokens/{id}
pub async fn delete_abm_token(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/abm_tokens
pub async fn list_abm_tokens(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("abm_tokens", serde_json::json!([]))
}

/// GET /api/_version_/fleet/abm_tokens/count
pub async fn count_abm_tokens(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("count", serde_json::json!(0))
}

/// PATCH /api/_version_/fleet/abm_tokens/{id}/fleets
pub async fn update_abm_token_teams(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
    Json(_body): Json<UpdateABMTokenTeamsBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/abm_tokens/{id}/renew
pub async fn renew_abm_token(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - VPP Tokens
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/vpp_tokens
pub async fn get_vpp_tokens(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("vpp_tokens", serde_json::json!([]))
}

/// POST /api/_version_/fleet/vpp_tokens
pub async fn upload_vpp_token(
    State(_state): State<AppState>,
) -> FleetResponse {
    // TODO: multipart upload
    fleet_ok("", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/vpp_tokens/{id}/fleets
pub async fn patch_vpp_tokens_teams(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
    Json(_body): Json<PatchVPPTokensTeamsBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/vpp_tokens/{id}/renew
pub async fn patch_vpp_token_renew(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/vpp_tokens/{id}
pub async fn delete_vpp_token(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - ABM (deprecated)
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/mdm/apple_bm (deprecated)
/// GET /api/_version_/fleet/abm (deprecated)
pub async fn get_apple_bm(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - Batch profile operations (deprecated Apple-specific)
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/apple/profiles/batch (deprecated)
pub async fn batch_set_mdm_apple_profiles(
    State(_state): State<AppState>,
    Json(_body): Json<BatchSetMDMProfilesBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/mdm/profiles/batch
pub async fn batch_set_mdm_profiles(
    State(_state): State<AppState>,
    Json(_body): Json<BatchSetMDMProfilesBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - MDM SSO
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/mdm/sso
pub async fn initiate_mdm_sso(
    State(_state): State<AppState>,
    Json(_body): Json<InitiateMDMSSOBody>,
) -> FleetResponse {
    fleet_ok("url", serde_json::json!(""))
}

/// POST /api/_version_/fleet/mdm/sso/callback
pub async fn callback_mdm_sso(
    State(_state): State<AppState>,
    Json(_body): Json<CallbackMDMSSOBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

// ---------------------------------------------------------------------------
// Handlers - Apple MDM Enrollment (unauthenticated/token-auth)
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/ota_enrollment
pub async fn mdm_apple_ota(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/enrollment_profiles/ota
pub async fn get_ota_profile(
    State(_state): State<AppState>,
) -> FleetResponse {
    // TODO: return .mobileconfig binary
    fleet_ok("", serde_json::json!({}))
}
