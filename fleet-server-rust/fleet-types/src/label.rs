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

/// LabelWithTeamName extends Label with a team name field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelWithTeamName {
    #[serde(flatten)]
    pub label: Label,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// LabelScope identifies how labels scope entities like MDM profiles and software installers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LabelScope {
    #[serde(rename = "exclude_any")]
    ExcludeAny,
    #[serde(rename = "include_any")]
    IncludeAny,
    #[serde(rename = "include_all")]
    IncludeAll,
}

/// LabelIdentsWithScope holds label identifiers with a scope.
#[derive(Debug, Clone)]
pub struct LabelIdentsWithScope {
    pub label_scope: LabelScope,
    pub by_name: std::collections::HashMap<String, LabelIdent>,
}

/// Valid label platform variants.
pub const VALID_LABEL_PLATFORM_VARIANTS: &[&str] = &["", "darwin", "windows", "ubuntu", "centos"];

// Well-known built-in label names.
pub const BUILTIN_LABEL_ALL_HOSTS: &str = "All Hosts";
pub const BUILTIN_LABEL_MACOS: &str = "macOS";
pub const BUILTIN_LABEL_UBUNTU_LINUX: &str = "Ubuntu Linux";
pub const BUILTIN_LABEL_CENTOS_LINUX: &str = "CentOS Linux";
pub const BUILTIN_LABEL_WINDOWS: &str = "MS Windows";
pub const BUILTIN_LABEL_RED_HAT_LINUX: &str = "Red Hat Linux";
pub const BUILTIN_LABEL_ALL_LINUX: &str = "All Linux";
pub const BUILTIN_LABEL_CHROME: &str = "chrome";
pub const BUILTIN_LABEL_MACOS_14_PLUS: &str = "macOS 14+ (Sonoma+)";
pub const BUILTIN_LABEL_IOS: &str = "iOS";
pub const BUILTIN_LABEL_IPADOS: &str = "iPadOS";
pub const BUILTIN_LABEL_FEDORA_LINUX: &str = "Fedora Linux";
pub const BUILTIN_LABEL_ANDROID: &str = "Android";

/// Label kind constant.
pub const LABEL_KIND: &str = "label";

/// MissingLabelError is returned when a label referenced by name cannot be found.
#[derive(Debug, Clone)]
pub struct MissingLabelError {
    pub message: String,
    pub missing_label_name: String,
}

impl std::fmt::Display for MissingLabelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MissingLabelError {}
