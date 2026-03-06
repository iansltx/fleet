//! MDM (Mobile Device Management) types.
//!
//! Core data types for MDM configuration profiles, commands, and summaries.
//! Ported from Go types in `server/fleet/mdm.go` and `server/fleet/apple_mdm.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// MDM Delivery Status & Operation Type
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// MDM Config Profile Payload (platform-agnostic)
// ---------------------------------------------------------------------------

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
    pub checksum: Option<String>,
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
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broken: Option<bool>,
}

// ---------------------------------------------------------------------------
// MDM Profiles Summary
// ---------------------------------------------------------------------------

/// MDMProfilesSummary contains counts of hosts grouped by profile delivery status.
/// Matches Go's `fleet.MDMProfilesSummary`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMProfilesSummary {
    pub verified: u32,
    pub verifying: u32,
    pub pending: u32,
    pub failed: u32,
}

// ---------------------------------------------------------------------------
// MDM Command
// ---------------------------------------------------------------------------

/// MDMCommand represents an MDM command (Apple or Windows).
/// Matches Go's `fleet.MDMCommand`.
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
}

// ---------------------------------------------------------------------------
// MDM Command Result
// ---------------------------------------------------------------------------

/// MDMCommandResult contains the result of an MDM command execution.
/// Matches Go's `fleet.MDMCommandResult`.
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
}

// ---------------------------------------------------------------------------
// Host MDM Profile
// ---------------------------------------------------------------------------

/// HostMDMProfile represents the status of an MDM profile on a specific host.
/// Matches Go's `fleet.HostMDMProfile`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMDMProfile {
    pub profile_uuid: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub operation_type: String,
    pub detail: String,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_local_account: Option<String>,
}

// ---------------------------------------------------------------------------
// MDM Disk Encryption Summary
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// MDM FileVault Summary
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// MDM Config Profile Status
// ---------------------------------------------------------------------------

/// MDMConfigProfileStatus contains status counts for a specific profile.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMConfigProfileStatus {
    pub verified: u32,
    pub verifying: u32,
    pub pending: u32,
    pub failed: u32,
}

// ---------------------------------------------------------------------------
// Apple MDM types (from apple_mdm.go)
// ---------------------------------------------------------------------------

/// MDMAppleEnrollmentProfile represents an Apple MDM enrollment profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleEnrollmentProfile {
    pub id: u32,
    pub token: String,
    #[serde(rename = "type")]
    pub enrollment_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dep_profile: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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

/// MDMAppleConfigProfile represents an Apple configuration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleConfigProfile {
    pub profile_uuid: String,
    pub profile_id: u32,
    pub team_id: Option<u32>,
    pub name: String,
    pub identifier: String,
    #[serde(skip)]
    pub mobileconfig: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub uploaded_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<Vec<u8>>,
}

/// MDMAppleDeclaration represents an Apple DDM declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleDeclaration {
    pub declaration_uuid: String,
    pub team_id: Option<u32>,
    pub identifier: String,
    pub name: String,
    #[serde(skip)]
    pub raw_json: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub uploaded_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<Vec<u8>>,
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
    pub team_id: u32,
    pub name: String,
    #[serde(skip)]
    pub bytes: Vec<u8>,
    pub sha256: String,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// MDMAppleSetupAssistant represents a macOS setup assistant configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAppleSetupAssistant {
    pub id: u32,
    pub team_id: Option<u32>,
    pub name: String,
    pub profile: serde_json::Value,
    pub uploaded_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Microsoft MDM types (from microsoft_mdm.go)
// ---------------------------------------------------------------------------

/// MDMWindowsConfigProfile represents a Windows configuration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMWindowsConfigProfile {
    pub profile_uuid: String,
    pub team_id: Option<u32>,
    pub name: String,
    #[serde(skip)]
    pub syncml: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub uploaded_at: DateTime<Utc>,
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

/// MDMWindowsEnrolledDevice represents a Windows device enrolled in MDM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMWindowsEnrolledDevice {
    pub id: u32,
    pub mdm_device_id: String,
    pub mdm_hardware_id: String,
    pub device_state: String,
    pub device_type: String,
    pub device_name: String,
    pub enroll_type: String,
    pub enroll_user_id: String,
    pub enroll_proto_version: String,
    pub enroll_client_version: String,
    pub not_in_oobe: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub host_uuid: String,
}

// ---------------------------------------------------------------------------
// General MDM types
// ---------------------------------------------------------------------------

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

/// MDMCommandListOptions configures MDM command listing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMCommandListOptions {
    #[serde(flatten)]
    pub list_options: crate::ListOptions,
}

/// MDMEULAPayload is the payload for MDM EULA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMEULAPayload {
    pub name: String,
    #[serde(skip)]
    pub bytes: Vec<u8>,
    pub token: String,
}
