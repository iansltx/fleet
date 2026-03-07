//! Software types matching Go's `server/fleet/software.go` and `server/fleet/software_installer.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::vulnerability::CVE;

// ─── Constants ───────────────────────────────────────────────────────────────

pub const SOFTWARE_FIELD_SEPARATOR: &str = "\u{0000}";
pub const SOFTWARE_NAME_MAX_LENGTH: usize = 255;
pub const SOFTWARE_VERSION_MAX_LENGTH: usize = 255;
pub const SOFTWARE_SOURCE_MAX_LENGTH: usize = 64;
pub const SOFTWARE_BUNDLE_IDENTIFIER_MAX_LENGTH: usize = 255;
pub const SOFTWARE_EXTENSION_ID_MAX_LENGTH: usize = 255;
pub const SOFTWARE_EXTENSION_FOR_MAX_LENGTH: usize = 255;
pub const SOFTWARE_RELEASE_MAX_LENGTH: usize = 64;
pub const SOFTWARE_VENDOR_MAX_LENGTH: usize = 114;
pub const SOFTWARE_ARCH_MAX_LENGTH: usize = 16;
pub const SOFTWARE_TEAM_IDENTIFIER_MAX_LENGTH: usize = 10;
pub const SOFTWARE_TITLE_DISPLAY_NAME_MAX_LENGTH: usize = 255;
pub const UPGRADE_CODE_EXPECTED_LENGTH: usize = 38;
pub const SOFTWARE_VENDOR_MAX_LENGTH_FMT: &str = "%.111s...";
pub const SOFTWARE_INSTALLER_URL_MAX_LENGTH: usize = 4000;
pub const MAX_SOFTWARE_INSTALL_ATTEMPTS: u32 = 3;
pub const BATCH_DOWNLOAD_MAX_RETRIES: u32 = 3;
pub const BATCH_UPLOAD_MAX_RETRIES: u32 = 3;

// ─── Software Installer Output Copy Constants ────────────────────────────────

pub const SOFTWARE_INSTALLER_QUERY_FAIL_COPY: &str =
    "Query didn't return result or failed\nInstall stopped";
pub const SOFTWARE_INSTALLER_QUERY_SUCCESS_COPY: &str =
    "Query returned result\nProceeding to install...";
pub const SOFTWARE_INSTALLER_SCRIPTS_DISABLED_COPY: &str =
    "Installing software...\nError: Scripts are disabled for this host. To run scripts, deploy the fleetd agent with --enable-scripts.";
pub const SOFTWARE_INSTALLER_DOWNLOAD_FAILED_COPY: &str =
    "Installing software...\nError: Software installer download failed.";

/// Special exit code returned by fleetd when install was attempted on a host with scripts disabled.
pub const EXIT_CODE_SCRIPTS_DISABLED: i32 = -2;
/// Special exit code returned by fleetd when fleetd failed to download the installer.
pub const EXIT_CODE_INSTALLER_DOWNLOAD_FAILED: i32 = -3;

// ─── Software ────────────────────────────────────────────────────────────────

/// Software is a named and versioned piece of software installed on a device.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    #[serde(default)]
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
    #[serde(skip)]
    pub title_id: Option<u32>,
    #[serde(skip)]
    pub name_source: String,
    #[serde(skip)]
    pub checksum: String,
    #[serde(skip)]
    pub installed: bool,
    #[serde(skip)]
    pub is_kernel: bool,
}

fn is_zero_i32(v: &i32) -> bool {
    *v == 0
}

/// VulnerableSoftware holds software info along with vulnerability resolution data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerableSoftware {
    pub id: u32,
    pub name: String,
    pub version: String,
    pub source: String,
    pub extension_for: String,
    pub generated_cpe: String,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub hosts_count: i32,
    pub resolved_in_version: Option<String>,
}

/// SoftwareVersion is an abstraction for the software titles APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareVersion {
    pub id: u32,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vulnerabilities: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts_count: Option<u32>,
    #[serde(skip)]
    pub title_id: u32,
}

/// SoftwareTitleSummary contains a lightweight subset of SoftwareTitle fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareTitleSummary {
    pub id: u32,
    pub name: String,
    pub source: String,
    pub extension_for: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgrade_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_id: Option<String>,
}

/// Configuration for auto-updates for a software title.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SoftwareAutoUpdateConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_update_enabled: Option<bool>,
    #[serde(rename = "auto_update_window_start", skip_serializing_if = "Option::is_none")]
    pub auto_update_start_time: Option<String>,
    #[serde(rename = "auto_update_window_end", skip_serializing_if = "Option::is_none")]
    pub auto_update_end_time: Option<String>,
}

/// SoftwareAutoUpdateSchedule ties auto-update config to a title and team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareAutoUpdateSchedule {
    pub title_id: u32,
    pub team_id: u32,
    #[serde(flatten)]
    pub config: SoftwareAutoUpdateConfig,
}

/// FleetMaintainedVersion represents a cached version of a Fleet-maintained app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetMaintainedVersion {
    pub id: u32,
    pub version: String,
}

/// SoftwareTitle represents a title backed by the `software_titles` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareTitle {
    pub id: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    pub source: String,
    pub extension_for: String,
    #[serde(default)]
    pub browser: String,
    pub hosts_count: u32,
    pub versions_count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub versions: Vec<SoftwareVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counts_updated_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub software_installers_count: i32,
    #[serde(skip)]
    pub vpp_apps_count: i32,
    #[serde(skip)]
    pub in_house_app_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_package: Option<SoftwareInstaller>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_store_app: Option<VPPAppStoreApp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_identifier: Option<String>,
    #[serde(skip)]
    pub is_kernel: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgrade_code: Option<String>,
    pub display_name: String,
    #[serde(flatten)]
    pub auto_update_config: SoftwareAutoUpdateConfig,
}

/// SoftwareTitleListResult is used when listing software titles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareTitleListResult {
    pub id: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    pub source: String,
    pub extension_for: String,
    #[serde(default)]
    pub browser: String,
    pub hosts_count: u32,
    pub versions_count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub versions: Vec<SoftwareVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_package: Option<SoftwarePackageOrApp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_store_app: Option<SoftwarePackageOrApp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgrade_code: Option<String>,
    pub display_name: String,
    #[serde(flatten)]
    pub auto_update_config: SoftwareAutoUpdateConfig,
}

/// SoftwareTitleListOptions configures software title listing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SoftwareTitleListOptions {
    #[serde(flatten)]
    pub list_options: crate::ListOptions,
    pub team_id: Option<u32>,
    #[serde(default)]
    pub vulnerable_only: bool,
    #[serde(default)]
    pub available_for_install: bool,
    #[serde(default)]
    pub self_service_only: bool,
    #[serde(default)]
    pub known_exploit: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_cvss: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_cvss: Option<f64>,
    #[serde(default)]
    pub packages_only: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub platform: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub hash_sha256: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub package_name: String,
    #[serde(skip)]
    pub for_setup_experience: bool,
}

/// HostSoftwareTitleListOptions configures per-host software title listing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostSoftwareTitleListOptions {
    #[serde(flatten)]
    pub list_options: crate::ListOptions,
    #[serde(default)]
    pub self_service_only: bool,
    #[serde(default)]
    pub include_available_for_install: bool,
    #[serde(skip)]
    pub include_available_for_install_explicitly_set: bool,
    #[serde(default)]
    pub only_available_for_install: bool,
    #[serde(default)]
    pub vulnerable_only: bool,
    #[serde(default)]
    pub known_exploit: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_cvss: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_cvss: Option<f64>,
    #[serde(skip)]
    pub is_mdm_enrolled: bool,
}

/// SoftwareListOptions configures software listing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SoftwareListOptions {
    #[serde(flatten)]
    pub list_options: crate::ListOptions,
    pub host_id: Option<u32>,
    pub team_id: Option<u32>,
    #[serde(default)]
    pub vulnerable_only: bool,
    #[serde(default)]
    pub without_vulnerability_details: bool,
    #[serde(default)]
    pub include_cve_scores: bool,
    #[serde(default)]
    pub known_exploit: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_cvss: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_cvss: Option<f64>,
    #[serde(default)]
    pub with_host_counts: bool,
}

/// SoftwareIterQueryOptions configures software iteration queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SoftwareIterQueryOptions {
    #[serde(default)]
    pub excluded_sources: Vec<String>,
    #[serde(default)]
    pub included_sources: Vec<String>,
    #[serde(default)]
    pub name_match: String,
    #[serde(default)]
    pub name_exclude: String,
}

/// AuthzSoftwareInventory is used for access controls on software inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthzSoftwareInventory {
    pub team_id: Option<u32>,
}

// ─── Host Software ───────────────────────────────────────────────────────────

/// PathSignatureInformation holds code-signing info for an installed software path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSignatureInformation {
    pub installed_path: String,
    pub team_identifier: String,
    #[serde(rename = "hash_sha256", skip_serializing_if = "Option::is_none")]
    pub cd_hash_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_path: Option<String>,
}

/// HostSoftwareEntry represents a single software entry associated with a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostSoftwareEntry {
    #[serde(flatten)]
    pub software: Software,
    #[serde(default)]
    pub installed_paths: Vec<String>,
    #[serde(default, rename = "signature_information")]
    pub path_signature_information: Vec<PathSignatureInformation>,
}

/// HostSoftware is the set of software installed on a specific host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostSoftware {
    #[serde(default)]
    pub software: Vec<HostSoftwareEntry>,
    pub software_updated_at: DateTime<Utc>,
}

/// UpdateHostSoftwareDBResult stores the result of calling UpdateHostSoftware.
#[derive(Debug, Clone, Default)]
pub struct UpdateHostSoftwareDBResult {
    pub was_curr_installed: Vec<Software>,
    pub deleted: Vec<Software>,
    pub inserted: Vec<Software>,
}

// ─── Software Installer ──────────────────────────────────────────────────────

/// SoftwareInstallerURL contains details to download the software installer from CDN.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInstallerURL {
    pub url: String,
    pub filename: String,
}

/// SoftwareInstallDetails contains all info for a client to install software.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInstallDetails {
    #[serde(skip)]
    pub host_id: u32,
    #[serde(rename = "install_id")]
    pub execution_id: String,
    pub installer_id: u32,
    pub pre_install_condition: String,
    pub install_script: String,
    pub uninstall_script: String,
    pub post_install_script: String,
    pub self_service: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installer_url: Option<SoftwareInstallerURL>,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub max_retries: u32,
}

fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}

/// SoftwareInstaller represents a software installer package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInstaller {
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_id: Option<u32>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip)]
    pub extension: String,
    pub version: String,
    pub platform: String,
    #[serde(skip)]
    pub package_id_list: String,
    #[serde(skip)]
    pub upgrade_code: String,
    pub uploaded_at: DateTime<Utc>,
    pub installer_id: u32,
    pub install_script: String,
    #[serde(skip)]
    pub install_script_content_id: u32,
    #[serde(skip)]
    pub uninstall_script_content_id: u32,
    pub pre_install_query: String,
    pub post_install_script: String,
    pub uninstall_script: String,
    #[serde(skip)]
    pub post_install_script_content_id: Option<u32>,
    #[serde(rename = "hash_sha256")]
    pub storage_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<SoftwareInstallerStatusSummary>,
    #[serde(skip)]
    pub software_title: String,
    pub self_service: bool,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fleet_maintained_app_id: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fleet_maintained_versions: Vec<FleetMaintainedVersion>,
    #[serde(default)]
    pub automatic_install_policies: Vec<AutomaticInstallPolicy>,
    #[serde(default)]
    pub labels_include_any: Vec<SoftwareScopeLabel>,
    #[serde(default)]
    pub labels_exclude_any: Vec<SoftwareScopeLabel>,
    #[serde(skip)]
    pub source: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(skip)]
    pub bundle_identifier: String,
    pub display_name: String,
}

/// SoftwareInstallerStatusSummary represents aggregated status metrics for a software installer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInstallerStatusSummary {
    pub installed: u32,
    pub pending_install: u32,
    pub failed_install: u32,
    pub pending_uninstall: u32,
    pub failed_uninstall: u32,
}

/// SoftwareInstallerStatus represents the status of a software installer on a host.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SoftwareInstallerStatus {
    #[serde(rename = "pending_install")]
    PendingInstall,
    #[serde(rename = "failed_install")]
    FailedInstall,
    #[serde(rename = "installed")]
    Installed,
    #[serde(rename = "pending_uninstall")]
    PendingUninstall,
    #[serde(rename = "failed_uninstall")]
    FailedUninstall,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "failed")]
    Failed,
}

/// SoftwareScopeLabel represents the many-to-many relationship between software titles and labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareScopeLabel {
    #[serde(rename = "name")]
    pub label_name: String,
    #[serde(rename = "id")]
    pub label_id: u32,
    #[serde(skip)]
    pub exclude: bool,
    #[serde(skip)]
    pub title_id: u32,
}

/// AutomaticInstallPolicy represents a policy that triggers automatic installation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomaticInstallPolicy {
    pub id: u32,
    pub name: String,
    #[serde(skip)]
    pub title_id: u32,
}

/// SoftwarePackageOrApp provides information about a software installer package or VPP app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwarePackageOrApp {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub app_store_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(default)]
    pub automatic_install_policies: Vec<AutomaticInstallPolicy>,
    pub version: String,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_service: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_install: Option<HostSoftwareInstall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_uninstall: Option<HostSoftwareUninstall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_during_setup: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fleet_maintained_app_id: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fleet_maintained_versions: Vec<FleetMaintainedVersion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
}

/// HostSoftwareInstall represents installation of software on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostSoftwareInstall {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub install_uuid: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub command_uuid: String,
    pub installed_at: DateTime<Utc>,
}

/// HostSoftwareUninstall represents uninstallation of software from a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostSoftwareUninstall {
    #[serde(rename = "script_execution_id", default, skip_serializing_if = "String::is_empty")]
    pub execution_id: String,
    pub uninstalled_at: DateTime<Utc>,
}

/// HostSoftwareInstalledVersion represents a version of software installed on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostSoftwareInstalledVersion {
    #[serde(skip)]
    pub software_id: u32,
    #[serde(skip)]
    pub software_title_id: u32,
    #[serde(skip)]
    pub source: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bundle_identifier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_opened_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub vulnerabilities: Vec<String>,
    #[serde(default)]
    pub installed_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signature_information: Vec<PathSignatureInformation>,
}

/// HostSoftwareWithInstaller represents software on a host with installer information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostSoftwareWithInstaller {
    pub id: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    pub source: String,
    pub extension_for: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<SoftwareInstallerStatus>,
    #[serde(default)]
    pub installed_versions: Vec<HostSoftwareInstalledVersion>,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgrade_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_package: Option<SoftwarePackageOrApp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_store_app: Option<SoftwarePackageOrApp>,
}

/// HostSoftwareInstallerResult represents a software install/uninstall result on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostSoftwareInstallerResult {
    #[serde(skip)]
    pub id: u32,
    pub install_uuid: String,
    pub software_title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_title_id: Option<u32>,
    #[serde(skip)]
    pub software_installer_id: Option<u32>,
    pub software_package: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub host_id: u32,
    pub status: SoftwareInstallerStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_install_query_output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_install_script_output: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub user_id: Option<u32>,
    #[serde(skip)]
    pub install_script_exit_code: Option<i32>,
    #[serde(skip)]
    pub post_install_script_exit_code: Option<i32>,
    pub self_service: bool,
    #[serde(skip)]
    pub host_deleted_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempt_number: Option<i32>,
}

/// SoftwareInstallResult represents the result of a software install/uninstall on a host (legacy).
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

/// HostSoftwareInstallResultPayload is the payload provided by fleetd for install results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostSoftwareInstallResultPayload {
    pub host_id: u32,
    pub install_uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_install_condition_output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_script_exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_script_output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_install_script_exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_install_script_output: Option<String>,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub retries_remaining: u32,
}

/// SoftwarePackageResponse is the response type used when applying software by batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwarePackageResponse {
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_id: Option<u32>,
    pub url: String,
    pub hash_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fleet_maintained_app_id: Option<u32>,
    #[serde(default)]
    pub fleet_maintained_app_slug: String,
    #[serde(default)]
    pub icon_hash_sha256: String,
    #[serde(default)]
    pub icon_filename: String,
    #[serde(skip)]
    pub local_icon_hash: String,
    #[serde(skip)]
    pub local_icon_path: String,
}

/// VPPAppResponse is the response type used when applying app store apps by batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPAppResponse {
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_id: Option<u32>,
    pub app_store_id: String,
    pub platform: String,
    #[serde(default)]
    pub icon_hash_sha256: String,
    #[serde(default)]
    pub icon_filename: String,
    #[serde(skip)]
    pub local_icon_hash: String,
    #[serde(skip)]
    pub local_icon_path: String,
    #[serde(skip)]
    pub app_team_id: u32,
}

/// SoftwareInstallerTokenMetadata is the metadata stored in Redis for a software installer token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInstallerTokenMetadata {
    pub title_id: u32,
    pub team_id: u32,
}

/// HostLastInstallData contains data for the last installation of a package on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostLastInstallData {
    pub execution_id: String,
    pub status: Option<SoftwareInstallerStatus>,
}

/// HostSoftwareInstallOptions contains options for a software or VPP app install request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostSoftwareInstallOptions {
    #[serde(default)]
    pub self_service: bool,
    pub policy_id: Option<u32>,
    #[serde(default)]
    pub for_setup_experience: bool,
    #[serde(default)]
    pub for_scheduled_updates: bool,
    pub user_id: Option<u32>,
    #[serde(default)]
    pub with_retries: bool,
}

// ─── SoftwareInstallerPayload (from scripts.go) ─────────────────────────────

/// SoftwareInstallerPayload is used for creating software installers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInstallerPayload {
    pub url: String,
    pub pre_install_query: String,
    pub install_script: String,
    pub uninstall_script: String,
    pub post_install_script: String,
    pub self_service: bool,
    #[serde(skip)]
    pub fleet_maintained: bool,
    #[serde(skip)]
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_during_setup: Option<bool>,
    #[serde(default)]
    pub labels_include_any: Vec<String>,
    #[serde(default)]
    pub labels_exclude_any: Vec<String>,
    pub sha256: String,
    #[serde(default)]
    pub categories: Vec<String>,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(rename = "fleet_maintained_app_version", default)]
    pub rollback_version: String,
    #[serde(skip)]
    pub icon_path: String,
    #[serde(skip)]
    pub icon_hash: String,
}

// ─── VPP Types ───────────────────────────────────────────────────────────────

/// VPPAppID is a unique identifier for a VPP application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPAppID {
    #[serde(rename = "app_store_id")]
    pub adam_id: String,
    pub platform: String,
}

/// VPPAppTeam contains extra metadata injected by fleet for a VPP app team assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPAppTeam {
    #[serde(flatten)]
    pub vpp_app_id: VPPAppID,
    #[serde(skip)]
    pub app_team_id: u32,
    pub self_service: bool,
    #[serde(skip)]
    pub install_during_setup: Option<bool>,
    #[serde(default)]
    pub labels_include_any: Vec<String>,
    #[serde(default)]
    pub labels_exclude_any: Vec<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<serde_json::Value>,
}

/// VPPApp represents a VPP (Volume Purchase Program) application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPApp {
    #[serde(flatten)]
    pub vpp_app_team: VPPAppTeam,
    pub bundle_identifier: String,
    pub icon_url: String,
    pub name: String,
    pub latest_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip)]
    pub title_id: u32,
    #[serde(skip)]
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    pub updated_at: DateTime<Utc>,
}

/// VPPAppStoreApp contains the fields for a VPP app on the get software title endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPAppStoreApp {
    #[serde(flatten)]
    pub vpp_app_id: VPPAppID,
    pub name: String,
    pub latest_version: String,
    #[serde(skip)]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<VPPAppStatusSummary>,
    pub self_service: bool,
    #[serde(skip)]
    pub vpp_apps_teams_id: u32,
    #[serde(default)]
    pub automatic_install_policies: Vec<AutomaticInstallPolicy>,
    #[serde(default)]
    pub labels_include_any: Vec<SoftwareScopeLabel>,
    #[serde(default)]
    pub labels_exclude_any: Vec<SoftwareScopeLabel>,
    #[serde(skip)]
    pub bundle_identifier: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub categories: Vec<String>,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<serde_json::Value>,
}

/// VPPAppStatusSummary represents aggregated status metrics for a VPP app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPAppStatusSummary {
    pub installed: u32,
    pub pending: u32,
    pub failed: u32,
}

/// VPPBatchPayload is the payload for batch VPP app operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPBatchPayload {
    pub app_store_id: String,
    pub self_service: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_during_setup: Option<bool>,
    #[serde(default)]
    pub labels_exclude_any: Vec<String>,
    #[serde(default)]
    pub labels_include_any: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub display_name: String,
    #[serde(skip)]
    pub icon_path: String,
    #[serde(skip)]
    pub icon_hash: String,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_update_enabled: Option<bool>,
    #[serde(rename = "auto_update_window_start", skip_serializing_if = "Option::is_none")]
    pub auto_update_start_time: Option<String>,
    #[serde(rename = "auto_update_window_end", skip_serializing_if = "Option::is_none")]
    pub auto_update_end_time: Option<String>,
}

/// VPPToken represents a VPP token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPToken {
    pub id: u32,
    pub org_name: String,
    pub location: String,
    pub renew_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// HostVPPSoftwareInstall represents a VPP software install attempt on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostVPPSoftwareInstall {
    pub command_uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ack_at: Option<DateTime<Utc>>,
    pub host_id: u32,
    pub install_command_status: String,
    pub bundle_identifier: String,
    pub retry_count: i32,
    pub expected_version: String,
}

// ─── Fleet Maintained App ────────────────────────────────────────────────────

/// MaintainedApp represents an app in the Fleet library of maintained apps.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaintainedApp {
    pub id: u32,
    pub name: String,
    pub slug: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub version: String,
    pub platform: String,
    #[serde(rename = "software_title_id", skip_serializing_if = "Option::is_none")]
    pub title_id: Option<u32>,
    #[serde(rename = "url", default, skip_serializing_if = "String::is_empty")]
    pub installer_url: String,
    #[serde(skip)]
    pub sha256: String,
    #[serde(skip)]
    pub unique_identifier: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub install_script: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub uninstall_script: String,
    #[serde(skip)]
    pub automatic_install_query: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub upgrade_code: String,
}

/// FleetMaintainedApp represents a Fleet-maintained app in the library (legacy alias).
pub type FleetMaintainedApp = MaintainedApp;

/// SoftwareCategory represents a category for software titles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareCategory {
    pub id: u32,
    pub name: String,
}

/// SoftwareSpec is used for gitops software specifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareSpec {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub app_store_apps: Vec<serde_json::Value>,
}

// ─── Agent Options (from agent_options.go) ───────────────────────────────────

/// AgentOptions represents osquery agent options configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOptions {
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub script_execution_timeout: i32,
    pub config: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overrides: Option<AgentOptionsOverrides>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_line_flags: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update_channels: Option<serde_json::Value>,
}

/// AgentOptionsOverrides includes any platform-based overrides.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentOptionsOverrides {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platforms: Option<HashMap<String, serde_json::Value>>,
}

// ─── In-House Apps (from in_house_apps.go) ───────────────────────────────────

/// InHouseAppPayload represents the payload for an in-house (.ipa) app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InHouseAppPayload {
    pub team_id: Option<u32>,
    pub title: String,
    pub filename: String,
    pub bundle_id: String,
    pub storage_id: String,
    pub platform: String,
    #[serde(default)]
    pub category_ids: Vec<u32>,
    pub version: String,
    #[serde(default)]
    pub self_service: bool,
}

// ─── Secret Variables (from secret_variables.go) ─────────────────────────────

/// SecretVariable represents a fleet secret variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretVariable {
    pub name: String,
    pub value: String,
    #[serde(skip)]
    pub updated_at: Option<DateTime<Utc>>,
}

/// SecretVariableIdentifier holds identifier information about a secret variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretVariableIdentifier {
    pub id: u32,
    pub name: String,
    pub updated_at: String,
}

// ─── Network Interface (from network_interfaces.go) ──────────────────────────

/// NetworkInterface represents a network interface on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub id: u32,
    #[serde(skip)]
    pub host_id: u32,
    pub interface: String,
    #[serde(rename = "address")]
    pub ip_address: String,
    pub mask: String,
    pub broadcast: String,
    pub point_to_point: String,
    pub mac: String,
    #[serde(rename = "type")]
    pub iface_type: i32,
    pub mtu: i32,
    pub metric: i32,
    pub ipackets: i64,
    pub opackets: i64,
    pub ibytes: i64,
    pub obytes: i64,
    pub ierrors: i64,
    pub oerrors: i64,
    pub last_change: i64,
}

// ─── Installer (from installer.go) ──────────────────────────────────────────

/// Installer describes an installer in an S3 bucket.
#[derive(Debug, Clone)]
pub struct Installer {
    pub enroll_secret: String,
    pub kind: String,
    pub desktop: bool,
    // Content is intentionally omitted (io.ReadSeeker in Go).
}

// ─── DigiCert Certificate (from digicert.go) ────────────────────────────────

/// DigiCertCertificate represents a certificate obtained from DigiCert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigiCertCertificate {
    pub pfx_data: Vec<u8>,
    pub password: String,
    pub not_valid_before: DateTime<Utc>,
    pub not_valid_after: DateTime<Utc>,
    pub serial_number: String,
}

// ─── EST Certificate (from est_ca.go) ───────────────────────────────────────

/// ESTCertificate represents a certificate obtained from an EST CA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ESTCertificate {
    pub certificate: Vec<u8>,
}

/// AppStoreAppUpdatePayload is the payload for updating an App Store app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStoreAppUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_service: Option<bool>,
    #[serde(default)]
    pub labels_include_any: Vec<String>,
    #[serde(default)]
    pub labels_exclude_any: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<serde_json::Value>,
    #[serde(flatten)]
    pub auto_update_config: SoftwareAutoUpdateConfig,
}

// ─── VulnSoftwareFilter ─────────────────────────────────────────────────────

/// VulnSoftwareFilter filters software for vulnerability scanning.
#[derive(Debug, Clone, Default)]
pub struct VulnSoftwareFilter {
    pub host_id: Option<u32>,
    /// LIKE filter on the software name.
    pub name: String,
    /// Exact match on the software source.
    pub source: String,
    /// Filter to kernel packages only (for RHEL goval-dictionary scanning).
    pub kernels_only: bool,
}

// ─── SoftwareAutoUpdateScheduleFilter ────────────────────────────────────────

/// SoftwareAutoUpdateScheduleFilter filters auto-update schedules.
#[derive(Debug, Clone, Default)]
pub struct SoftwareAutoUpdateScheduleFilter {
    pub enabled: Option<bool>,
}

// ─── SoftwareInstallerStore trait ────────────────────────────────────────────

/// SoftwareInstallerStore is the interface to store and retrieve software
/// installer files. Fleet supports storing to the local filesystem and to an
/// S3 bucket.
///
/// Note: This is a simplified Rust trait version of the Go interface. The actual
/// I/O types differ from Go's io.ReadCloser/io.ReadSeeker.
pub trait SoftwareInstallerStore: Send + Sync {
    /// Get retrieves the installer content by ID, returning a reader and the content length.
    fn get(
        &self,
        installer_id: &str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<(Box<dyn std::io::Read + Send>, i64), Box<dyn std::error::Error>>,
                > + Send,
        >,
    >;

    /// Put stores installer content by ID.
    fn put(
        &self,
        installer_id: &str,
        content: &[u8],
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send>,
    >;

    /// Exists checks if an installer exists by ID.
    fn exists(
        &self,
        installer_id: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<bool, Box<dyn std::error::Error>>> + Send>,
    >;

    /// Cleanup removes unused installers created before the given time.
    fn cleanup(
        &self,
        used_installer_ids: &[String],
        remove_created_before: chrono::DateTime<chrono::Utc>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<i32, Box<dyn std::error::Error>>> + Send>,
    >;

    /// Sign generates a signed URL for the installer.
    fn sign(
        &self,
        file_id: &str,
        expires_in: std::time::Duration,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, Box<dyn std::error::Error>>> + Send>,
    >;
}

// ─── UploadSoftwareInstallerPayload ──────────────────────────────────────────

/// UploadSoftwareInstallerPayload is used for creating software installers.
#[derive(Debug, Clone, Default)]
pub struct UploadSoftwareInstallerPayload {
    pub team_id: Option<u32>,
    pub install_script: String,
    pub pre_install_query: String,
    pub post_install_script: String,
    pub storage_id: String,
    pub filename: String,
    pub title: String,
    pub version: String,
    pub source: String,
    pub platform: String,
    pub bundle_identifier: String,
    pub self_service: bool,
    pub user_id: u32,
    pub url: String,
    pub fleet_maintained_app_id: Option<u32>,
    /// RollbackVersion is the version to pin as "active" for a fleet-maintained app.
    pub rollback_version: String,
    /// FMAVersionCached indicates this FMA version is already cached.
    pub fma_version_cached: bool,
    pub package_ids: Vec<String>,
    pub upgrade_code: String,
    pub uninstall_script: String,
    pub extension: String,
    /// Keep saved value if None, otherwise set as indicated.
    pub install_during_setup: Option<bool>,
    /// Names of "include any" labels.
    pub labels_include_any: Vec<String>,
    /// Names of "exclude any" labels.
    pub labels_exclude_any: Vec<String>,
    pub automatic_install: bool,
    pub automatic_install_query: String,
    pub categories: Vec<String>,
    pub category_ids: Vec<u32>,
    pub display_name: String,
}

// ─── UpdateSoftwareInstallerPayload ──────────────────────────────────────────

/// UpdateSoftwareInstallerPayload is used for updating software installers.
#[derive(Debug, Clone, Default)]
pub struct UpdateSoftwareInstallerPayload {
    pub title_id: u32,
    pub team_id: Option<u32>,
    pub installer_id: u32,
    pub user_id: u32,
    pub install_script: Option<String>,
    pub pre_install_query: Option<String>,
    pub post_install_script: Option<String>,
    pub self_service: Option<bool>,
    pub uninstall_script: Option<String>,
    pub storage_id: String,
    pub filename: String,
    pub version: String,
    pub package_ids: Vec<String>,
    pub upgrade_code: String,
    /// Names of "include any" labels.
    pub labels_include_any: Vec<String>,
    /// Names of "exclude any" labels.
    pub labels_exclude_any: Vec<String>,
    pub categories: Vec<String>,
    pub category_ids: Vec<u32>,
    /// DisplayName is an end-user friendly name.
    pub display_name: Option<String>,
}

// ─── ExistingSoftwareInstaller ───────────────────────────────────────────────

/// ExistingSoftwareInstaller holds data about an existing software installer.
#[derive(Debug, Clone, Default)]
pub struct ExistingSoftwareInstaller {
    pub installer_id: u32,
    pub team_id: Option<u32>,
    pub filename: String,
    pub extension: String,
    pub version: String,
    pub platform: String,
    pub source: String,
    pub bundle_identifier: Option<String>,
    pub title: String,
    pub package_id_list: String,
    pub package_ids: Vec<String>,
}

// ─── HostSoftwareInstallerResultAuthz ────────────────────────────────────────

/// HostSoftwareInstallerResultAuthz is used for authorization of host software installer results.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostSoftwareInstallerResultAuthz {
    pub host_team_id: Option<u32>,
}

// ─── SoftwarePackageSpec ─────────────────────────────────────────────────────

/// SoftwarePackageSpec is used for gitops software package specifications.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SoftwarePackageSpec {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub self_service: bool,
    #[serde(default)]
    pub pre_install_query: serde_json::Value,
    #[serde(default)]
    pub install_script: serde_json::Value,
    #[serde(default)]
    pub post_install_script: serde_json::Value,
    #[serde(default)]
    pub uninstall_script: serde_json::Value,
    #[serde(default)]
    pub labels_include_any: Vec<String>,
    #[serde(default)]
    pub labels_exclude_any: Vec<String>,
    #[serde(rename = "setup_experience", skip_serializing_if = "Option::is_none")]
    pub install_during_setup: Option<bool>,
    #[serde(default)]
    pub icon: serde_json::Value,
    /// FMA slug.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(default)]
    pub version: String,
    /// Resolved path of the file used to fill the software package.
    #[serde(default)]
    pub referenced_yaml_path: String,
    #[serde(default)]
    pub hash_sha256: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub display_name: String,
}

// ─── MaintainedAppSpec ───────────────────────────────────────────────────────

/// MaintainedAppSpec is used for gitops fleet-maintained app specifications.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaintainedAppSpec {
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub self_service: bool,
    #[serde(default)]
    pub pre_install_query: serde_json::Value,
    #[serde(default)]
    pub install_script: serde_json::Value,
    #[serde(default)]
    pub post_install_script: serde_json::Value,
    #[serde(default)]
    pub uninstall_script: serde_json::Value,
    #[serde(default)]
    pub labels_include_any: Vec<String>,
    #[serde(default)]
    pub labels_exclude_any: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(rename = "setup_experience", skip_serializing_if = "Option::is_none")]
    pub install_during_setup: Option<bool>,
    #[serde(default)]
    pub icon: serde_json::Value,
}

// ─── Icon management types ───────────────────────────────────────────────────

/// IconFileUpdate holds a title ID and file path for icon upload.
#[derive(Debug, Clone, Default)]
pub struct IconFileUpdate {
    pub title_id: u32,
    pub path: String,
}

/// IconMetaUpdate holds a title ID, path, and hash for icon metadata update.
#[derive(Debug, Clone, Default)]
pub struct IconMetaUpdate {
    pub title_id: u32,
    pub path: String,
    pub hash: String,
}

/// IconGitOpsSettings holds configuration for icon processing during gitops.
#[derive(Debug, Clone, Default)]
pub struct IconGitOpsSettings {
    pub concurrent_uploads: i32,
    pub concurrent_updates: i32,
    pub uploaded_hashes: Vec<String>,
}

/// IconChanges holds the set of icon changes to apply.
#[derive(Debug, Clone, Default)]
pub struct IconChanges {
    pub team_id: u32,
    pub uploaded_hashes: Vec<String>,
    pub icons_to_upload: Vec<IconFileUpdate>,
    pub icons_to_update: Vec<IconMetaUpdate>,
    pub title_ids_to_remove_icons_from: Vec<u32>,
}

// ─── VPPBatchPayloadWithPlatform ─────────────────────────────────────────────

/// VPPBatchPayloadWithPlatform is the payload for batch VPP app operations with resolved platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPBatchPayloadWithPlatform {
    pub app_store_id: String,
    pub self_service: bool,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_during_setup: Option<bool>,
    #[serde(default)]
    pub labels_exclude_any: Vec<String>,
    #[serde(default)]
    pub labels_include_any: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(skip)]
    pub category_ids: Vec<u32>,
    #[serde(default)]
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_update_enabled: Option<bool>,
    #[serde(rename = "auto_update_window_start", skip_serializing_if = "Option::is_none")]
    pub auto_update_start_time: Option<String>,
    #[serde(rename = "auto_update_window_end", skip_serializing_if = "Option::is_none")]
    pub auto_update_end_time: Option<String>,
}

// ─── Helper implementations ──────────────────────────────────────────────────

impl SoftwareIterQueryOptions {
    /// IsValid checks that either excluded or included sources is specified but not both.
    pub fn is_valid(&self) -> bool {
        !(self.included_sources.len() != 0 && self.excluded_sources.len() != 0)
    }
}

impl UpdateHostSoftwareDBResult {
    /// Returns all software that should be currently installed on the host.
    pub fn curr_installed(&self) -> Vec<Software> {
        let delete_set: std::collections::HashSet<u32> =
            self.deleted.iter().map(|d| d.id).collect();
        let mut result: Vec<Software> = self
            .was_curr_installed
            .iter()
            .filter(|c| !delete_set.contains(&c.id))
            .cloned()
            .collect();
        result.extend(self.inserted.iter().cloned());
        result
    }
}

impl SoftwareInstallerStatus {
    /// Returns true if the status is a valid software installer status.
    pub fn is_valid(&self) -> bool {
        matches!(
            self,
            SoftwareInstallerStatus::PendingInstall
                | SoftwareInstallerStatus::FailedInstall
                | SoftwareInstallerStatus::Installed
                | SoftwareInstallerStatus::PendingUninstall
                | SoftwareInstallerStatus::FailedUninstall
                | SoftwareInstallerStatus::Pending
                | SoftwareInstallerStatus::Failed
        )
    }
}
