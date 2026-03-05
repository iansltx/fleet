//! Target types matching Go's `server/fleet/targets.go`.

use serde::{Deserialize, Serialize};

/// TargetType identifies the type of a target.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TargetType {
    Label,
    Host,
    Team,
}

impl std::fmt::Display for TargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetType::Label => write!(f, "label"),
            TargetType::Host => write!(f, "host"),
            TargetType::Team => write!(f, "team"),
        }
    }
}

/// Target represents a generic target (host, label, or team).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub id: u32,
    #[serde(rename = "type")]
    pub target_type: TargetType,
    pub name: String,
}

/// HostTargets is the set of targets for a campaign (live query).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostTargets {
    #[serde(default)]
    pub hosts: Vec<u32>,
    #[serde(default)]
    pub labels: Vec<u32>,
    #[serde(default)]
    pub teams: Vec<u32>,
}

/// TargetMetrics contains information about the online status of a set of hosts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TargetMetrics {
    /// Total hosts in any status.
    pub total: u32,
    /// Hosts that have checked in within their expected interval.
    pub online: u32,
    /// Hosts that have not checked in within their expected interval.
    pub offline: u32,
    /// Hosts that have not checked in within the last 30 days.
    pub mia: u32,
    /// Hosts that have enrolled in the last 24 hours.
    pub new: u32,
}

/// TargetSearchResults holds the results of searching for targets.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TargetSearchResults {
    #[serde(default)]
    pub hosts: Vec<crate::host::Host>,
    #[serde(default)]
    pub labels: Vec<crate::label::Label>,
    #[serde(default)]
    pub teams: Vec<crate::team::Team>,
}
