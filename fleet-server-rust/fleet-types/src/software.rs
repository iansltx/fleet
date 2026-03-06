//! Software types matching Go's `server/fleet/software.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::vulnerability::CVE;

/// Software is a named and versioned piece of software installed on a device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Software {
    pub id: u32,
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bundle_identifier: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub extension_id: String,
    pub extension_for: String,
    pub browser: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub release: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub vendor: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub arch: String,
    pub generated_cpe: String,
    #[serde(default)]
    pub vulnerabilities: Vec<CVE>,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub hosts_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_opened_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgrade_code: Option<String>,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_id: Option<u32>,
}

fn is_zero_i32(v: &i32) -> bool {
    *v == 0
}

/// HostSoftwareEntry represents a single software entry associated with a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostSoftwareEntry {
    #[serde(flatten)]
    pub software: Software,
    // Additional per-host software fields can be added here.
}

/// SoftwareTitle represents a unique combination of software name and source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareTitle {
    pub id: u32,
    pub name: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser: Option<String>,
    pub hosts_count: u32,
    pub versions_count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub versions: Vec<SoftwareTitleVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counts_updated_at: Option<DateTime<Utc>>,
}

/// SoftwareTitleVersion represents a specific version of a software title.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareTitleVersion {
    pub id: u32,
    pub version: String,
    #[serde(default)]
    pub vulnerabilities: Vec<String>,
    pub hosts_count: u32,
}

/// SoftwareInstallerStatus represents the status of a software installer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SoftwareInstallerStatus {
    #[serde(rename = "installed")]
    Installed,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "failed")]
    Failed,
}

/// FleetMaintainedApp represents a Fleet-maintained app in the library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetMaintainedApp {
    pub id: u32,
    pub name: String,
    pub slug: String,
    pub platform: String,
    pub unique_identifier: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// SoftwareInstallResult represents the result of a software install/uninstall on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInstallResult {
    pub execution_id: String,
    pub host_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_installer_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_title_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_script_exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_script_output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_install_query_output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_install_script_exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_install_script_output: Option<String>,
    pub self_service: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// SoftwareTitleListOptions configures software title listing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SoftwareTitleListOptions {
    #[serde(flatten)]
    pub list_options: crate::ListOptions,
    pub team_id: Option<u32>,
    pub vulnerable_only: bool,
    pub available_for_install: bool,
    pub self_service_only: bool,
    pub known_exploit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_cvss: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_cvss: Option<f64>,
    pub packages_only: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub platform: String,
}

/// SoftwareListOptions configures software listing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SoftwareListOptions {
    #[serde(flatten)]
    pub list_options: crate::ListOptions,
    pub host_id: Option<u32>,
    pub team_id: Option<u32>,
    pub vulnerable_only: bool,
    pub known_exploit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_cvss: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_cvss: Option<f64>,
    pub with_host_counts: bool,
}

/// HostSoftwareTitleListOptions configures per-host software title listing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostSoftwareTitleListOptions {
    #[serde(flatten)]
    pub list_options: crate::ListOptions,
    pub self_service_only: bool,
    pub vulnerable_only: bool,
    pub known_exploit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_cvss: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_cvss: Option<f64>,
}

/// VPPApp represents a VPP (Volume Purchase Program) app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPApp {
    pub adam_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_version: Option<String>,
    pub platform: String,
    pub self_service: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// VPPAppTeam represents a VPP app assignment to a team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPAppTeam {
    pub vpp_app_id: u32,
    pub team_id: Option<u32>,
    pub adam_id: String,
    pub platform: String,
    pub self_service: bool,
}

/// VPPToken represents a VPP token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPToken {
    pub id: u32,
    pub org_name: String,
    pub location: String,
    pub renew_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// SoftwareCategory represents a category for software titles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareCategory {
    pub id: u32,
    pub name: String,
}

/// SoftwareInstallDetails holds details for a software install operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInstallDetails {
    pub host_id: u32,
    pub execution_id: String,
    pub installer_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_install_condition: Option<String>,
    pub install_script: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_install_script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uninstall_script: Option<String>,
    pub self_service: bool,
}

/// SoftwareSpec is used for gitops software specifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareSpec {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub app_store_apps: Vec<serde_json::Value>,
}

// Software field length constants
pub const SOFTWARE_NAME_MAX_LENGTH: usize = 255;
pub const SOFTWARE_VERSION_MAX_LENGTH: usize = 255;
pub const SOFTWARE_SOURCE_MAX_LENGTH: usize = 64;
pub const SOFTWARE_BUNDLE_IDENTIFIER_MAX_LENGTH: usize = 255;
pub const SOFTWARE_VENDOR_MAX_LENGTH: usize = 114;
