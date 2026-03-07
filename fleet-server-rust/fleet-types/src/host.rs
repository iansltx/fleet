//! Host types matching Go's `server/fleet/hosts.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::policy::HostPolicy;
use crate::software::HostSoftwareEntry;
use crate::vulnerability::CVE;
use crate::ListOptions;

/// HostStatus represents the online status of a host.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HostStatus {
    #[serde(rename = "online")]
    Online,
    #[serde(rename = "offline")]
    Offline,
    #[serde(rename = "mia")]
    MIA,
    #[serde(rename = "new")]
    New,
    #[serde(rename = "missing")]
    Missing,
}

/// MDMEnrollStatus defines the possible MDM enrollment statuses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MDMEnrollStatus {
    #[serde(rename = "manual")]
    Manual,
    #[serde(rename = "automatic")]
    Automatic,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "unenrolled")]
    Unenrolled,
    #[serde(rename = "enrolled")]
    Enrolled,
    #[serde(rename = "personal")]
    Personal,
}

/// OSSettingsStatus defines the possible statuses of the host's OS settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OSSettingsStatus {
    #[serde(rename = "verified")]
    Verified,
    #[serde(rename = "verifying")]
    Verifying,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "failed")]
    Failed,
}

/// DiskEncryptionStatus defines the possible statuses of disk encryption.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiskEncryptionStatus {
    #[serde(rename = "verified")]
    Verified,
    #[serde(rename = "verifying")]
    Verifying,
    #[serde(rename = "action_required")]
    ActionRequired,
    #[serde(rename = "enforcing")]
    Enforcing,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "removing_enforcement")]
    RemovingEnforcement,
}

/// MDMBootstrapPackageStatus defines the possible statuses of the host's MDM bootstrap package.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MDMBootstrapPackageStatus {
    #[serde(rename = "installed")]
    Installed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "pending")]
    Pending,
}

/// HostListOptions defines options for listing hosts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostListOptions {
    #[serde(flatten)]
    pub list_options: ListOptions,

    pub device_mapping: bool,
    pub additional_filters: Vec<String>,
    pub status_filter: Option<HostStatus>,
    pub team_filter: Option<u32>,
    pub policy_id_filter: Option<u32>,
    pub policy_response_filter: Option<bool>,
    pub software_id_filter: Option<u32>,
    pub software_version_id_filter: Option<u32>,
    pub software_title_id_filter: Option<u32>,
    pub os_id_filter: Option<u32>,
    pub os_name_filter: Option<String>,
    pub os_version_filter: Option<String>,
    pub os_version_id_filter: Option<u32>,
    pub disable_issues: bool,
    pub mdm_id_filter: Option<u32>,
    pub mdm_name_filter: Option<String>,
    pub mdm_enrollment_status_filter: Option<MDMEnrollStatus>,
    pub munki_issue_id_filter: Option<u32>,
    pub low_disk_space_filter: Option<i32>,
    pub vulnerability_filter: Option<String>,
    pub label_id_filter: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_status_filter: Option<crate::software::SoftwareInstallerStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos_settings_filter: Option<OSSettingsStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos_settings_disk_encryption_filter: Option<DiskEncryptionStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_settings_filter: Option<OSSettingsStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_settings_disk_encryption_filter: Option<DiskEncryptionStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mdm_bootstrap_package_filter: Option<MDMBootstrapPackageStatus>,
    pub populate_software: bool,
    pub populate_policies: bool,
    pub populate_users: bool,
    pub populate_labels: bool,
    pub include_device_status: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected_to_fleet_filter: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_uuid_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_status_filter: Option<OSSettingsStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_script_execution_status_filter: Option<BatchScriptExecutionStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_script_execution_id_filter: Option<String>,
    pub populate_software_vulnerability_details: bool,
}

/// HostUser represents a user account on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostUser {
    pub uid: u32,
    pub username: String,
    #[serde(rename = "type")]
    pub user_type: String,
    pub groupname: String,
    pub shell: String,
}

/// HostIssues contains counts of issues associated with a host.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostIssues {
    pub failing_policies_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub critical_vulnerabilities_count: Option<u64>,
    pub total_issues_count: u64,
}

/// MDMHostData contains MDM-related data for a host.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MDMHostData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrollment_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub dep_profile_error: bool,
    #[serde(default)]
    pub encryption_key_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_key_archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_settings: Option<HostMDMOSSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<Vec<crate::mdm::HostMDMProfile>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos_settings: Option<MDMHostMacOSSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos_setup: Option<HostMDMMacOSSetup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected_to_fleet: Option<bool>,
}

/// HostSoftware holds the software list and update timestamp for a host.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostSoftware {
    #[serde(default)]
    pub software: Vec<HostSoftwareEntry>,
    pub software_updated_at: DateTime<Utc>,
}

/// Host is the primary model representing a managed device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Software
    #[serde(flatten)]
    pub host_software: HostSoftware,

    pub id: u32,
    pub detail_updated_at: DateTime<Utc>,
    pub label_updated_at: DateTime<Utc>,
    pub policy_updated_at: DateTime<Utc>,
    pub last_enrolled_at: DateTime<Utc>,
    pub seen_time: DateTime<Utc>,
    pub refetch_requested: bool,
    pub hostname: String,
    pub uuid: String,
    pub platform: String,
    pub osquery_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orbit_version: Option<String>,
    #[serde(rename = "fleet_desktop_version")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desktop_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripts_enabled: Option<bool>,
    pub os_version: String,
    pub build: String,
    pub platform_like: String,
    pub code_name: String,
    /// Uptime in nanoseconds (matching Go's time.Duration).
    pub uptime: i64,
    pub memory: i64,

    // system_info fields
    pub cpu_type: String,
    pub cpu_subtype: String,
    pub cpu_brand: String,
    pub cpu_physical_cores: i32,
    pub cpu_logical_cores: i32,
    pub hardware_vendor: String,
    pub hardware_model: String,
    pub hardware_version: String,
    pub hardware_serial: String,
    pub computer_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_ip_id: Option<u32>,
    pub public_ip: String,
    pub primary_ip: String,
    pub primary_mac: String,
    pub distributed_interval: u32,
    pub config_tls_refresh: u32,
    pub logger_tls_period: u32,
    pub team_id: Option<u32>,

    // Loaded via JOIN
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional: Option<serde_json::Value>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<HostUser>,

    pub gigs_disk_space_available: f64,
    pub percent_disk_space_available: f64,
    pub gigs_total_disk_space: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gigs_all_disk_space: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_encryption_enabled: Option<bool>,

    #[serde(flatten)]
    pub issues: HostIssues,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_mapping: Option<serde_json::Value>,

    pub mdm: MDMHostData,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub refetch_critical_queries_until: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dep_assigned_to_fleet: Option<bool>,

    pub last_restarted_at: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub policies: Option<Vec<HostPolicy>>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pack_stats: Vec<crate::pack::PackStats>,

    // Additional host fields can be added here as needed.
}

/// HostDetail provides the full host metadata along with associated labels and packs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostDetail {
    #[serde(flatten)]
    pub host: Host,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<crate::label::LabelSummary>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packs: Vec<crate::pack::Pack>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policies: Vec<crate::policy::HostPolicy>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub software: Vec<crate::software::Software>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub batteries: Option<Vec<HostBattery>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintenance_window: Option<HostMaintenanceWindow>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub end_users: Vec<HostEndUser>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_mdm_enrolled_at: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_mdm_checked_in_at: Option<DateTime<Utc>>,

    #[serde(default)]
    pub conditional_access_bypassed: bool,
}

/// HostSummary contains the counts for the dashboard host summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostSummary {
    #[serde(rename = "totals_hosts_count")]
    pub totals_hosts_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub online_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offline_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mia_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_30_days_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub platforms: Vec<HostSummaryPlatform>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(default)]
    pub all_linux_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low_disk_space_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub builtin_labels: Vec<crate::label::LabelSummary>,
}

/// HostSummaryPlatform holds the count for a specific platform in the host summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostSummaryPlatform {
    pub platform: String,
    pub hosts_count: u32,
}

/// HostHealth contains a subset of Host data indicating how healthy a Host is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostHealth {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_encryption_enabled: Option<bool>,
    pub failing_policies_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failing_critical_policies_count: Option<i32>,
}

/// OSVersionStats holds aggregate stats for a given OS version.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OSVersionStats {
    pub id: u32,
    pub name: String,
    pub name_only: String,
    pub version: String,
    pub platform: String,
    pub hosts_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_cpes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vulnerabilities: Option<Vec<String>>,
}

/// HostOrbitInfo maps to the host_orbit_info table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostOrbitInfo {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desktop_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripts_enabled: Option<bool>,
}

// ---------------------------------------------------------------------------
// Host MDM types (from Go hosts.go)
// ---------------------------------------------------------------------------

/// HostMDM represents MDM enrollment info for a single host.
/// Matches Go's `fleet.HostMDM`.
#[derive(Debug, Clone)]
pub struct HostMDM {
    pub host_id: u32,
    pub enrolled: bool,
    pub server_url: String,
    pub installed_from_dep: bool,
    pub is_server: bool,
    pub is_personal_enrollment: bool,
    pub mdm_id: Option<u32>,
    pub name: String,
    pub dep_profile_assign_status: Option<String>,
}

impl HostMDM {
    /// Returns the enrollment status string, matching Go's `HostMDM.EnrollmentStatus()`.
    pub fn enrollment_status(&self) -> &'static str {
        match (self.enrolled, self.installed_from_dep, self.is_personal_enrollment) {
            (true, false, true) => "On (personal)",
            (true, false, false) => "On (manual)",
            (true, true, _) => "On (automatic)",
            (false, true, _) => "Pending",
            _ => "Off",
        }
    }
}

/// Custom JSON serialization for HostMDM matching Go's MarshalJSON.
/// Servers are serialized as null; otherwise outputs enrollment_status, server_url, name, id.
impl Serialize for HostMDM {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        if self.is_server {
            return serializer.serialize_none();
        }
        let field_count = 2 + if self.name.is_empty() { 0 } else { 1 } + if self.mdm_id.is_some() { 1 } else { 0 };
        let mut s = serializer.serialize_struct("HostMDM", field_count)?;
        s.serialize_field("enrollment_status", self.enrollment_status())?;
        s.serialize_field("server_url", &self.server_url)?;
        if !self.name.is_empty() {
            s.serialize_field("name", &self.name)?;
        }
        if let Some(id) = self.mdm_id {
            s.serialize_field("id", &id)?;
        }
        s.end()
    }
}

/// HostMunkiInfo holds munki version for a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMunkiInfo {
    pub version: String,
}

/// HostMunkiIssue represents a munki issue on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMunkiIssue {
    #[serde(rename = "id")]
    pub munki_issue_id: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub issue_type: String,
    pub created_at: DateTime<Utc>,
}

/// MacadminsData holds macadmins (munki + MDM) data for a single host.
#[derive(Debug, Clone, Serialize)]
pub struct MacadminsData {
    pub munki: Option<HostMunkiInfo>,
    #[serde(rename = "mobile_device_management")]
    pub mdm: Option<HostMDM>,
    pub munki_issues: Vec<HostMunkiIssue>,
}

// ---------------------------------------------------------------------------
// Aggregated MDM / Macadmins types
// ---------------------------------------------------------------------------

/// AggregatedMDMStatus holds enrollment status counts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregatedMDMStatus {
    pub enrolled_manual_hosts_count: i64,
    pub enrolled_automated_hosts_count: i64,
    pub enrolled_personal_hosts_count: i64,
    pub pending_hosts_count: i64,
    pub unenrolled_hosts_count: i64,
    pub hosts_count: i64,
}

/// MDMSolution identifies an MDM solution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMSolution {
    pub id: u32,
    pub name: String,
    pub server_url: String,
}

/// AggregatedMDMSolutions extends MDMSolution with a host count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMDMSolutions {
    #[serde(flatten)]
    pub solution: MDMSolution,
    pub hosts_count: i64,
}

/// AggregatedMDMData is the response for the host MDM summary endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct AggregatedMDMData {
    pub counts_updated_at: DateTime<Utc>,
    pub mobile_device_management_enrollment_status: AggregatedMDMStatus,
    pub mobile_device_management_solution: Vec<AggregatedMDMSolutions>,
}

/// AggregatedMunkiVersion extends HostMunkiInfo with a host count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMunkiVersion {
    pub version: String,
    pub hosts_count: i64,
}

/// MunkiIssue is the base munki issue type (for aggregation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MunkiIssue {
    pub id: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub issue_type: String,
}

/// AggregatedMunkiIssue extends MunkiIssue with a host count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMunkiIssue {
    #[serde(flatten)]
    pub issue: MunkiIssue,
    pub hosts_count: i64,
}

/// AggregatedMacadminsData holds all aggregated macadmins data.
#[derive(Debug, Clone, Serialize)]
pub struct AggregatedMacadminsData {
    pub counts_updated_at: DateTime<Utc>,
    pub munki_versions: Vec<AggregatedMunkiVersion>,
    pub munki_issues: Vec<AggregatedMunkiIssue>,
    pub mobile_device_management_enrollment_status: AggregatedMDMStatus,
    pub mobile_device_management_solution: Vec<AggregatedMDMSolutions>,
}

// ---------------------------------------------------------------------------
// Additional host types (from Go hosts.go)
// ---------------------------------------------------------------------------

/// HostBattery represents battery information for a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostBattery {
    pub id: u32,
    #[serde(skip)]
    pub host_id: u32,
    #[serde(skip)]
    pub serial_number: String,
    pub cycle_count: i32,
    pub health: String,
}

/// HostDeviceMapping represents a device-to-user mapping for a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostDeviceMapping {
    #[serde(skip)]
    pub id: u32,
    #[serde(skip)]
    pub host_id: u32,
    pub email: String,
    pub source: String,
}

/// HostEndUser represents an end user associated with a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostEndUser {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub idp_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub idp_username: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub idp_full_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub idp_groups: Vec<String>,
    #[serde(rename = "idp_department", default, skip_serializing_if = "String::is_empty")]
    pub idp_department: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idp_info_updated_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub other_emails: Vec<HostDeviceMapping>,
}

/// HostMaintenanceWindow represents a maintenance window for a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMaintenanceWindow {
    pub starts_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

/// HostMDMOSSettings contains the OS settings status for MDM-managed hosts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMDMOSSettings {
    pub disk_encryption: HostMDMDiskEncryption,
}

/// HostMDMDiskEncryption contains the disk encryption status and detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMDMDiskEncryption {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<DiskEncryptionStatus>,
    pub detail: String,
}

/// MDMHostMacOSSettings contains macOS-specific MDM settings for a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMHostMacOSSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_encryption: Option<DiskEncryptionStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_required: Option<String>,
}

/// HostMDMMacOSSetup contains macOS MDM setup status for a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMDMMacOSSetup {
    pub bootstrap_package_status: MDMBootstrapPackageStatus,
    pub detail: String,
    pub bootstrap_package_name: String,
}

/// HostDiskEncryptionKey contains the disk encryption key for a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostDiskEncryptionKey {
    #[serde(skip)]
    pub host_id: u32,
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "decrypted_value")]
    pub key: String,
}

/// HostLite is a minimal representation of a host with essential fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostLite {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    pub hostname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub osquery_host_id: Option<String>,
    pub node_key: String,
    pub uuid: String,
    pub hardware_serial: String,
    pub seen_time: DateTime<Utc>,
    pub distributed_interval: u32,
    pub config_tls_refresh: u32,
}

/// HostDetailOptions defines options for host detail queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostDetailOptions {
    pub include_cve_scores: bool,
    pub include_critical_vulnerabilities_count: bool,
    pub include_policies: bool,
    pub exclude_software: bool,
}

/// HostMDMCheckinInfo contains the information needed for MDM check-in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMDMCheckinInfo {
    pub host_id: u32,
    pub hardware_serial: String,
    pub installed_from_dep: bool,
    pub display_name: String,
    pub team_id: u32,
    pub dep_assigned_to_fleet: bool,
    pub osquery_enrolled: bool,
    pub platform: String,
}

/// OSVersion represents a specific OS version with host counts and vulnerabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OSVersion {
    pub id: u32,
    pub os_version_id: u32,
    pub hosts_count: i32,
    pub name: String,
    pub name_only: String,
    pub version: String,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_cpes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vulnerabilities: Option<Vec<CVE>>,
    pub vulnerabilities_count: i32,
}

/// OSVersions holds a list of OS versions with a timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OSVersions {
    pub counts_updated_at: DateTime<Utc>,
    pub os_versions: Vec<OSVersion>,
}

/// HostVulnerabilitySummary provides a summary of vulnerabilities for a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostVulnerabilitySummary {
    pub id: u32,
    pub hostname: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub software_installed_paths: Vec<String>,
}

/// NetworkInterface represents a network interface on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub id: u32,
    pub host_id: u32,
    pub interface: String,
    pub address: String,
    pub mask: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub broadcast: String,
}

/// HostHealthVulnerableSoftware holds a vulnerable software entry for host health.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostHealthVulnerableSoftware {
    pub id: u32,
    pub name: String,
    pub version: String,
}

/// HostHealthFailingPolicy holds a failing policy entry for host health.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostHealthFailingPolicy {
    pub id: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub critical: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
}

/// BatchScriptExecutionStatus defines the possible statuses of a batch script execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BatchScriptExecutionStatus {
    #[serde(rename = "ran")]
    Ran,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "errored")]
    Errored,
    #[serde(rename = "canceled")]
    Canceled,
    #[serde(rename = "incompatible")]
    Incompatible,
}

/// ActionRequiredState defines the possible action-required states for disk encryption.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionRequiredState {
    #[serde(rename = "log_out")]
    LogOut,
    #[serde(rename = "rotate_key")]
    RotateKey,
}

/// HostMacOSProfile represents a macOS profile installed on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMacOSProfile {
    pub display_name: String,
    pub identifier: String,
    pub install_date: DateTime<Utc>,
}

/// HostSoftwareInstalledPath represents where in the file system a software on a host was installed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostSoftwareInstalledPath {
    pub id: u32,
    pub host_id: u32,
    pub software_id: u32,
    pub installed_path: String,
    pub team_identifier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdhash_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_path: Option<String>,
}

/// VulnerableOS extends OSVersion with a resolved_in_version field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerableOS {
    #[serde(flatten)]
    pub os_version: OSVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_in_version: Option<String>,
}

/// Kernel represents a Linux kernel found on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kernel {
    pub id: u32,
    pub version: String,
    #[serde(default)]
    pub vulnerabilities: Vec<String>,
    pub hosts_count: u32,
}

/// HostArchivedDiskEncryptionKey contains an archived disk encryption key for a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostArchivedDiskEncryptionKey {
    #[serde(skip)]
    pub host_id: u32,
    #[serde(skip)]
    pub base64_encrypted: String,
    #[serde(skip)]
    pub base64_encrypted_salt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_slot: Option<u32>,
    pub created_at: DateTime<Utc>,
}

/// DeletedHostDetails contains details about a host that has been deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedHostDetails {
    pub id: u32,
    pub display_name: String,
    pub serial: String,
    pub host_expiry_window: i32,
}

/// AddHostsToTeamParams contains parameters for adding hosts to a team.
#[derive(Debug, Clone)]
pub struct AddHostsToTeamParams {
    pub team_id: Option<u32>,
    pub host_ids: Vec<u32>,
    pub batch_size: u32,
}

impl Default for AddHostsToTeamParams {
    fn default() -> Self {
        Self {
            team_id: None,
            host_ids: Vec::new(),
            batch_size: 10_000,
        }
    }
}

/// HostVitalType categorizes host vital data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostVitalType {
    /// Domestic vitals are stored in the host table.
    Domestic,
    /// Foreign vitals are stored in a separate table and joined.
    Foreign,
    /// Additional vitals are stored as JSON in the host_additional table.
    Additional,
}

/// HostVital describes a single host vital field.
#[derive(Debug, Clone)]
pub struct HostVital {
    pub name: String,
    pub vital_type: HostVitalType,
    pub data_type: String,
    pub foreign_vital_group: Option<String>,
    pub path: String,
}

/// HostForeignVitalGroup describes a foreign vitals group for host vitals labels.
#[derive(Debug, Clone)]
pub struct HostForeignVitalGroup {
    pub name: String,
    pub query: String,
}

/// HostVitalOperator defines comparison operators for host vitals criteria.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HostVitalOperator {
    #[serde(rename = "=")]
    Equal,
    #[serde(rename = "!=")]
    NotEqual,
    #[serde(rename = ">")]
    Greater,
    #[serde(rename = "<")]
    Less,
    #[serde(rename = "LIKE")]
    Like,
}

/// HostVitalCriteria defines criteria for host vitals labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostVitalCriteria {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vital: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<HostVitalOperator>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub and: Vec<HostVitalCriteria>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub or: Vec<HostVitalCriteria>,
}

// Well-known MDM solution name constants.
pub const WELL_KNOWN_MDM_IRU: &str = "Iru";
pub const WELL_KNOWN_MDM_JAMF: &str = "Jamf";
pub const WELL_KNOWN_MDM_JUMPCLOUD: &str = "JumpCloud";
pub const WELL_KNOWN_MDM_VMWARE: &str = "VMware Workspace ONE";
pub const WELL_KNOWN_MDM_INTUNE: &str = "Intune";
pub const WELL_KNOWN_MDM_SIMPLEMDM: &str = "SimpleMDM";
pub const WELL_KNOWN_MDM_FLEET: &str = "Fleet";
pub const WELL_KNOWN_MDM_MOSYLE: &str = "Mosyle";

// Device mapping source constants.
pub const DEVICE_MAPPING_GOOGLE_CHROME_PROFILES: &str = "google_chrome_profiles";
pub const DEVICE_MAPPING_MDM_IDP_ACCOUNTS: &str = "mdm_idp_accounts";
pub const DEVICE_MAPPING_IDP: &str = "idp";
pub const DEVICE_MAPPING_CUSTOM_INSTALLER: &str = "custom_installer";
pub const DEVICE_MAPPING_CUSTOM_OVERRIDE: &str = "custom_override";
pub const DEVICE_MAPPING_CUSTOM_PREFIX: &str = "custom_";
pub const DEVICE_MAPPING_CUSTOM_REPLACEMENT: &str = "custom";

/// GeoLocation contains geolocation data for a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_iso: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<GeoLocationGeometry>,
}

/// GeoLocationGeometry contains geolocation geometry data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocationGeometry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(default)]
    pub coordinates: Vec<f64>,
}

/// HostResponse is the response struct that contains the full host information
/// along with the host online status and the display text for the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostResponse {
    #[serde(flatten)]
    pub host: Host,
    pub status: HostStatus,
    pub display_text: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<crate::label::Label>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geolocation: Option<GeoLocation>,
}

/// Duration constants matching Go's host status logic.
pub const ONLINE_INTERVAL_BUFFER: u32 = 60;

/// NewDuration: if a host has been created within this period it's considered new (24 hours).
pub const NEW_DURATION: chrono::Duration = chrono::Duration::hours(24);

/// MIADuration: if a host hasn't communicated for this period it is considered MIA (30 days).
pub const MIA_DURATION: chrono::Duration = chrono::Duration::days(30);

/// Error message returned when a host identifier search yields no results.
pub const HOST_IDENTIFIER_NOT_FOUND: &str = "Host doesn't exist. Make sure you provide a valid hostname, UUID, or serial number. Learn more about host identifiers: https://fleetdm.com/learn-more-about/host-identifiers";

/// HostKind is the kind string used for hosts.
pub const HOST_KIND: &str = "host";

// ---------------------------------------------------------------------------
// HostStatus impl
// ---------------------------------------------------------------------------

impl HostStatus {
    /// Returns true if the status value is one of the known valid statuses.
    pub fn is_valid(&self) -> bool {
        matches!(
            self,
            HostStatus::Online
                | HostStatus::Offline
                | HostStatus::MIA
                | HostStatus::New
                | HostStatus::Missing
        )
    }
}

// ---------------------------------------------------------------------------
// OSSettingsStatus impl
// ---------------------------------------------------------------------------

impl OSSettingsStatus {
    /// Returns true if the status value is one of the known valid statuses.
    pub fn is_valid(&self) -> bool {
        matches!(
            self,
            OSSettingsStatus::Verified
                | OSSettingsStatus::Verifying
                | OSSettingsStatus::Pending
                | OSSettingsStatus::Failed
        )
    }
}

// ---------------------------------------------------------------------------
// DiskEncryptionStatus impl
// ---------------------------------------------------------------------------

impl DiskEncryptionStatus {
    /// Returns true if the status value is one of the known valid statuses.
    pub fn is_valid(&self) -> bool {
        matches!(
            self,
            DiskEncryptionStatus::Verified
                | DiskEncryptionStatus::Verifying
                | DiskEncryptionStatus::ActionRequired
                | DiskEncryptionStatus::Enforcing
                | DiskEncryptionStatus::Failed
                | DiskEncryptionStatus::RemovingEnforcement
        )
    }
}

// ---------------------------------------------------------------------------
// MDMBootstrapPackageStatus impl
// ---------------------------------------------------------------------------

impl MDMBootstrapPackageStatus {
    /// Returns true if the status value is one of the known valid statuses.
    pub fn is_valid(&self) -> bool {
        matches!(
            self,
            MDMBootstrapPackageStatus::Installed
                | MDMBootstrapPackageStatus::Failed
                | MDMBootstrapPackageStatus::Pending
        )
    }
}

// ---------------------------------------------------------------------------
// BatchScriptExecutionStatus impl
// ---------------------------------------------------------------------------

impl BatchScriptExecutionStatus {
    /// Returns true if the status value is one of the known valid statuses.
    pub fn is_valid(&self) -> bool {
        matches!(
            self,
            BatchScriptExecutionStatus::Ran
                | BatchScriptExecutionStatus::Pending
                | BatchScriptExecutionStatus::Errored
                | BatchScriptExecutionStatus::Canceled
                | BatchScriptExecutionStatus::Incompatible
        )
    }
}

// ---------------------------------------------------------------------------
// Host impl
// ---------------------------------------------------------------------------

impl Host {
    /// Returns the display name for the host, matching Go's `Host.DisplayName()`.
    pub fn display_name(&self) -> String {
        host_display_name(
            &self.computer_name,
            &self.hostname,
            &self.hardware_model,
            &self.hardware_serial,
        )
    }

    /// Returns the host's generic platform as supported by Fleet,
    /// matching Go's `Host.FleetPlatform()`.
    pub fn fleet_platform(&self) -> &str {
        platform_from_host(&self.platform)
    }

    /// Calculates the online status of the host at the given time,
    /// matching Go's `Host.Status()`.
    pub fn status(&self, now: DateTime<Utc>) -> HostStatus {
        let online_interval = std::cmp::min(self.distributed_interval, self.config_tls_refresh);
        let online_interval = online_interval + ONLINE_INTERVAL_BUFFER;

        let threshold = self.seen_time + chrono::Duration::seconds(online_interval as i64);
        if threshold < now {
            HostStatus::Offline
        } else {
            HostStatus::Online
        }
    }

    /// Returns true if the host was created within `NEW_DURATION` of `now`,
    /// matching Go's `Host.IsNew()`.
    pub fn is_new(&self, now: DateTime<Utc>) -> bool {
        let with_duration = self.created_at + NEW_DURATION;
        with_duration >= now
    }

    /// Returns true if the host platform supports LUKS disk encryption,
    /// matching Go's `Host.IsLUKSSupported()`.
    pub fn is_luks_supported(&self) -> bool {
        self.platform == "ubuntu"
            || self.os_version.contains("Fedora")
            || self.platform == "arch"
            || self.platform == "archarm"
            || self.platform == "manjaro"
            || self.platform == "manjaro-arm"
    }

    /// Returns true if the host platform supports RPM packages.
    pub fn platform_supports_rpm_packages(&self) -> bool {
        HOST_RPM_PACKAGE_OSS.contains(&self.platform.as_str())
    }

    /// Returns true if the host platform supports DEB packages.
    pub fn platform_supports_deb_packages(&self) -> bool {
        HOST_DEB_PACKAGE_OSS.contains(&self.platform.as_str())
    }

    /// Returns whether the device runs osquery.
    pub fn supports_osquery(&self) -> bool {
        platform_supports_osquery(&self.platform)
    }
}

// ---------------------------------------------------------------------------
// Platform helper functions and constants
// ---------------------------------------------------------------------------

/// HostLinuxOSs are the possible linux values for Host.Platform.
pub const HOST_LINUX_OSS: &[&str] = &[
    "linux",
    "ubuntu",
    "debian",
    "rhel",
    "centos",
    "sles",
    "kali",
    "gentoo",
    "amzn",
    "pop",
    "arch",
    "linuxmint",
    "void",
    "nixos",
    "endeavouros",
    "manjaro",
    "manjaro-arm",
    "opensuse-leap",
    "opensuse-tumbleweed",
    "tuxedo",
    "neon",
    "archarm",
];

/// Linux platforms that support DEB packages.
pub const HOST_DEB_PACKAGE_OSS: &[&str] = &[
    "linux", "ubuntu", "debian", "kali", "pop", "linuxmint", "tuxedo", "neon",
];

/// Linux platforms that support RPM packages.
pub const HOST_RPM_PACKAGE_OSS: &[&str] = &[
    "linux",
    "rhel",
    "centos",
    "sles",
    "amzn",
    "opensuse-leap",
    "opensuse-tumbleweed",
];

/// Linux platforms that support neither DEB nor RPM packages.
pub const HOST_NEITHER_DEB_NOR_RPM_PACKAGE_OSS: &[&str] = &[
    "arch",
    "archarm",
    "gentoo",
    "void",
    "nixos",
    "endeavouros",
    "manjaro",
    "manjaro-arm",
];

/// Returns true if the platform is a known Linux variant.
pub fn is_linux(host_platform: &str) -> bool {
    HOST_LINUX_OSS.contains(&host_platform)
}

/// Returns true if the platform is an Apple platform (macOS, iOS, iPadOS).
pub fn is_apple_platform(host_platform: &str) -> bool {
    host_platform == "darwin" || host_platform == "ios" || host_platform == "ipados"
}

/// Returns true if the platform is macOS.
pub fn is_macos_platform(host_platform: &str) -> bool {
    host_platform == "darwin"
}

/// Returns true if the platform is iOS or iPadOS.
pub fn is_apple_mobile_platform(host_platform: &str) -> bool {
    host_platform == "ios" || host_platform == "ipados"
}

/// Returns true if the platform is Android.
pub fn is_android_platform(host_platform: &str) -> bool {
    host_platform == "android"
}

/// Returns true if the platform is Unix-like (Linux or macOS).
pub fn is_unix_like(host_platform: &str) -> bool {
    is_linux(host_platform) || host_platform == "darwin"
}

/// Returns true if osquery is supported on this platform.
pub fn platform_supports_osquery(platform: &str) -> bool {
    platform != "ios" && platform != "ipados" && platform != "android"
}

/// Converts a host platform string into the generic platform known by Fleet.
/// Returns empty string if the platform is unknown.
pub fn platform_from_host(host_platform: &str) -> &str {
    if is_linux(host_platform) {
        return "linux";
    }
    match host_platform {
        "darwin" | "windows" | "CrOS" | "chrome" | "ios" | "ipados" | "android" => host_platform,
        _ => "",
    }
}

/// Returns the list of platforms corresponding to the (possibly generic) platform provided.
/// For example, "linux" expands to all linux platform identifiers.
pub fn expand_platform(platform: &str) -> Vec<String> {
    if platform == "linux" {
        HOST_LINUX_OSS.iter().map(|s| s.to_string()).collect()
    } else {
        vec![platform.to_string()]
    }
}

/// Returns the display name for a host, matching Go's `HostDisplayName()`.
pub fn host_display_name(
    computer_name: &str,
    hostname: &str,
    hardware_model: &str,
    hardware_serial: &str,
) -> String {
    if !computer_name.is_empty() {
        computer_name.to_string()
    } else if !hostname.is_empty() {
        hostname.to_string()
    } else if !hardware_model.is_empty() && !hardware_serial.is_empty() {
        format!("{} ({})", hardware_model, hardware_serial)
    } else {
        String::new()
    }
}

// ---------------------------------------------------------------------------
// Host certificate types (from Go host_certificates.go)
// ---------------------------------------------------------------------------

/// HostCertificateSource represents the source of a host certificate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HostCertificateSource {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
}

impl HostCertificateSource {
    /// Returns true if the source value is valid.
    pub fn is_valid(&self) -> bool {
        matches!(
            self,
            HostCertificateSource::System | HostCertificateSource::User
        )
    }
}

/// HostCertificateRecord is the database model for a host certificate,
/// matching Go's `HostCertificateRecord`.
#[derive(Debug, Clone)]
pub struct HostCertificateRecord {
    pub id: u32,
    pub host_id: u32,
    pub sha1_sum: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub not_valid_after: DateTime<Utc>,
    pub not_valid_before: DateTime<Utc>,
    pub certificate_authority: bool,
    pub common_name: String,
    pub key_algorithm: String,
    pub key_strength: i32,
    pub key_usage: String,
    pub serial: String,
    pub signing_algorithm: String,
    pub subject_country: String,
    pub subject_org: String,
    pub subject_org_unit: String,
    pub subject_common_name: String,
    pub issuer_country: String,
    pub issuer_org: String,
    pub issuer_org_unit: String,
    pub issuer_common_name: String,
    pub source: HostCertificateSource,
    pub username: String,
}

impl HostCertificateRecord {
    /// Converts this record into a `HostCertificatePayload` for API responses.
    pub fn to_payload(&self) -> HostCertificatePayload {
        HostCertificatePayload {
            id: self.id,
            not_valid_after: self.not_valid_after,
            not_valid_before: self.not_valid_before,
            certificate_authority: self.certificate_authority,
            common_name: self.common_name.clone(),
            key_algorithm: self.key_algorithm.clone(),
            key_strength: self.key_strength,
            key_usage: self.key_usage.clone(),
            serial: self.serial.clone(),
            signing_algorithm: self.signing_algorithm.clone(),
            source: self.source.clone(),
            username: self.username.clone(),
            subject: Some(HostCertificateNameDetails {
                common_name: self.subject_common_name.clone(),
                country: self.subject_country.clone(),
                organization: self.subject_org.clone(),
                organizational_unit: self.subject_org_unit.clone(),
            }),
            issuer: Some(HostCertificateNameDetails {
                common_name: self.issuer_common_name.clone(),
                country: self.issuer_country.clone(),
                organization: self.issuer_org.clone(),
                organizational_unit: self.issuer_org_unit.clone(),
            }),
        }
    }
}

/// HostCertificatePayload is the JSON model for API endpoints that return host certificates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostCertificatePayload {
    pub id: u32,
    pub not_valid_after: DateTime<Utc>,
    pub not_valid_before: DateTime<Utc>,
    pub certificate_authority: bool,
    pub common_name: String,
    pub key_algorithm: String,
    pub key_strength: i32,
    pub key_usage: String,
    pub serial: String,
    pub signing_algorithm: String,
    pub source: HostCertificateSource,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<HostCertificateNameDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<HostCertificateNameDetails>,
}

/// HostCertificateNameDetails contains the subject or issuer details of a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostCertificateNameDetails {
    pub common_name: String,
    pub country: String,
    pub organization: String,
    pub organizational_unit: String,
}

// ---------------------------------------------------------------------------
// HostListOptions impl
// ---------------------------------------------------------------------------

impl HostListOptions {
    /// Returns true if no filters are set, matching Go's `HostListOptions.Empty()`.
    pub fn empty(&self) -> bool {
        self.list_options == ListOptions::default()
            && !self.device_mapping
            && self.additional_filters.is_empty()
            && self.status_filter.is_none()
            && self.team_filter.is_none()
            && self.policy_id_filter.is_none()
            && self.policy_response_filter.is_none()
            && self.software_id_filter.is_none()
            && self.software_version_id_filter.is_none()
            && self.software_title_id_filter.is_none()
            && self.software_status_filter.is_none()
            && self.os_id_filter.is_none()
            && self.os_name_filter.is_none()
            && self.os_version_filter.is_none()
            && !self.disable_issues
            && self.macos_settings_filter.is_none()
            && self.macos_settings_disk_encryption_filter.is_none()
            && self.mdm_bootstrap_package_filter.is_none()
            && self.mdm_id_filter.is_none()
            && self.mdm_name_filter.is_none()
            && self.mdm_enrollment_status_filter.is_none()
            && self.munki_issue_id_filter.is_none()
            && self.low_disk_space_filter.is_none()
            && self.os_settings_filter.is_none()
            && self.os_settings_disk_encryption_filter.is_none()
            && self.profile_uuid_filter.is_none()
            && self.profile_status_filter.is_none()
    }
}

/// MDMNameFromServerURL returns the MDM solution name corresponding to the
/// given server URL. If no match is found, it returns an empty string.
pub fn mdm_name_from_server_url(server_url: &str) -> &'static str {
    let lower = server_url.to_lowercase();
    let checks: &[(&str, &str)] = &[
        ("kandji", WELL_KNOWN_MDM_IRU),
        ("iru.com", WELL_KNOWN_MDM_IRU),
        ("jamf", WELL_KNOWN_MDM_JAMF),
        ("jumpcloud", WELL_KNOWN_MDM_JUMPCLOUD),
        ("airwatch", WELL_KNOWN_MDM_VMWARE),
        ("awmdm", WELL_KNOWN_MDM_VMWARE),
        ("microsoft", WELL_KNOWN_MDM_INTUNE),
        ("simplemdm", WELL_KNOWN_MDM_SIMPLEMDM),
        ("fleetdm", WELL_KNOWN_MDM_FLEET),
        ("mosyle", WELL_KNOWN_MDM_MOSYLE),
    ];
    for (check, name) in checks {
        if lower.contains(check) {
            return name;
        }
    }
    ""
}
