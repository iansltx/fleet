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

/// ScheduledQuery represents a query scheduled in a pack.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScheduledQuery {
    pub id: u32,
    pub pack_id: u32,
    pub query_id: u32,
    pub query_name: String,
    pub query: String,
    pub name: String,
    pub description: String,
    pub interval: u32,
    pub snapshot: Option<bool>,
    pub removed: Option<bool>,
    pub platform: String,
    pub version: String,
    pub shard: Option<u32>,
    pub denylist: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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

/// PackPayload is the struct used to create/update packs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub platform: Option<String>,
    pub disabled: Option<bool>,
    pub host_ids: Option<Vec<u32>>,
    pub label_ids: Option<Vec<u32>>,
    pub team_ids: Option<Vec<u32>>,
}

/// PackSpec is the spec format for packs (used in YAML/gitops).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackSpec {
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub id: u32,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub platform: String,
    pub disabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targets: Option<PackSpecTargets>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub queries: Vec<PackSpecQuery>,
}

fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}

/// PackSpecTargets specifies the targets for a pack spec.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackSpecTargets {
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub teams: Vec<String>,
}

/// PackSpecQuery defines a scheduled query within a pack spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackSpecQuery {
    #[serde(rename = "query")]
    pub query_name: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub interval: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub denylist: Option<bool>,
}

/// Pack kind constant.
pub const PACK_KIND: &str = "pack";

// ───────────────────────────────────────────────────────────────────────────
// Validation helpers
// ───────────────────────────────────────────────────────────────────────────

/// Errors that can occur when verifying pack-related payloads.
#[derive(Debug, Clone, thiserror::Error)]
pub enum PackValidationError {
    #[error("pack name cannot be empty")]
    EmptyName,
    #[error(
        "pack scheduled query interval must be an integer greater than 1 and less than 604800"
    )]
    InvalidInterval,
}

fn is_empty_string(s: &str) -> bool {
    s.trim().is_empty()
}

impl Pack {
    /// Returns the team ID if this is a team pack (type = "team-$ID"), or `None` otherwise.
    pub fn team_pack_id(&self) -> Result<Option<u32>, std::num::ParseIntError> {
        if !self.is_team_pack() {
            return Ok(None);
        }
        let t = self
            .pack_type
            .as_ref()
            .unwrap()
            .strip_prefix("team-")
            .unwrap();
        t.parse::<u32>().map(Some)
    }

    /// Verify verifies the pack's fields are valid.
    pub fn verify(&self) -> Result<(), PackValidationError> {
        if is_empty_string(&self.name) {
            return Err(PackValidationError::EmptyName);
        }
        Ok(())
    }
}

impl PackPayload {
    /// Verify verifies the pack payload's fields are valid.
    pub fn verify(&self) -> Result<(), PackValidationError> {
        if let Some(ref name) = self.name {
            if is_empty_string(name) {
                return Err(PackValidationError::EmptyName);
            }
        }
        Ok(())
    }
}

impl PackSpec {
    /// Verify verifies the pack spec's fields are valid.
    pub fn verify(&self) -> Result<(), PackValidationError> {
        if is_empty_string(&self.name) {
            return Err(PackValidationError::EmptyName);
        }
        for sq in &self.queries {
            if sq.interval < 1 || sq.interval > MAX_SCHEDULED_QUERY_INTERVAL {
                return Err(PackValidationError::InvalidInterval);
            }
        }
        Ok(())
    }
}
