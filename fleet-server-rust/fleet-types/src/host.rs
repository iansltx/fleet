//! Host types matching Go's `server/fleet/hosts.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::policy::HostPolicy;
use crate::software::HostSoftwareEntry;
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
    // Additional filter fields omitted for brevity; add as needed.
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
    // Additional MDM fields omitted for brevity.
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
