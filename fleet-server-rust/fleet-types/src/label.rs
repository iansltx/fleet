//! Label types matching Go's `server/fleet/labels.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// LabelType categorizes the kind of label.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LabelType {
    /// User created labels that can be modified.
    #[serde(rename = "regular")]
    Regular,
    /// Labels built into Fleet that cannot be modified by users.
    #[serde(rename = "builtin")]
    BuiltIn,
}

impl Default for LabelType {
    fn default() -> Self {
        LabelType::Regular
    }
}

/// LabelMembershipType sets how the membership of the label is determined.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LabelMembershipType {
    /// Populated dynamically by the execution of a label query.
    #[serde(rename = "dynamic")]
    Dynamic,
    /// Populated manually.
    #[serde(rename = "manual")]
    Manual,
    /// Populated dynamically based on host vitals data.
    #[serde(rename = "host_vitals")]
    HostVitals,
}

impl Default for LabelMembershipType {
    fn default() -> Self {
        LabelMembershipType::Dynamic
    }
}

/// LabelPayload is used to create a new label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelPayload {
    pub name: String,
    pub query: String,
    pub platform: String,
    pub description: String,
    #[serde(default)]
    pub hosts: Vec<String>,
    #[serde(default)]
    pub host_ids: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criteria: Option<serde_json::Value>,
}

/// ModifyLabelPayload is used to change editable fields for a Label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyLabelPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    /// If not nil, the new list of host identifiers for manual labels.
    pub hosts: Option<Vec<String>>,
    pub host_ids: Option<Vec<u32>>,
}

/// Label represents a group of hosts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: u32,
    pub author_id: Option<u32>,
    pub name: String,
    pub description: String,
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criteria: Option<serde_json::Value>,
    pub platform: String,
    pub label_type: LabelType,
    pub label_membership_type: LabelMembershipType,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub host_count: i32,
    pub team_id: Option<u32>,
}

fn is_zero(v: &i32) -> bool {
    *v == 0
}

/// LabelSummary is a minimal label representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSummary {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub team_id: Option<u32>,
    pub label_type: LabelType,
}

/// LabelIdent holds an id/name pair for label references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelIdent {
    #[serde(rename = "id")]
    pub label_id: u32,
    #[serde(rename = "name")]
    pub label_name: String,
}

/// LabelQueryExecution records the result of a label query execution on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelQueryExecution {
    pub id: u32,
    pub updated_at: DateTime<Utc>,
    pub matches: bool,
    pub label_id: u32,
    pub host_id: u32,
}

/// LabelSpec is used to apply label specs (from YAML).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSpec {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub query: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub platform: String,
    #[serde(default)]
    pub label_type: LabelType,
    #[serde(default)]
    pub label_membership_type: LabelMembershipType,
    #[serde(default)]
    pub hosts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criteria: Option<serde_json::Value>,
    pub team_id: Option<u32>,
}

// Well-known built-in label names.
pub const BUILTIN_LABEL_ALL_HOSTS: &str = "All Hosts";
pub const BUILTIN_LABEL_MACOS: &str = "macOS";
pub const BUILTIN_LABEL_UBUNTU_LINUX: &str = "Ubuntu Linux";
pub const BUILTIN_LABEL_CENTOS_LINUX: &str = "CentOS Linux";
pub const BUILTIN_LABEL_WINDOWS: &str = "MS Windows";
