//! MDM (Mobile Device Management) types.
//!
//! Core data types for MDM configuration profiles, commands, and summaries.
//! Ported from Go types in `server/fleet/mdm.go`, `server/fleet/apple_mdm.go`,
//! `server/fleet/microsoft_mdm.go`, `server/fleet/windows_mdm.go`, and
//! `server/fleet/linux_mdm.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===========================================================================
// Constants from mdm.go
// ===========================================================================

pub const MDM_PLATFORM_APPLE: &str = "apple";
pub const MDM_PLATFORM_MICROSOFT: &str = "microsoft";

pub const MDM_APPLE_DECLARATION_UUID_PREFIX: &str = "d";
pub const MDM_APPLE_PROFILE_UUID_PREFIX: &str = "a";
pub const MDM_WINDOWS_PROFILE_UUID_PREFIX: &str = "w";
pub const MDM_ANDROID_PROFILE_UUID_PREFIX: &str = "g";

pub const STICKY_MDM_ENROLLMENT_KEY_PREFIX: &str = "sticky_mdm_enrollment_";

/// FleetVarName represents the name of a Fleet variable (without the FLEET_VAR_ prefix).
pub type FleetVarName = String;

// Fleet variable name constants
pub const FLEET_VAR_HOST_END_USER_EMAIL_IDP: &str = "HOST_END_USER_EMAIL_IDP";
pub const FLEET_VAR_HOST_HARDWARE_SERIAL: &str = "HOST_HARDWARE_SERIAL";
pub const FLEET_VAR_HOST_END_USER_IDP_USERNAME: &str = "HOST_END_USER_IDP_USERNAME";
pub const FLEET_VAR_HOST_END_USER_IDP_USERNAME_LOCAL_PART: &str =
    "HOST_END_USER_IDP_USERNAME_LOCAL_PART";
pub const FLEET_VAR_HOST_END_USER_IDP_GROUPS: &str = "HOST_END_USER_IDP_GROUPS";
pub const FLEET_VAR_HOST_END_USER_IDP_DEPARTMENT: &str = "HOST_END_USER_IDP_DEPARTMENT";
pub const FLEET_VAR_HOST_END_USER_IDP_FULLNAME: &str = "HOST_END_USER_IDP_FULL_NAME";
pub const FLEET_VAR_HOST_UUID: &str = "HOST_UUID";
pub const FLEET_VAR_HOST_PLATFORM: &str = "HOST_PLATFORM";
pub const FLEET_VAR_NDES_SCEP_CHALLENGE: &str = "NDES_SCEP_CHALLENGE";
pub const FLEET_VAR_NDES_SCEP_PROXY_URL: &str = "NDES_SCEP_PROXY_URL";
pub const FLEET_VAR_SCEP_RENEWAL_ID: &str = "SCEP_RENEWAL_ID";
pub const FLEET_VAR_DIGICERT_DATA_PREFIX: &str = "DIGICERT_DATA_";
pub const FLEET_VAR_DIGICERT_PASSWORD_PREFIX: &str = "DIGICERT_PASSWORD_";
pub const FLEET_VAR_CUSTOM_SCEP_CHALLENGE_PREFIX: &str = "CUSTOM_SCEP_CHALLENGE_";
pub const FLEET_VAR_CUSTOM_SCEP_PROXY_URL_PREFIX: &str = "CUSTOM_SCEP_PROXY_URL_";
pub const FLEET_VAR_SMALLSTEP_SCEP_CHALLENGE_PREFIX: &str = "SMALLSTEP_SCEP_CHALLENGE_";
pub const FLEET_VAR_SMALLSTEP_SCEP_PROXY_URL_PREFIX: &str = "SMALLSTEP_SCEP_PROXY_URL_";
pub const FLEET_VAR_SCEP_WINDOWS_CERTIFICATE_ID: &str = "SCEP_WINDOWS_CERTIFICATE_ID";

// Refetch command UUID prefixes
pub const REFETCH_BASE_COMMAND_UUID_PREFIX: &str = "REFETCH-";
pub const REFETCH_DEVICE_COMMAND_UUID_PREFIX: &str = "REFETCH-DEVICE-";
pub const REFETCH_APPS_COMMAND_UUID_PREFIX: &str = "REFETCH-APPS-";
pub const REFETCH_CERTS_COMMAND_UUID_PREFIX: &str = "REFETCH-CERTS-";

pub const VERIFY_SOFTWARE_INSTALL_VPP_PREFIX: &str = "VERIFY-VPP-INSTALLS-";

pub const VPP_TIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

// ===========================================================================
// MDM Delivery Status & Operation Type (from mdm.go)
// ===========================================================================

/// MDMDeliveryStatus represents the delivery status of an MDM profile or command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MDMDeliveryStatus {
    Pending,
    Verifying,
    Verified,
    Failed,
}

impl std::fmt::Display for MDMDeliveryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MDMDeliveryStatus::Pending => write!(f, "pending"),
            MDMDeliveryStatus::Verifying => write!(f, "verifying"),
            MDMDeliveryStatus::Verified => write!(f, "verified"),
            MDMDeliveryStatus::Failed => write!(f, "failed"),
        }
    }
}

/// MDMOperationType represents the type of MDM operation (install or remove).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MDMOperationType {
    #[serde(rename = "install")]
    Install,
    #[serde(rename = "remove")]
    Remove,
}

impl std::fmt::Display for MDMOperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MDMOperationType::Install => write!(f, "install"),
            MDMOperationType::Remove => write!(f, "remove"),
        }
    }
}

/// MDMCommandStatusFilter for filtering MDM command lists.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MDMCommandStatusFilter {
    Pending,
    Ran,
    Failed,
}

// ===========================================================================
// MDM Config Profile Payload (platform-agnostic, from mdm.go)
// ===========================================================================

/// MDMConfigProfilePayload is the platform-agnostic profile struct returned
/// by listing endpoints. Matches Go's `fleet.MDMConfigProfilePayload`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMConfigProfilePayload {
    pub profile_uuid: String,
    pub team_id: Option<u32>,
    pub name: String,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
    /// Note: JSON field is `updated_at` for historical reasons (Go uses `uploaded_at` in DB).
    #[serde(rename = "updated_at")]
    pub uploaded_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub labels_include_all: Vec<ConfigurationProfileLabel>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub labels_include_any: Vec<ConfigurationProfileLabel>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub labels_exclude_any: Vec<ConfigurationProfileLabel>,
}

/// ConfigurationProfileLabel represents a label associated with a profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationProfileLabel {
    #[serde(skip_serializing)]
    pub profile_uuid: String,
    #[serde(rename = "name")]
    pub label_name: String,
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub label_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broken: Option<bool>,
    #[serde(skip_serializing)]
    pub exclude: bool,
    #[serde(skip_serializing)]
    pub require_all: bool,
}

/// BatchModifyMDMConfigProfilePayload represents the payload for a config profile
/// when performing a batch modify operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchModifyMDMConfigProfilePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels_include_all: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels_include_any: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels_exclude_any: Option<Vec<String>>,
}

/// MDMProfileBatchPayload represents the payload to batch-set profiles for a team or no-team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMProfileBatchPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contents: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels_include_all: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels_include_any: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels_exclude_any: Option<Vec<String>>,
    #[serde(skip)]
    pub secrets_updated_at: Option<DateTime<Utc>>,
}

/// MDMProfileSpec represents the spec used to define configuration profiles via yaml files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMProfileSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels_include_all: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels_include_any: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels_exclude_any: Option<Vec<String>>,
}

/// MDMLabelsMode represents how labels are used to filter profiles.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MDMLabelsMode {
    #[serde(rename = "labels_include_all")]
    LabelsIncludeAll,
    #[serde(rename = "labels_include_any")]
    LabelsIncludeAny,
    #[serde(rename = "labels_exclude_any")]
    LabelsExcludeAny,
}

// ===========================================================================
// MDM Profiles Summary (from mdm.go)
// ===========================================================================

/// MDMProfilesSummary reports the number of hosts being managed with configuration profiles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMProfilesSummary {
    pub verified: u32,
    pub verifying: u32,
    pub pending: u32,
    pub failed: u32,
}

// ===========================================================================
// MDM Command (from mdm.go)
// ===========================================================================

/// MDMCommand represents an MDM command that has been enqueued for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMCommand {
    pub host_uuid: String,
    pub command_uuid: String,
    pub updated_at: DateTime<Utc>,
    pub request_type: String,
    pub status: String,
    pub hostname: String,
    #[serde(skip)]
    pub team_id: Option<u32>,
    #[serde(default)]
    pub command_status: String,
}

/// MDMCommandListOptions defines the options to control the list of MDM commands to return.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMCommandListOptions {
    #[serde(flatten)]
    pub list_options: crate::ListOptions,
    #[serde(flatten)]
    pub filters: MDMCommandFilters,
}

/// MDMCommandFilters defines filters for listing MDM commands.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMCommandFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_statuses: Option<Vec<MDMCommandStatusFilter>>,
}

// ===========================================================================
// MDM Command Result (from mdm.go)
// ===========================================================================

/// MDMCommandResult contains the result of an MDM command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMCommandResult {
    pub host_uuid: String,
    pub command_uuid: String,
    pub status: String,
    pub updated_at: DateTime<Utc>,
    pub request_type: String,
    #[serde(default)]
    pub result: Vec<u8>,
    #[serde(default)]
    pub hostname: String,
    #[serde(default)]
    pub payload: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results_metadata: Option<HashMap<String, serde_json::Value>>,
}

/// CommandEnqueueResult is the result of a command execution on enrolled Apple devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEnqueueResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_uuids: Option<Vec<String>>,
    pub platform: String,
}

// ===========================================================================
// Host MDM Profile (from mdm.go)
// ===========================================================================

/// HostMDMProfile represents the status of an MDM profile on a specific host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMDMProfile {
    #[serde(skip)]
    pub host_uuid: String,
    #[serde(skip)]
    pub command_uuid: String,
    pub profile_uuid: String,
    pub name: String,
    #[serde(skip)]
    pub identifier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub operation_type: MDMOperationType,
    pub detail: String,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_local_account: Option<String>,
}

/// ExpectedMDMProfile represents an MDM profile that is expected to be installed on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedMDMProfile {
    pub profile_uuid: String,
    pub identifier: String,
    pub name: String,
    pub earliest_install_date: DateTime<Utc>,
    #[serde(skip)]
    pub raw_profile: Vec<u8>,
    pub count_profile_labels: u32,
    pub count_host_labels: u32,
    pub count_non_broken_labels: u32,
}

/// HostMDMProfileRetryCount represents the number of times Fleet has attempted to install
/// the identified profile on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMDMProfileRetryCount {
    pub profile_identifier: String,
    pub profile_name: String,
    pub retries: u32,
}

/// MDMConfigProfileStatus represents the number of hosts in each status for a
/// given configuration profile.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMConfigProfileStatus {
    pub verified: u32,
    pub verifying: u32,
    pub pending: u32,
    pub failed: u32,
}

// ===========================================================================
// MDM Disk Encryption Summary (from mdm.go)
// ===========================================================================

/// MDMDiskEncryptionSummary contains counts of hosts grouped by disk encryption status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMDiskEncryptionSummary {
    pub verified: MDMPlatformsCounts,
    pub verifying: MDMPlatformsCounts,
    pub action_required: MDMPlatformsCounts,
    pub enforcing: MDMPlatformsCounts,
    pub failed: MDMPlatformsCounts,
    pub removing_enforcement: MDMPlatformsCounts,
}

/// MDMPlatformsCounts contains per-platform host counts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMPlatformsCounts {
    pub macos: u32,
    pub windows: u32,
    pub linux: u32,
}

// ===========================================================================
// MDM FileVault Summary (from apple_mdm.go)
// ===========================================================================

/// MDMAppleFileVaultSummary contains FileVault-specific status counts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMAppleFileVaultSummary {
    pub verified: u32,
    pub verifying: u32,
    pub action_required: u32,
    pub enforcing: u32,
    pub failed: u32,
    pub removing_enforcement: u32,
}

/// MDMAppleBootstrapPackageSummary reports the number of hosts targeted to install
/// the MDM bootstrap package.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMAppleBootstrapPackageSummary {
    pub installed: u32,
    pub pending: u32,
    pub failed: u32,
}

// ===========================================================================
// MDM Asset Names (from mdm.go)
// ===========================================================================

/// MDMAssetName represents the name of an MDM configuration asset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MDMAssetName {
    #[serde(rename = "ca_cert")]
    CACert,
    #[serde(rename = "ca_key")]
    CAKey,
    #[serde(rename = "apns_key")]
    APNSKey,
    #[serde(rename = "apns_cert")]
    APNSCert,
    #[serde(rename = "abm_key")]
    ABMKey,
    #[serde(rename = "abm_cert")]
    ABMCert,
    #[serde(rename = "abm_token")]
    ABMTokenDeprecated,
    #[serde(rename = "scep_challenge")]
    SCEPChallenge,
    #[serde(rename = "vpp_token")]
    VPPTokenDeprecated,
    #[serde(rename = "ndes_password")]
    NDESPassword,
    #[serde(rename = "android_pubsub_token")]
    AndroidPubSubToken,
    #[serde(rename = "android_fleet_server_secret")]
    AndroidFleetServerSecret,
    #[serde(rename = "host_identity_ca_cert")]
    HostIdentityCACert,
    #[serde(rename = "host_identity_ca_key")]
    HostIdentityCAKey,
    #[serde(rename = "conditional_access_ca_cert")]
    ConditionalAccessCACert,
    #[serde(rename = "conditional_access_ca_key")]
    ConditionalAccessCAKey,
    #[serde(rename = "conditional_access_idp_cert")]
    ConditionalAccessIDPCert,
    #[serde(rename = "conditional_access_idp_key")]
    ConditionalAccessIDPKey,
    #[serde(rename = "vpp_proxy_bearer_token")]
    VPPProxyBearerToken,
}

/// MDMConfigAsset holds an MDM configuration asset (name, value, checksum).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMConfigAsset {
    pub name: MDMAssetName,
    #[serde(skip)]
    pub value: Vec<u8>,
    pub md5_checksum: String,
}

// ===========================================================================
// MDM Command Authorization (from mdm.go)
// ===========================================================================

/// MDMCommandAuthz is used to check user authorization to read/write an MDM command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMCommandAuthz {
    pub team_id: Option<u32>,
}

/// MDMConfigProfileAuthz is used to check user authorization to read/write an MDM config profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMConfigProfileAuthz {
    pub team_id: Option<u32>,
}

// ===========================================================================
// MDM Wipe Metadata (from mdm.go)
// ===========================================================================

/// MDMWipeMetadata specifies optional metadata for the remote wipe command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMWipeMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<MDMWindowsWipeMetadata>,
}

// ===========================================================================
// MDM IdP Account (from mdm.go)
// ===========================================================================

/// MDMIdPAccount contains account information of a third-party IdP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMIdPAccount {
    pub uuid: String,
    pub username: String,
    pub fullname: String,
    pub email: String,
}

// ===========================================================================
// Host MDM Identifiers (from mdm.go)
// ===========================================================================

/// HostMDMIdentifiers contains identifying information for an MDM host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMDMIdentifiers {
    pub id: u32,
    pub uuid: String,
    pub hardware_serial: String,
    pub hostname: String,
    pub platform: String,
    pub team_id: Option<u32>,
}

/// HostMDMCommand represents a host-level MDM command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMDMCommand {
    pub host_id: u32,
    pub command_type: String,
}

/// MDMProfileUUIDFleetVariables represents the Fleet variables used by a
/// profile identified by its UUID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMProfileUUIDFleetVariables {
    pub profile_uuid: String,
    pub fleet_variables: Vec<String>,
}

/// MDMProfileIdentifierFleetVariables represents the Fleet variables used by a
/// profile identified by its identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMProfileIdentifierFleetVariables {
    pub identifier: String,
    pub fleet_variables: Vec<String>,
}

/// BatchResendMDMProfileFilters represents the filters for batch-redelivery of an MDM profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResendMDMProfileFilters {
    pub profile_status: MDMDeliveryStatus,
}

// ===========================================================================
// General MDM types (from mdm.go)
// ===========================================================================

/// MDMPlatform represents a supported MDM platform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MDMPlatform {
    #[serde(rename = "darwin")]
    Apple,
    #[serde(rename = "windows")]
    Windows,
    #[serde(rename = "linux")]
    Linux,
}

/// MDMEULAPayload is the payload for MDM EULA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMEULAPayload {
    pub name: String,
    #[serde(skip)]
    pub bytes: Vec<u8>,
    pub token: String,
}

/// MDMEULA represents an EULA (End User License Agreement) file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMEULA {
    pub name: String,
    #[serde(skip)]
    pub bytes: Vec<u8>,
    pub sha256: Vec<u8>,
    pub token: String,
    pub created_at: DateTime<Utc>,
}

// ===========================================================================
// AppleMDM / AppleBM types (from mdm.go)
// ===========================================================================

/// AppleMDM contains Apple MDM certificate information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMDM {
    pub common_name: String,
    pub serial_number: String,
    pub issuer: String,
    pub renew_date: DateTime<Utc>,
}

/// AppleBM contains Apple Business Manager token information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleBM {
    pub apple_id: String,
    pub org_name: String,
    pub mdm_server_url: String,
    pub renew_date: DateTime<Utc>,
    pub default_team: String,
}

/// ABMToken represents an Apple Business Manager token record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABMToken {
    pub id: u32,
    pub apple_id: String,
    #[serde(rename = "org_name")]
    pub organization_name: String,
    #[serde(rename = "renew_date")]
    pub renew_at: DateTime<Utc>,
    pub terms_expired: bool,
    #[serde(skip)]
    pub macos_default_team_id: Option<u32>,
    #[serde(skip)]
    pub ios_default_team_id: Option<u32>,
    #[serde(skip)]
    pub ipados_default_team_id: Option<u32>,
    #[serde(skip)]
    pub encrypted_token: Vec<u8>,
    pub mdm_server_url: String,
    #[serde(skip)]
    pub macos_team_name: String,
    #[serde(skip)]
    pub ios_team_name: String,
    #[serde(skip)]
    pub ipados_team_name: String,
    pub macos_team: ABMTokenTeam,
    pub ios_team: ABMTokenTeam,
    pub ipados_team: ABMTokenTeam,
}

/// ABMTokenTeam represents a team associated with an ABM token.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ABMTokenTeam {
    pub name: String,
    pub team_id: u32,
}

/// AppleCSR contains the keys/certs for an Apple Certificate Signing Request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleCSR {
    pub apns_key: Vec<u8>,
    pub scep_cert: Vec<u8>,
    pub scep_key: Vec<u8>,
}

// ===========================================================================
// VPP types (from mdm.go)
// ===========================================================================

/// VPPTokenInfo is the representation of the VPP token that we send out via API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPTokenInfo {
    pub org_name: String,
    pub renew_date: String,
    pub location: String,
}

/// VPPTokenRaw is the representation of the decoded JSON object downloaded from ABM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPTokenRaw {
    #[serde(rename = "orgName")]
    pub org_name: String,
    pub token: String,
    #[serde(rename = "expDate")]
    pub exp_date: String,
}

/// VPPTokenData is the VPP data stored in the DB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPTokenData {
    pub location: String,
    pub token: String,
}

/// VPPTokenDB represents a VPP token record in the DB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPTokenDB {
    pub id: u32,
    #[serde(rename = "org_name")]
    pub organization_name: String,
    pub location: String,
    pub renew_date: DateTime<Utc>,
    #[serde(skip)]
    pub token: String,
    pub teams: Vec<TeamTuple>,
}

/// TeamTuple represents a team ID and name pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamTuple {
    pub team_id: u32,
    pub name: String,
}

/// NullTeamType represents the state of a VPP token's team assignment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NullTeamType {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "allteams")]
    AllTeams,
    #[serde(rename = "noteam")]
    NoTeam,
}

// ===========================================================================
// Apple Device types (from mdm.go)
// ===========================================================================

/// AppleDevice represents the type of Apple device.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AppleDevice {
    MacOS = 0,
    IOS = 1,
    IPadOS = 2,
}

/// InstallableDevicePlatform represents a platform that can install apps.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InstallableDevicePlatform {
    #[serde(rename = "darwin")]
    MacOS,
    #[serde(rename = "ios")]
    IOS,
    #[serde(rename = "ipados")]
    IPadOS,
    #[serde(rename = "android")]
    Android,
}

/// AppleDevicesToRefetch represents hosts that need to be refetched.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleDevicesToRefetch {
    pub host_id: u32,
    pub uuid: String,
    pub installed_from_dep: bool,
    pub commands_already_sent: Vec<String>,
}

// ===========================================================================
// MDM Profiles Updates (from apple_mdm.go)
// ===========================================================================

/// MDMProfilesUpdates flags updates that were done during batch processing of profiles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMProfilesUpdates {
    pub apple_config_profile: bool,
    pub windows_config_profile: bool,
    pub apple_declaration: bool,
    pub android_config_profile: bool,
}

// ===========================================================================
// Apple MDM types (from apple_mdm.go)
// ===========================================================================

// Well-known Apple MDM status responses
pub const MDM_APPLE_STATUS_ACKNOWLEDGED: &str = "Acknowledged";
pub const MDM_APPLE_STATUS_ERROR: &str = "Error";
pub const MDM_APPLE_STATUS_COMMAND_FORMAT_ERROR: &str = "CommandFormatError";
pub const MDM_APPLE_STATUS_IDLE: &str = "Idle";
pub const MDM_APPLE_STATUS_NOT_NOW: &str = "NotNow";

/// MDMAppleEnrollmentType is the type for Apple MDM enrollments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MDMAppleEnrollmentType {
    #[serde(rename = "automatic")]
    Automatic,
    #[serde(rename = "manual")]
    Manual,
}

/// MDMAppleEnrollmentProfilePayload contains the data necessary to create
/// an enrollment profile in Fleet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleEnrollmentProfilePayload {
    #[serde(rename = "type")]
    pub enrollment_type: MDMAppleEnrollmentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dep_profile: Option<serde_json::Value>,
    #[serde(skip)]
    pub token: String,
}

/// MDMAppleEnrollmentProfile represents an Apple MDM enrollment profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleEnrollmentProfile {
    pub id: u32,
    pub token: String,
    #[serde(rename = "type")]
    pub enrollment_type: MDMAppleEnrollmentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dep_profile: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrollment_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// MDMAppleDEPKeyPair contains the DEP public key certificate and private key pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDEPKeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

/// MDMAppleInstaller holds installer packages for Apple devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleInstaller {
    pub id: u32,
    pub name: String,
    pub size: i64,
    pub manifest: String,
    #[serde(skip)]
    pub installer: Vec<u8>,
    pub url_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// MDMAppleDevice represents an MDM enrolled Apple device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDevice {
    pub id: String,
    pub serial_number: String,
    pub enabled: bool,
}

/// MDMAppleCommand represents an Apple MDM command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleCommand {
    pub device_id: String,
    pub command_uuid: String,
    pub updated_at: DateTime<Utc>,
    pub request_type: String,
    pub status: String,
    pub hostname: String,
    #[serde(skip)]
    pub team_id: Option<u32>,
}

/// PayloadScope represents the scope of an Apple MDM payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PayloadScope {
    #[serde(rename = "User")]
    User,
    #[serde(rename = "System")]
    System,
}

impl Default for PayloadScope {
    fn default() -> Self {
        PayloadScope::System
    }
}

/// MDMAppleConfigProfile represents an Apple configuration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleConfigProfile {
    pub profile_uuid: String,
    pub profile_id: u32,
    pub team_id: Option<u32>,
    pub identifier: String,
    #[serde(default)]
    pub scope: PayloadScope,
    pub name: String,
    #[serde(skip)]
    pub mobileconfig: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub labels_include_all: Vec<ConfigurationProfileLabel>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub labels_include_any: Vec<ConfigurationProfileLabel>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub labels_exclude_any: Vec<ConfigurationProfileLabel>,
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updated_at")]
    pub uploaded_at: DateTime<Utc>,
    #[serde(skip)]
    pub secrets_updated_at: Option<DateTime<Utc>>,
}

/// HostMDMAppleProfile represents the status of an Apple MDM profile on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMDMAppleProfile {
    #[serde(skip)]
    pub host_uuid: String,
    #[serde(skip)]
    pub command_uuid: String,
    pub profile_uuid: String,
    pub name: String,
    #[serde(skip)]
    pub identifier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MDMDeliveryStatus>,
    pub operation_type: MDMOperationType,
    pub detail: String,
    #[serde(skip)]
    pub variables_updated_at: Option<DateTime<Utc>>,
    pub scope: PayloadScope,
    #[serde(default)]
    pub managed_local_account: String,
}

/// HostMDMCertificateProfile represents the status of an MDM certificate profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMDMCertificateProfile {
    pub host_uuid: String,
    pub profile_uuid: String,
    pub status: Option<MDMDeliveryStatus>,
    pub challenge_retrieved_at: Option<DateTime<Utc>>,
    pub not_valid_before: Option<DateTime<Utc>>,
    pub not_valid_after: Option<DateTime<Utc>>,
    #[serde(rename = "type")]
    pub ca_type: String,
    pub ca_name: String,
    pub serial: Option<String>,
}

/// HostMDMProfileDetail represents a specific detail string for a host MDM profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HostMDMProfileDetail {
    #[serde(rename = "Failed, was verified")]
    FailedWasVerified,
    #[serde(rename = "Failed, was verifying")]
    FailedWasVerifying,
}

/// MDMAppleProfilePayload represents the payload for an Apple profile on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleProfilePayload {
    pub profile_uuid: String,
    pub profile_identifier: String,
    pub profile_name: String,
    pub host_uuid: String,
    pub host_platform: String,
    pub checksum: Vec<u8>,
    pub secrets_updated_at: Option<DateTime<Utc>>,
    pub status: Option<MDMDeliveryStatus>,
    pub operation_type: MDMOperationType,
    pub detail: String,
    pub command_uuid: String,
    pub ignore_error: bool,
    pub scope: PayloadScope,
    pub device_enrolled_at: Option<DateTime<Utc>>,
}

/// MDMAppleBulkUpsertHostProfilePayload is the payload for bulk-upserting Apple host profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleBulkUpsertHostProfilePayload {
    pub profile_uuid: String,
    pub profile_identifier: String,
    pub profile_name: String,
    pub host_uuid: String,
    pub command_uuid: String,
    pub operation_type: MDMOperationType,
    pub status: Option<MDMDeliveryStatus>,
    pub detail: String,
    pub checksum: Vec<u8>,
    pub secrets_updated_at: Option<DateTime<Utc>>,
    pub ignore_error: bool,
    pub variables_updated_at: Option<DateTime<Utc>>,
    pub scope: PayloadScope,
}

/// MDMAppleDEPDevice represents an Apple DEP device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDEPDevice {
    pub serial_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_uuid: Option<String>,
}

/// MDMAppleBootstrapPackage represents a bootstrap package for Apple MDM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleBootstrapPackage {
    pub name: String,
    pub team_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
    pub sha256: Vec<u8>,
    pub token: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    pub updated_at: DateTime<Utc>,
}

/// MDMAppleSetupAssistant represents a macOS setup assistant configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleSetupAssistant {
    #[serde(skip)]
    pub id: u32,
    pub team_id: Option<u32>,
    pub name: String,
    #[serde(rename = "enrollment_profile")]
    pub profile: serde_json::Value,
    pub uploaded_at: DateTime<Utc>,
}

/// MDMAppleSettingsPayload describes the payload to update MDM macOS settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleSettingsPayload {
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_disk_encryption: Option<bool>,
}

/// MDMAppleSetupPayload describes the payload to update MDM macOS setup values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleSetupPayload {
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_end_user_authentication: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_release_device_manually: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual_agent_install: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_all_software_macos: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_end_user_info: Option<bool>,
}

/// MDMAppleFleetdConfig contains the fields used to configure `fleetd` on macOS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleFleetdConfig {
    pub fleet_url: String,
    pub enroll_secret: String,
    pub enable_scripts: bool,
}

/// MDMCustomEnrollmentProfileItem represents an MDM enrollment profile item
/// with custom fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMCustomEnrollmentProfileItem {
    pub end_user_email: String,
}

/// MDMApplePreassignProfilePayload is the payload for preassigning profiles to hosts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMApplePreassignProfilePayload {
    pub external_host_identifier: String,
    pub host_uuid: String,
    pub profile: Vec<u8>,
    pub group: String,
    #[serde(default)]
    pub exclude: bool,
}

/// MDMApplePreassignHostProfiles represents the set of profiles pre-assigned to a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMApplePreassignHostProfiles {
    pub host_uuid: String,
    pub profiles: Vec<MDMApplePreassignProfile>,
}

/// MDMApplePreassignProfile represents a single profile pre-assigned to a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMApplePreassignProfile {
    pub profile: Vec<u8>,
    pub group: String,
    pub hex_md5_hash: String,
    pub exclude: bool,
}

/// HostDEPAssignment represents a row in the host_dep_assignments table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostDEPAssignment {
    pub host_id: u32,
    pub added_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub abm_token_id: Option<u32>,
    pub mdm_migration_deadline: Option<DateTime<Utc>>,
    pub mdm_migration_completed: Option<DateTime<Utc>>,
}

/// DEPAssignProfileResponseStatus represents the status of a DEP profile assignment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DEPAssignProfileResponseStatus {
    #[serde(rename = "SUCCESS")]
    Success,
    #[serde(rename = "NOT_ACCESSIBLE")]
    NotAccessible,
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "THROTTLED")]
    Throttled,
}

/// NanoEnrollment represents a row in the nano_enrollments table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NanoEnrollment {
    pub id: String,
    pub device_id: String,
    #[serde(rename = "type")]
    pub enrollment_type: String,
    pub enabled: bool,
    pub token_update_tally: i32,
}

/// NanoUser represents a row in the nano_users table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NanoUser {
    pub id: String,
    pub device_id: String,
    pub user_short_name: String,
    pub user_long_name: String,
}

/// EnrolledAPIResult is a per-enrollment API result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrolledAPIResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_error: Option<String>,
}

/// EnrolledAPIResults is a map of enrollments to API results.
pub type EnrolledAPIResults = HashMap<String, EnrolledAPIResult>;

/// SCEPIdentityCertificate represents a certificate issued during MDM enrollment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SCEPIdentityCertificate {
    pub serial: String,
    pub not_valid_after: DateTime<Utc>,
    pub certificate_pem: Vec<u8>,
}

/// SCEPIdentityAssociation represents an association between an identity certificate and a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SCEPIdentityAssociation {
    pub host_uuid: String,
    pub sha256: String,
    pub enroll_reference: String,
    pub renew_command_uuid: String,
    pub enrolled_from_migration: bool,
    #[serde(rename = "type")]
    pub enrollment_type: String,
}

// ===========================================================================
// Apple DDM (Declarative Device Management) types (from apple_mdm.go)
// ===========================================================================

/// MDMAppleDeclaration represents an Apple DDM declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDeclaration {
    #[serde(rename = "profile_uuid")]
    pub declaration_uuid: String,
    pub team_id: Option<u32>,
    pub identifier: String,
    pub name: String,
    #[serde(skip)]
    pub raw_json: Vec<u8>,
    #[serde(skip)]
    pub token: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub labels_include_all: Vec<ConfigurationProfileLabel>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub labels_include_any: Vec<ConfigurationProfileLabel>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub labels_exclude_any: Vec<ConfigurationProfileLabel>,
    pub created_at: DateTime<Utc>,
    pub uploaded_at: DateTime<Utc>,
    #[serde(skip)]
    pub secrets_updated_at: Option<DateTime<Utc>>,
}

/// MDMAppleRawDeclaration contains the type and identifier from raw declaration JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleRawDeclaration {
    #[serde(rename = "Type")]
    pub declaration_type: String,
    #[serde(rename = "Identifier")]
    pub identifier: String,
}

/// MDMAppleHostDeclaration represents the state of a declaration on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleHostDeclaration {
    #[serde(skip)]
    pub host_uuid: String,
    #[serde(rename = "profile_uuid")]
    pub declaration_uuid: String,
    pub name: String,
    #[serde(skip)]
    pub identifier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MDMDeliveryStatus>,
    pub operation_type: MDMOperationType,
    pub detail: String,
    #[serde(skip)]
    pub token: String,
    #[serde(skip)]
    pub secrets_updated_at: Option<DateTime<Utc>>,
}

/// MDMAppleDDMTokensResponse is the response from the DDM tokens endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDDMTokensResponse {
    pub sync_tokens: MDMAppleDDMDeclarationsToken,
}

/// MDMAppleDDMDeclarationsToken describes the state of declarations on the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDDMDeclarationsToken {
    pub declarations_token: String,
    pub timestamp: DateTime<Utc>,
}

/// MDMAppleDDMDeclarationItemsResponse is the response from the DDM declaration items endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDDMDeclarationItemsResponse {
    pub declarations: MDMAppleDDMManifestItems,
    pub declarations_token: String,
}

/// MDMAppleDDMManifestItems contains lists of declarations available on the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDDMManifestItems {
    pub activations: Vec<MDMAppleDDMManifest>,
    pub assets: Vec<MDMAppleDDMManifest>,
    pub configurations: Vec<MDMAppleDDMManifest>,
    pub management: Vec<MDMAppleDDMManifest>,
}

/// MDMAppleDDMManifest describes a declaration in a manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDDMManifest {
    pub identifier: String,
    pub server_token: String,
}

/// MDMAppleDDMDeclarationItem represents a declaration item in the datastore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDDMDeclarationItem {
    pub declaration_uuid: String,
    pub identifier: String,
    #[serde(rename = "token")]
    pub server_token: String,
    pub status: Option<String>,
    pub operation_type: Option<String>,
    pub uploaded_at: DateTime<Utc>,
}

/// MDMAppleDDMDeclarationResponse represents a declaration response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDDMDeclarationResponse {
    pub identifier: String,
    #[serde(rename = "type")]
    pub declaration_type: String,
    pub payload: serde_json::Value,
    pub server_token: String,
}

/// MDMAppleDDMStatusReport represents a report of the device's current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDDMStatusReport {
    #[serde(rename = "StatusItems")]
    pub status_items: MDMAppleDDMStatusItems,
    #[serde(rename = "Errors")]
    pub errors: Vec<MDMAppleDDMErrors>,
}

/// MDMAppleDDMStatusItems are the status items for a report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDDMStatusItems {
    pub management: MDMAppleDDMStatusManagement,
}

/// MDMAppleDDMStatusManagement represents status report of the client's processed declarations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDDMStatusManagement {
    pub declarations: MDMAppleDDMStatusDeclarations,
}

/// MDMAppleDDMStatusDeclarations represents a collection of the client's processed declarations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDDMStatusDeclarations {
    pub activations: Vec<MDMAppleDDMStatusDeclaration>,
    pub configurations: Vec<MDMAppleDDMStatusDeclaration>,
    pub assets: Vec<MDMAppleDDMStatusDeclaration>,
    pub management: Vec<MDMAppleDDMStatusDeclaration>,
}

/// MDMAppleDeclarationValidity represents the validity of a declaration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MDMAppleDeclarationValidity {
    Valid,
    Invalid,
    Unknown,
}

/// MDMAppleDDMStatusDeclaration represents a processed declaration for the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDDMStatusDeclaration {
    pub active: bool,
    pub identifier: String,
    pub valid: MDMAppleDeclarationValidity,
    #[serde(rename = "server-token")]
    pub server_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasons: Option<Vec<MDMAppleDDMStatusErrorReason>>,
}

/// MDMAppleDDMErrors represents a status report error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDDMErrors {
    #[serde(rename = "StatusItem")]
    pub status_item: String,
    #[serde(rename = "Reasons")]
    pub reasons: Vec<MDMAppleDDMStatusErrorReason>,
}

/// MDMAppleDDMStatusErrorReason contains details about an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDDMStatusErrorReason {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Details")]
    pub details: HashMap<String, serde_json::Value>,
}

/// MDMAppleDDMActivationPayload represents the payload of an activation declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDDMActivationPayload {
    #[serde(rename = "Predicate")]
    pub predicate: String,
    #[serde(rename = "StandardConfigurations")]
    pub standard_configurations: Vec<String>,
}

/// MDMAppleDDMActivation represents the declaration of an activation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDDMActivation {
    #[serde(rename = "Identifier")]
    pub identifier: String,
    #[serde(rename = "Payload")]
    pub payload: MDMAppleDDMActivationPayload,
    #[serde(rename = "ServerToken")]
    pub server_token: String,
    #[serde(rename = "Type")]
    pub activation_type: String,
}

/// MDMAppleMachineInfo is a device's information sent as part of an MDM enrollment profile request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleMachineInfo {
    #[serde(rename = "IMEI", skip_serializing_if = "Option::is_none")]
    pub imei: Option<String>,
    #[serde(rename = "LANGUAGE", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(rename = "MDM_CAN_REQUEST_SOFTWARE_UPDATE")]
    pub mdm_can_request_software_update: bool,
    #[serde(rename = "MEID", skip_serializing_if = "Option::is_none")]
    pub meid: Option<String>,
    #[serde(rename = "OS_VERSION")]
    pub os_version: String,
    #[serde(rename = "PAIRING_TOKEN", skip_serializing_if = "Option::is_none")]
    pub pairing_token: Option<String>,
    #[serde(rename = "PRODUCT")]
    pub product: String,
    #[serde(rename = "SERIAL")]
    pub serial: String,
    #[serde(
        rename = "SOFTWARE_UPDATE_DEVICE_ID",
        skip_serializing_if = "Option::is_none"
    )]
    pub software_update_device_id: Option<String>,
    #[serde(
        rename = "SUPPLEMENTAL_BUILD_VERSION",
        skip_serializing_if = "Option::is_none"
    )]
    pub supplemental_build_version: Option<String>,
    #[serde(
        rename = "SUPPLEMENTAL_OS_VERSION_EXTRA",
        skip_serializing_if = "Option::is_none"
    )]
    pub supplemental_os_version_extra: Option<String>,
    #[serde(rename = "UDID")]
    pub udid: String,
    #[serde(rename = "VERSION")]
    pub version: String,
}

/// MDMAppleSoftwareUpdateRequired is the error response to indicate a device
/// needs a software update before enrollment can proceed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleSoftwareUpdateRequired {
    pub code: String,
    pub details: MDMAppleSoftwareUpdateRequiredDetails,
}

/// MDMAppleSoftwareUpdateRequiredDetails contains the required update version info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleSoftwareUpdateRequiredDetails {
    #[serde(rename = "OSVersion")]
    pub os_version: String,
    #[serde(rename = "BuildVersion")]
    pub build_version: String,
}

pub const MDM_APPLE_SOFTWARE_UPDATE_REQUIRED_CODE: &str = "com.apple.softwareupdate.required";

/// MDMAppleSoftwareUpdateAsset represents a software update asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleSoftwareUpdateAsset {
    #[serde(rename = "ProductVersion")]
    pub product_version: String,
    #[serde(rename = "Build")]
    pub build: String,
}

/// MDMManagedCertificate represents a managed certificate for an MDM profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMManagedCertificate {
    pub profile_uuid: String,
    pub host_uuid: String,
    pub challenge_retrieved_at: Option<DateTime<Utc>>,
    pub not_valid_before: Option<DateTime<Utc>>,
    pub not_valid_after: Option<DateTime<Utc>>,
    #[serde(rename = "type")]
    pub ca_type: String,
    pub ca_name: String,
    pub serial: Option<String>,
}

/// MDMAppleEnrolledDeviceInfo represents the information of an enrolled Apple device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleEnrolledDeviceInfo {
    pub id: String,
    pub serial_number: String,
    pub authenticate: String,
    pub platform: String,
    pub enroll_team_id: Option<u32>,
}

/// HostLocationData represents location data for a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostLocationData {
    pub host_id: u32,
    pub latitude: f64,
    pub longitude: f64,
}

// Apple MDM command names
pub const DEVICE_LOCATION_CMD_NAME: &str = "DeviceLocation";
pub const ENABLE_LOST_MODE_CMD_NAME: &str = "EnableLostMode";
pub const DISABLE_LOST_MODE_CMD_NAME: &str = "DisableLostMode";

// ===========================================================================
// Microsoft/Windows MDM types (from microsoft_mdm.go)
// ===========================================================================

pub const WINDOWS_SCEP_LOC_URI_PART: &str = "/Vendor/MSFT/ClientCertificateInstall/SCEP";
pub const WINDOWS_MDM_AUTH_NONCE_PREFIX: &str = "mwenonce:";

/// MS-MDE2 message types
pub const MDE_DISCOVERY: u32 = 0;
pub const MDE_POLICY: u32 = 1;
pub const MDE_ENROLLMENT: u32 = 2;
pub const MDE_FAULT: u32 = 3;
pub const MS_MDM: u32 = 4;

/// WindowsMDMEnrollmentType represents supported Windows MDM enrollment types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WindowsMDMEnrollmentType {
    Programmatic = 1,
    Automatic = 2,
}

/// WindowsMDMAccessTokenPayload is the payload that gets encoded as JSON and
/// provided as opaque access token to the RegisterDeviceWithManagement API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsMDMAccessTokenPayload {
    #[serde(rename = "type")]
    pub enrollment_type: WindowsMDMEnrollmentType,
    pub payload: WindowsMDMAccessTokenPayloadInner,
}

/// Inner payload of the WindowsMDMAccessTokenPayload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsMDMAccessTokenPayloadInner {
    #[serde(default)]
    pub orbit_node_key: String,
    #[serde(default)]
    pub auth_token: String,
}

/// MDMWindowsEnrolledDevice contains the information of an enrolled Windows host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMWindowsEnrolledDevice {
    pub id: u32,
    pub host_uuid: String,
    pub mdm_device_id: String,
    pub mdm_hardware_id: String,
    #[serde(rename = "device_state")]
    pub mdm_device_state: String,
    #[serde(rename = "device_type")]
    pub mdm_device_type: String,
    #[serde(rename = "device_name")]
    pub mdm_device_name: String,
    #[serde(rename = "enroll_type")]
    pub mdm_enroll_type: String,
    #[serde(rename = "enroll_user_id")]
    pub mdm_enroll_user_id: String,
    #[serde(rename = "enroll_proto_version")]
    pub mdm_enroll_proto_version: String,
    #[serde(rename = "enroll_client_version")]
    pub mdm_enroll_client_version: String,
    pub not_in_oobe: bool,
    #[serde(skip)]
    pub credentials_hash: Option<Vec<u8>>,
    #[serde(skip)]
    pub credentials_acknowledged: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// MDMWindowsCommand represents a Windows MDM command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMWindowsCommand {
    pub command_uuid: String,
    pub raw_command: Vec<u8>,
    pub target_loc_uri: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// HostMDMWindowsProfile represents the status of an MDM profile for a Windows host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMDMWindowsProfile {
    pub host_uuid: String,
    pub command_uuid: String,
    pub profile_uuid: String,
    pub name: String,
    pub status: Option<MDMDeliveryStatus>,
    pub operation_type: MDMOperationType,
    pub detail: String,
}

/// MDMWindowsConfigProfile represents a Windows configuration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMWindowsConfigProfile {
    pub profile_uuid: String,
    pub team_id: Option<u32>,
    pub name: String,
    #[serde(skip)]
    pub syncml: Vec<u8>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub labels_include_all: Vec<ConfigurationProfileLabel>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub labels_include_any: Vec<ConfigurationProfileLabel>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub labels_exclude_any: Vec<ConfigurationProfileLabel>,
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updated_at")]
    pub uploaded_at: DateTime<Utc>,
}

// ===========================================================================
// Windows MDM types (from windows_mdm.go)
// ===========================================================================

/// MDMWindowsBitLockerSummary reports the number of Windows hosts being managed with BitLocker.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMWindowsBitLockerSummary {
    pub verified: u32,
    pub verifying: u32,
    pub action_required: u32,
    pub enforcing: u32,
    pub failed: u32,
    pub removing_enforcement: u32,
}

/// MDMWindowsProfilePayload represents a Windows profile payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMWindowsProfilePayload {
    pub profile_uuid: String,
    pub profile_name: String,
    pub host_uuid: String,
    pub status: Option<MDMDeliveryStatus>,
    pub operation_type: MDMOperationType,
    pub detail: String,
    pub command_uuid: String,
    pub retries: i32,
    #[serde(skip)]
    pub checksum: Vec<u8>,
    pub secrets_updated_at: Option<DateTime<Utc>>,
}

/// MDMWindowsBulkUpsertHostProfilePayload is the payload for bulk-upserting Windows host profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMWindowsBulkUpsertHostProfilePayload {
    pub profile_uuid: String,
    pub profile_name: String,
    pub host_uuid: String,
    pub command_uuid: String,
    pub operation_type: MDMOperationType,
    pub status: Option<MDMDeliveryStatus>,
    pub detail: String,
    pub checksum: Vec<u8>,
}

/// MDMWindowsProfileContents contains the SyncML and checksum for a Windows profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMWindowsProfileContents {
    pub syncml: Vec<u8>,
    pub checksum: Vec<u8>,
}

/// MDMWindowsWipeType specifies what type of remote wipe to perform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MDMWindowsWipeType {
    #[serde(rename = "doWipe")]
    DoWipe,
    #[serde(rename = "doWipeProtected")]
    DoWipeProtected,
}

/// MDMWindowsWipeMetadata contains metadata for a Windows wipe command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMWindowsWipeMetadata {
    pub wipe_type: MDMWindowsWipeType,
}

// ===========================================================================
// SyncML protocol command verbs (from microsoft_mdm.go)
// ===========================================================================

pub const CMD_ADD: &str = "Add";
pub const CMD_ALERT: &str = "Alert";
pub const CMD_ATOMIC: &str = "Atomic";
pub const CMD_DELETE: &str = "Delete";
pub const CMD_EXEC: &str = "Exec";
pub const CMD_GET: &str = "Get";
pub const CMD_REPLACE: &str = "Replace";
pub const CMD_RESULTS: &str = "Results";
pub const CMD_STATUS: &str = "Status";

/// MDMCommandType represents SyncML command types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MDMCommandType {
    Raw = 0,
    Add = 1,
    Alert = 2,
    Atomic = 3,
    Delete = 4,
    Exec = 5,
    Get = 6,
    Replace = 7,
    Results = 8,
    Status = 9,
}

/// SyncMLDataType represents data types for SyncML commands.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncMLDataType {
    Empty = 0,
    NoFormat = 1,
    Text = 2,
    Xml = 3,
    Integer = 4,
    Boolean = 5,
    Base64 = 6,
}

/// ProtoCmdState is the state of the SyncML protocol commands.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProtoCmdState {
    Received = 0,
    Pending = 1,
    Sent = 2,
    ResponseProcessing = 3,
    ResponseAck = 4,
}

// ===========================================================================
// Linux MDM types (from linux_mdm.go)
// ===========================================================================

/// MDMLinuxDiskEncryptionSummary reports the number of Linux hosts being managed
/// with disk encryption.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMLinuxDiskEncryptionSummary {
    pub verified: u32,
    pub action_required: u32,
    pub failed: u32,
}

// ===========================================================================
// Microsoft MDM SOAP protocol types (from microsoft_mdm.go)
// ===========================================================================

/// SoapResponse is the SOAP Envelope Response type for MS-MDE2 responses from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapResponse {
    #[serde(rename = "XMLNSS")]
    pub xmlns_s: String,
    #[serde(rename = "XMLNSA")]
    pub xmlns_a: String,
    #[serde(rename = "XMLNSU", skip_serializing_if = "Option::is_none")]
    pub xmlns_u: Option<String>,
    pub header: ResponseHeader,
    pub body: BodyResponse,
}

/// SoapRequest is the SOAP Envelope Request type for MS-MDE2 requests to the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapRequest {
    #[serde(rename = "XMLNSS")]
    pub xmlns_s: String,
    #[serde(rename = "XMLNSA")]
    pub xmlns_a: String,
    #[serde(rename = "XMLNSU", skip_serializing_if = "Option::is_none")]
    pub xmlns_u: Option<String>,
    #[serde(rename = "XMLNSWsse", skip_serializing_if = "Option::is_none")]
    pub xmlns_wsse: Option<String>,
    #[serde(rename = "XMLNSWST", skip_serializing_if = "Option::is_none")]
    pub xmlns_wst: Option<String>,
    #[serde(rename = "XMLNSAC", skip_serializing_if = "Option::is_none")]
    pub xmlns_ac: Option<String>,
    pub header: RequestHeader,
    pub body: BodyRequest,
    /// Raw XML bytes, stored alongside the decoded fields for convenience.
    #[serde(skip)]
    pub raw: Vec<u8>,
}

/// ResponseHeader is the header for MDM responses from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseHeader {
    pub action: Action,
    pub relates_to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_id: Option<ActivityId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<WsSecurity>,
}

/// RequestHeader is the header for MDM requests to the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestHeader {
    pub action: Action,
    pub message_id: String,
    pub reply_to: ReplyTo,
    pub to: To,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<TokenSecurity>,
}

/// BodyResponse is the body of the MDM SOAP response message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xsd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xsi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discover_response: Option<DiscoverResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_policies_response: Option<GetPoliciesResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_security_token_response_collection: Option<RequestSecurityTokenResponseCollection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soap_fault: Option<SoapFault>,
}

/// BodyRequest is the body of the MDM SOAP request message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xsi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xsd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discover: Option<Discover>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_policies: Option<GetPolicies>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_security_token: Option<RequestSecurityToken>,
}

/// Action is the SOAP action header field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub content: String,
    pub must_understand: String,
}

/// ActivityId is a unique identifier for the activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityId {
    pub content: String,
    pub correlation_id: String,
    #[serde(rename = "xmlns")]
    pub xmlns: String,
}

/// Timestamp for certificate authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamp {
    pub id: String,
    pub created: String,
    pub expires: String,
}

/// WsSecurity is the security token container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsSecurity {
    #[serde(rename = "xmlns")]
    pub xmlns: String,
    pub must_understand: String,
    pub timestamp: Timestamp,
}

/// HeaderBinarySecurityToken is the security token container for encoded security sensitive data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderBinarySecurityToken {
    pub content: String,
    #[serde(rename = "ValueType")]
    pub value: String,
    #[serde(rename = "EncodingType")]
    pub encoding: String,
}

/// TokenSecurity is the security token container for BinarySecurityToken.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSecurity {
    pub must_understand: String,
    pub security: HeaderBinarySecurityToken,
}

/// To is the target endpoint header field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct To {
    pub content: String,
    pub must_understand: String,
}

/// ReplyTo is the message correlation header field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyTo {
    pub address: String,
}

// ===========================================================================
// Discover MS-MDE2 message types (from microsoft_mdm.go)
// ===========================================================================

/// Discover MS-MDE2 message request type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discover {
    #[serde(rename = "xmlns")]
    pub xmlns: String,
    pub request: DiscoverRequest,
}

/// AuthPolicies contains the authentication policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthPolicies {
    pub auth_policy: Vec<String>,
}

/// DiscoverRequest contains the discovery request parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverRequest {
    #[serde(rename = "xmlns")]
    pub xmlns: String,
    pub email_address: String,
    pub request_version: String,
    pub device_type: String,
    pub application_version: String,
    #[serde(rename = "OSEdition")]
    pub os_edition: String,
    pub auth_policies: AuthPolicies,
}

/// DiscoverResponse MS-MDE2 message response type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverResponse {
    #[serde(rename = "xmlns")]
    pub xmlns: String,
    pub discover_result: DiscoverResult,
}

/// DiscoverResult contains the discovery response data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverResult {
    pub auth_policy: String,
    pub enrollment_version: String,
    pub enrollment_policy_service_url: String,
    pub enrollment_service_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_service_url: Option<String>,
}

// ===========================================================================
// GetPolicies MS-MDE2 message types (from microsoft_mdm.go)
// ===========================================================================

/// GetPolicies MS-MDE2 message request type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPolicies {
    #[serde(rename = "xmlns")]
    pub xmlns: String,
    pub client: Client,
    pub request_filter: RequestFilter,
}

/// ClientContent holds a content string and xsi attribute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientContent {
    pub content: String,
    pub xsi: String,
}

/// Client contains the client information for GetPolicies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub last_update: ClientContent,
    pub preferred_language: ClientContent,
}

/// RequestFilter for the GetPolicies request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestFilter {
    pub xsi: String,
}

/// GetPoliciesResponse MS-MDE2 message response type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPoliciesResponse {
    #[serde(rename = "xmlns")]
    pub xmlns: String,
    pub response: PolicyResponse,
    #[serde(rename = "oIDs")]
    pub oids: OIDs,
}

/// ContentAttr holds content and xsi/xmlns attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAttr {
    pub content: String,
    pub xsi: String,
    #[serde(rename = "xmlns")]
    pub xmlns: String,
}

/// GenericAttr holds a single xsi attribute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericAttr {
    pub xsi: String,
}

/// CertificateValidity specifies validity period for certificates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateValidity {
    pub validity_period_seconds: String,
    pub renewal_period_seconds: String,
}

/// Permission specifies enrollment permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub enroll: String,
    pub auto_enroll: String,
}

/// ProviderAttr holds a provider content string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAttr {
    pub content: String,
}

/// PrivateKeyAttributes specifies the private key attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateKeyAttributes {
    pub minimal_key_length: String,
    pub key_spec: GenericAttr,
    pub key_usage_property: GenericAttr,
    pub permissions: GenericAttr,
    pub algorithm_oid_reference: GenericAttr,
    pub crypto_providers: Vec<ProviderAttr>,
}

/// Revision specifies policy revision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    pub major_revision: String,
    pub minor_revision: String,
}

/// Attributes specifies the policy attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAttributes {
    pub common_name: String,
    pub policy_schema: String,
    pub certificate_validity: CertificateValidity,
    pub permission: Permission,
    pub private_key_attributes: PrivateKeyAttributes,
    pub revision: Revision,
    pub superseded_policies: GenericAttr,
    pub private_key_flags: GenericAttr,
    pub subject_name_flags: GenericAttr,
    pub enrollment_flags: GenericAttr,
    pub general_flags: GenericAttr,
    pub hash_algorithm_oid_reference: String,
    pub ra_requirements: GenericAttr,
    pub key_archival_attributes: GenericAttr,
    pub extensions: GenericAttr,
}

/// GPPolicy represents a group policy object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPPolicy {
    pub policy_oid_reference: String,
    pub cas: GenericAttr,
    pub attributes: PolicyAttributes,
}

/// Policies contains the policies in the response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policies {
    pub policy: GPPolicy,
}

/// PolicyResponse is the response element of GetPoliciesResponse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResponse {
    pub policy_id: String,
    pub policy_friendly_name: ContentAttr,
    pub next_update_hours: ContentAttr,
    pub policies_not_changed: ContentAttr,
    pub policies: Policies,
}

/// OID represents an object identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OID {
    pub value: String,
    pub group: String,
    pub oid_reference_id: String,
    pub default_name: String,
}

/// OIDs contains a list of object identifiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OIDs {
    pub content: String,
    pub oid: Vec<OID>,
}

// ===========================================================================
// RequestSecurityToken MS-MDE2 message types (from microsoft_mdm.go)
// ===========================================================================

/// RequestSecurityToken MS-MDE2 message request type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSecurityToken {
    pub token_type: String,
    pub request_type: String,
    pub binary_security_token: BinarySecurityToken,
    pub additional_context: AdditionalContext,
    #[serde(skip)]
    pub map_context_items: Option<HashMap<String, ContextItem>>,
}

/// BinarySecurityToken contains the base64 encoded security token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinarySecurityToken {
    pub content: String,
    #[serde(rename = "xmlns", skip_serializing_if = "Option::is_none")]
    pub xmlns: Option<String>,
    #[serde(rename = "ValueType")]
    pub value_type: String,
    #[serde(rename = "EncodingType")]
    pub encoding_type: String,
}

/// ContextItem represents a single context item in AdditionalContext.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    pub name: String,
    pub value: String,
}

/// AdditionalContext contains context items for RequestSecurityToken.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalContext {
    #[serde(rename = "xmlns")]
    pub xmlns: String,
    pub context_items: Vec<ContextItem>,
}

// ===========================================================================
// RequestSecurityTokenResponseCollection MS-MDE2 types (from microsoft_mdm.go)
// ===========================================================================

/// RequestSecurityTokenResponseCollection MS-MDE2 message response type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSecurityTokenResponseCollection {
    #[serde(rename = "xmlns")]
    pub xmlns: String,
    pub request_security_token_response: RequestSecurityTokenResponse,
}

/// SecAttr holds content and xmlns attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecAttr {
    pub content: String,
    #[serde(rename = "xmlns")]
    pub xmlns: String,
}

/// RequestedSecurityToken wraps a BinarySecurityToken.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestedSecurityToken {
    pub binary_security_token: BinarySecurityToken,
}

/// RequestSecurityTokenResponse is the MS-MDE2 security token response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSecurityTokenResponse {
    pub token_type: String,
    pub disposition_message: SecAttr,
    pub requested_security_token: RequestedSecurityToken,
    pub request_id: SecAttr,
}

// ===========================================================================
// SoapFault MS-MDE2 message types (from microsoft_mdm.go)
// ===========================================================================

/// Subcode for SOAP fault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subcode {
    pub value: String,
}

/// Code for SOAP fault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Code {
    pub value: String,
    pub subcode: Subcode,
}

/// ReasonText for SOAP fault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonText {
    pub content: String,
    pub lang: String,
}

/// Reason for SOAP fault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reason {
    pub text: ReasonText,
}

/// SoapFault MS-MDE2 message response type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapFault {
    pub code: Code,
    pub reason: Reason,
    #[serde(skip)]
    pub original_message_type: i32,
}

// ===========================================================================
// WapProvisioningDoc (XML Provisioning Schema) types (from microsoft_mdm.go)
// ===========================================================================

/// Param represents a provisioning parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datatype: Option<String>,
}

/// Characteristic represents a provisioning characteristic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Characteristic {
    #[serde(rename = "type")]
    pub characteristic_type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<Param>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub characteristics: Vec<Characteristic>,
}

/// WapProvisioningDoc is the MS-MDE2 provisioning document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WapProvisioningDoc {
    pub version: String,
    pub characteristics: Vec<Characteristic>,
}

// ===========================================================================
// SyncML protocol types (from microsoft_mdm.go)
// ===========================================================================

/// SyncML represents a SyncML message used by MS-MDM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncML {
    #[serde(rename = "xmlns")]
    pub xmlns: String,
    pub sync_hdr: SyncHdr,
    pub sync_body: SyncBody,
    /// Raw XML bytes, stored alongside the decoded fields for convenience.
    #[serde(skip)]
    pub raw: Vec<u8>,
}

/// SyncHdr is the SyncML message header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHdr {
    pub ver_dtd: String,
    pub ver_proto: String,
    pub session_id: String,
    pub msg_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<LocURI>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<LocURI>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<MetaHdr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cred: Option<CredHdr>,
}

/// MetaHdr is metadata for the SyncML header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaHdr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_msg_size: Option<String>,
}

/// CredHdr contains credential information for the SyncML header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredHdr {
    pub meta: SyncMLMeta,
    pub data: String,
}

/// SyncBody is the SyncML message body containing protocol commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_elem: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub add: Vec<SyncMLCmd>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alert: Vec<SyncMLCmd>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub atomic: Vec<SyncMLCmd>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delete: Vec<SyncMLCmd>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exec: Vec<SyncMLCmd>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub get: Vec<SyncMLCmd>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replace: Vec<SyncMLCmd>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<SyncMLCmd>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub status: Vec<SyncMLCmd>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw: Vec<SyncMLCmd>,
}

/// ProtoCmdOperation represents a SyncML protocol command with its verb.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoCmdOperation {
    pub verb: String,
    pub cmd: SyncMLCmd,
}

/// CmdID holds the value of a CmdID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmdID {
    pub value: String,
    #[serde(skip)]
    pub include_fleet_comment: bool,
}

/// SyncMLCmd represents a SyncML protocol command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMLCmd {
    pub cmd_id: CmdID,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<CmdItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chal: Option<SyncMLChallenge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replace_commands: Vec<SyncMLCmd>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub add_commands: Vec<SyncMLCmd>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exec_commands: Vec<SyncMLCmd>,
}

/// SyncMLChallenge represents a SyncML authentication challenge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMLChallenge {
    pub meta: ChallengeMeta,
}

/// ChallengeMeta contains metadata for a SyncML challenge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeMeta {
    #[serde(flatten)]
    pub meta: SyncMLMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_nonce: Option<MetaAttr>,
}

/// CmdItem represents an item within a SyncML command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmdItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<SyncMLMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<RawXmlData>,
}

/// RawXmlData represents raw XML data content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawXmlData {
    pub content: String,
}

/// SyncMLMeta represents metadata for a SyncML command item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMLMeta {
    #[serde(rename = "Type", skip_serializing_if = "Option::is_none")]
    pub meta_type: Option<MetaAttr>,
    #[serde(rename = "Format", skip_serializing_if = "Option::is_none")]
    pub format: Option<MetaAttr>,
}

/// MetaAttr represents a metadata attribute with xmlns and content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaAttr {
    #[serde(rename = "xmlns")]
    pub xmlns: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// LocURI represents a SyncML location URI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocURI {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loc_uri: Option<String>,
}

/// EnrichedSyncML wraps a SyncML message with indexed command references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedSyncML {
    pub sync_ml: SyncML,
    #[serde(default)]
    pub cmd_ref_uuid_to_status: HashMap<String, SyncMLCmd>,
    #[serde(default)]
    pub cmd_ref_uuid_to_results: HashMap<String, SyncMLCmd>,
    #[serde(default)]
    pub cmd_ref_uuids: Vec<String>,
}

// ===========================================================================
// Apple MDM additional types (from apple_mdm.go)
// ===========================================================================

/// MDMAppleAccountDrivenUserEnrollDeviceInfo is a minimal version of DeviceInfo
/// sent on Account Driven User Enrollment requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleAccountDrivenUserEnrollDeviceInfo {
    #[serde(rename = "VERSION")]
    pub version: String,
    #[serde(rename = "PRODUCT")]
    pub product: String,
    #[serde(rename = "LANGUAGE", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(rename = "OS_VERSION", skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
    #[serde(
        rename = "SOFTWARE_UPDATE_DEVICE_ID",
        skip_serializing_if = "Option::is_none"
    )]
    pub software_update_device_id: Option<String>,
    #[serde(
        rename = "SUPPLEMENTAL_BUILD_VERSION",
        skip_serializing_if = "Option::is_none"
    )]
    pub supplemental_build_version: Option<String>,
}

/// Windows MDM requires premium command message constant.
pub const WINDOWS_MDM_REQUIRES_PREMIUM_CMD_MESSAGE: &str =
    "Missing or invalid license. Wipe command is available in Fleet Premium only.";
