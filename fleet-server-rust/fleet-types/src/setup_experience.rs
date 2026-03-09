//! Setup Experience types matching Go's `server/fleet/setup_experience.go`.

use serde::{Deserialize, Serialize};

use crate::host::MDMBootstrapPackageStatus;
use crate::mdm::MDMDeliveryStatus;
use crate::software::SoftwareInstallerStatus;

// ─── SetupExperienceStatusResultStatus ───────────────────────────────────────

/// The status of a particular step in the setup experience process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SetupExperienceStatusResultStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failure")]
    Failure,
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl SetupExperienceStatusResultStatus {
    /// Returns true if the status is one of the known valid statuses
    /// (pending, running, success, failure).
    pub fn is_valid(&self) -> bool {
        matches!(
            self,
            Self::Pending | Self::Running | Self::Success | Self::Failure
        )
    }

    /// Returns true if the status is a terminal status (success or failure).
    pub fn is_terminal_status(&self) -> bool {
        matches!(self, Self::Success | Self::Failure)
    }
}

impl std::fmt::Display for SetupExperienceStatusResultStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Success => write!(f, "success"),
            Self::Failure => write!(f, "failure"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

// ─── SetupExperienceStatusResult ─────────────────────────────────────────────

/// Represents the status of a particular step in the macOS setup experience
/// process for a particular host. These steps can either be a software installer
/// installation, a VPP app installation, or a script execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupExperienceStatusResult {
    #[serde(skip)]
    pub id: u64,
    #[serde(skip)]
    pub host_uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<SetupExperienceStatusResultStatus>,
    #[serde(skip)]
    pub software_installer_id: Option<u64>,
    #[serde(skip)]
    pub host_software_installs_execution_id: Option<String>,
    #[serde(skip)]
    pub vpp_app_team_id: Option<u64>,
    #[serde(skip)]
    pub vpp_app_adam_id: Option<String>,
    #[serde(skip)]
    pub vpp_app_platform: Option<String>,
    #[serde(skip)]
    pub nano_command_uuid: Option<String>,
    #[serde(skip)]
    pub setup_experience_script_id: Option<u64>,
    #[serde(skip)]
    pub script_content_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_title_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
}

impl SetupExperienceStatusResult {
    /// Returns true if this result is for a setup experience script step.
    pub fn is_for_script(&self) -> bool {
        self.setup_experience_script_id.is_some()
    }

    /// Returns true if this result is for a setup experience software step:
    /// either a software installer or a VPP app.
    pub fn is_for_software(&self) -> bool {
        self.vpp_app_team_id.is_some() || self.software_installer_id.is_some()
    }

    /// Returns true if this result is for a setup experience software installer step.
    pub fn is_for_software_package(&self) -> bool {
        self.software_installer_id.is_some()
    }
}

// ─── SetupExperienceBootstrapPackageResult ───────────────────────────────────

/// Result status of the bootstrap package step in the setup experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupExperienceBootstrapPackageResult {
    pub name: String,
    pub status: MDMBootstrapPackageStatus,
}

// ─── SetupExperienceConfigurationProfileResult ───────────────────────────────

/// Result status of a configuration profile step in the setup experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupExperienceConfigurationProfileResult {
    pub profile_uuid: String,
    pub name: String,
    pub status: MDMDeliveryStatus,
}

// ─── SetupExperienceAccountConfigurationResult ───────────────────────────────

/// Result status of the account configuration step in the setup experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupExperienceAccountConfigurationResult {
    pub command_uuid: String,
    pub status: String,
}

// ─── SetupExperienceVPPInstallResult ─────────────────────────────────────────

/// Internal result for a VPP app installation during setup experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupExperienceVPPInstallResult {
    pub host_uuid: String,
    pub command_uuid: String,
    pub command_status: String,
}

// ─── SetupExperienceSoftwareInstallResult ────────────────────────────────────

/// Internal result for a software installer installation during setup experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupExperienceSoftwareInstallResult {
    pub host_uuid: String,
    pub execution_id: String,
    pub installer_status: SoftwareInstallerStatus,
}

// ─── SetupExperienceScriptResult ─────────────────────────────────────────────

/// Internal result for a script execution during setup experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupExperienceScriptResult {
    pub host_uuid: String,
    pub execution_id: String,
    pub exit_code: i32,
}

// ─── SetupExperienceStatusPayload ────────────────────────────────────────────

/// Payload sent to Orbit to communicate the current status of the setup
/// experience for a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupExperienceStatusPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<SetupExperienceStatusResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software: Option<Vec<SetupExperienceStatusResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bootstrap_package: Option<SetupExperienceBootstrapPackageResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_profiles: Option<Vec<SetupExperienceConfigurationProfileResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_configuration: Option<SetupExperienceAccountConfigurationResult>,
    pub org_logo_url: String,
    pub require_all_software: bool,
}

// ─── DeviceSetupExperienceStatusPayload ──────────────────────────────────────

/// Holds the status of the "Setup experience" for a device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSetupExperienceStatusPayload {
    /// Software holds the status of the software to install on the device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software: Option<Vec<SetupExperienceStatusResult>>,
    /// Scripts holds the status of the scripts to run on the device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripts: Option<Vec<SetupExperienceStatusResult>>,
}

// ─── SetupExperienceCount ────────────────────────────────────────────────────

/// Counts of setup experience items by type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupExperienceCount {
    pub installers: u64,
    pub scripts: u64,
    pub vpp: u64,
}

// ─── Constants ───────────────────────────────────────────────────────────────

/// Platforms that support setup experience.
pub const SETUP_EXPERIENCE_SUPPORTED_PLATFORMS: &[&str] =
    &["macos", "ios", "ipados", "windows", "linux", "android"];
