//! Pack types matching Go's `server/fleet/packs.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::target::Target;
use crate::ListOptions;

/// Maximum scheduled query interval in seconds allowed by osquery.
pub const MAX_SCHEDULED_QUERY_INTERVAL: u32 = 604800;

/// PackListOptions defines options for listing packs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackListOptions {
    #[serde(flatten)]
    pub list_options: ListOptions,
    pub include_system_packs: bool,
}

/// Pack is the structure representing an osquery query pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pack {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: u32,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub platform: String,
    pub disabled: bool,
    /// Type indicates the type of the pack:
    /// - "global" is the type of the global pack.
    /// - "team-$ID" is the type for team packs.
    /// - None is the type for a user created pack.
    #[serde(rename = "type")]
    pub pack_type: Option<String>,
    #[serde(default)]
    pub labels: Vec<Target>,
    #[serde(default)]
    pub label_ids: Vec<u32>,
    #[serde(default)]
    pub hosts: Vec<Target>,
    #[serde(default)]
    pub host_ids: Vec<u32>,
    #[serde(default)]
    pub teams: Vec<Target>,
    #[serde(default)]
    pub team_ids: Vec<u32>,
}

impl Pack {
    /// Returns true if the pack is a team-specific pack.
    pub fn is_team_pack(&self) -> bool {
        self.pack_type
            .as_ref()
            .map_or(false, |t| t.starts_with("team-"))
    }

    /// Returns true if the pack is the global pack.
    pub fn is_global_pack(&self) -> bool {
        self.pack_type.as_ref().map_or(false, |t| t == "global")
    }

    /// Returns true if the pack type is editable (nil or empty string).
    pub fn editable_pack_type(&self) -> bool {
        self.pack_type.as_ref().map_or(true, |t| t.is_empty())
    }
}

/// ScheduledQueryStats contains per-host stats for a scheduled query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledQueryStats {
    pub scheduled_query_name: String,
    pub scheduled_query_id: u32,
    pub query_name: String,
    pub description: String,
    pub pack_name: String,
    pub pack_id: u32,
    pub average_memory: u64,
    pub denylisted: bool,
    pub executions: u64,
    pub interval: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_executed: Option<DateTime<Utc>>,
    pub output_size: u64,
    pub system_time: u64,
    pub user_time: u64,
    pub wall_time: u64,
}

/// PackStats contains stats for a pack, including its scheduled queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackStats {
    pub pack_id: u32,
    pub pack_name: String,
    /// Type indicates the type of the pack:
    /// - "global" for the global pack
    /// - "team-$ID" for team packs
    /// - "pack" for user created packs
    #[serde(rename = "type")]
    pub pack_type: String,
    pub query_stats: Vec<ScheduledQueryStats>,
}
