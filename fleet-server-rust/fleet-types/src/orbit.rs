//! Orbit types matching Go's `server/fleet/orbit.go`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OrbitConfigNotifications are notifications that the fleet server sends to
/// fleetd (orbit) so that it can run commands or more generally react to this
/// information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrbitConfigNotifications {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub renew_enrollment_profile: bool,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub rotate_disk_encryption_key: bool,

    /// NeedsMDMMigration is set to true if MDM is enabled for the host's
    /// platform, MDM migration is enabled for that platform, and the host is
    /// eligible for such a migration (e.g. it is enrolled in a third-party MDM
    /// solution).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub needs_mdm_migration: bool,

    /// NeedsProgrammaticWindowsMDMEnrollment is sent as true if Windows MDM is
    /// enabled and the device should be enrolled as far as the server knows.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub needs_programmatic_windows_mdm_enrollment: bool,

    /// WindowsMDMDiscoveryEndpoint is the URL to use as Windows MDM discovery.
    /// It must be sent when NeedsProgrammaticWindowsMDMEnrollment is true.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub windows_mdm_discovery_endpoint: String,

    /// NeedsProgrammaticWindowsMDMUnenrollment is sent as true if Windows MDM is
    /// disabled and the device was enrolled in Fleet's MDM.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub needs_programmatic_windows_mdm_unenrollment: bool,

    /// PendingScriptExecutionIDs lists the IDs of scripts that are pending
    /// execution on that host.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pending_script_execution_ids: Vec<String>,

    /// EnforceBitLockerEncryption is sent as true if Windows MDM is
    /// enabled and the device should encrypt its disk volumes with BitLocker.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub enforce_bitlocker_encryption: bool,

    /// PendingSoftwareInstallerIDs contains a list of software install_ids queued for installation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pending_software_installer_ids: Vec<String>,

    /// RunSetupExperience indicates whether Orbit should run the Fleet setup experience
    /// during macOS Setup Assistant.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub run_setup_experience: bool,

    /// RunDiskEncryptionEscrow tells Orbit to prompt the end user to escrow disk
    /// encryption data for Linux platforms where disk encryption is supported.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub run_disk_encryption_escrow: bool,
}

/// OrbitConfig holds the configuration returned to an Orbit client.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrbitConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_execution_timeout: Option<i64>,

    #[serde(
        default,
        rename = "command_line_startup_flags",
        skip_serializing_if = "Option::is_none"
    )]
    pub flags: Option<serde_json::Value>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nudge_config: Option<serde_json::Value>,

    #[serde(default)]
    pub notifications: OrbitConfigNotifications,

    /// UpdateChannels contains the TUF channels to use on fleetd components.
    /// If None it means the server isn't using/setting this feature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update_channels: Option<OrbitUpdateChannels>,
}

/// OrbitUpdateChannels hold the update channels that can be configured in fleetd agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitUpdateChannels {
    /// Orbit holds the orbit channel.
    pub orbit: String,
    /// Osqueryd holds the osqueryd channel.
    pub osqueryd: String,
    /// Desktop holds the Fleet Desktop channel.
    pub desktop: String,
}

/// OrbitHostInfo holds device information used during Orbit enroll.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrbitHostInfo {
    /// HardwareUUID is the device's hardware UUID.
    #[serde(default)]
    pub hardware_uuid: String,
    /// HardwareSerial is the device's serial number. Only set for macOS and Linux hosts.
    #[serde(default)]
    pub hardware_serial: String,
    /// Hostname is the device hostname.
    #[serde(default)]
    pub hostname: String,
    /// Platform is the device's platform as defined by osquery's os_version table.
    #[serde(default)]
    pub platform: String,
    /// PlatformLike is the device's platform_like as defined by osquery's os_version table.
    #[serde(default)]
    pub platform_like: String,
    /// OsqueryIdentifier holds the identifier that osqueryd will use in its enrollment.
    #[serde(default)]
    pub osquery_identifier: String,
    /// ComputerName is the device's friendly name (optional).
    #[serde(default)]
    pub computer_name: String,
    /// HardwareModel is the device's hardware model.
    #[serde(default)]
    pub hardware_model: String,
}

/// ExtensionInfo holds the data of an osquery extension to apply to an Orbit client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    /// Platform is one of "windows", "linux" or "macos".
    pub platform: String,
    /// Channel is the selected TUF channel to listen for updates.
    pub channel: String,
    /// Labels are the label names the host must be member of to run this extension.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
}

/// Extensions holds a set of extensions to apply to an Orbit client.
/// The key of the map is the extension name (as defined on the TUF server).
pub type Extensions = HashMap<String, ExtensionInfo>;

/// OrbitHostDiskEncryptionKeyPayload contains the disk encryption key for a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitHostDiskEncryptionKeyPayload {
    pub encryption_key: Vec<u8>,
    pub client_error: String,
}

/// SetupExperienceInitResult is the payload returned when the orbit client manually initiates
/// setup experience for non-darwin platforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupExperienceInitResult {
    pub enabled: bool,
}
