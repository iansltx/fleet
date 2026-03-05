//! Query types matching Go's `server/fleet/queries.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::label::LabelIdent;
use crate::pack::Pack;

/// AggregatedStats are the stats aggregated from all the individual stats
/// reported by hosts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregatedStats {
    pub system_time_p50: Option<f64>,
    pub system_time_p95: Option<f64>,
    pub user_time_p50: Option<f64>,
    pub user_time_p95: Option<f64>,
    pub total_executions: Option<f64>,
}

/// QueryPayload is the payload used to create and modify queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub query: Option<String>,
    pub observer_can_run: Option<bool>,
    pub team_id: Option<u32>,
    pub interval: Option<u32>,
    pub platform: Option<String>,
    pub min_osquery_version: Option<String>,
    pub automations_enabled: Option<bool>,
    pub logging: Option<String>,
    pub discard_data: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_include_any: Vec<String>,
}

/// QueryResultRow represents a single result row from a query report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResultRow {
    pub host_id: u32,
    pub hostname: String,
    pub last_fetched: DateTime<Utc>,
    #[serde(default)]
    pub columns: serde_json::Value,
}

/// Query represents an osquery query to run on devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: u32,
    pub team_id: Option<u32>,
    pub interval: u32,
    pub platform: String,
    pub min_osquery_version: String,
    pub automations_enabled: bool,
    pub logging: String,
    pub name: String,
    pub description: String,
    pub query: String,
    pub saved: bool,
    pub observer_can_run: bool,
    pub author_id: Option<u32>,
    pub author_name: String,
    pub author_email: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packs: Vec<Pack>,
    #[serde(rename = "stats")]
    #[serde(default)]
    pub aggregated_stats: AggregatedStats,
    pub discard_data: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_include_any: Vec<LabelIdent>,
}
