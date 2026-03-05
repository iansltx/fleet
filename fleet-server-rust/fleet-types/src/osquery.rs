//! Osquery types matching Go's `server/fleet/osquery.go`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// OsqueryDistributedQueryResults represents the format of the results of an
/// osquery distributed query.
pub type OsqueryDistributedQueryResults = HashMap<String, Vec<HashMap<String, String>>>;

/// OsqueryStatus represents osquery status codes (0 = success, nonzero = failure).
pub type OsqueryStatus = i32;

/// StatusOK is the success code returned by osquery.
pub const STATUS_OK: OsqueryStatus = 0;

/// QueryContent is the format of a query stanza in an osquery configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryContent {
    pub query: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub interval: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub denylist: Option<bool>,
}

/// Queries is a helper which represents the format of a set of queries in a pack.
pub type Queries = HashMap<String, QueryContent>;

/// PackContent is the format of an osquery query pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackContent {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub platform: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub version: String,
    #[serde(default)]
    pub shard: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub discovery: Vec<String>,
    pub queries: Queries,
}

/// Packs is a helper which represents the format of a list of osquery query packs.
pub type Packs = HashMap<String, PackContent>;

/// DatastoreEnrollOsqueryConfig holds the configuration for datastore Host enrollment.
#[derive(Debug, Clone)]
pub struct DatastoreEnrollOsqueryConfig {
    pub is_mdm_enabled: bool,
    pub osquery_host_id: String,
    pub hardware_uuid: String,
    pub hardware_serial: String,
    pub node_key: String,
    pub team_id: Option<u32>,
    pub cooldown: Duration,
    pub ignore_team_update: bool,
}
