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

// Software field length constants
pub const SOFTWARE_NAME_MAX_LENGTH: usize = 255;
pub const SOFTWARE_VERSION_MAX_LENGTH: usize = 255;
pub const SOFTWARE_SOURCE_MAX_LENGTH: usize = 64;
pub const SOFTWARE_BUNDLE_IDENTIFIER_MAX_LENGTH: usize = 255;
pub const SOFTWARE_VENDOR_MAX_LENGTH: usize = 114;
