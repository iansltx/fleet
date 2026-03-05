//! Campaign types matching Go's `server/fleet/campaigns.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::target::TargetMetrics;

/// DistributedQueryStatus is the lifecycle status of a distributed query campaign.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum DistributedQueryStatus {
    Waiting = 0,
    Running = 1,
    Complete = 2,
}

/// DistributedQueryCampaign is the basic metadata associated with a distributed query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedQueryCampaign {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub metrics: TargetMetrics,
    pub id: u32,
    pub query_id: u32,
    pub status: DistributedQueryStatus,
    pub user_id: u32,
}

/// DistributedQueryCampaignTarget stores a target for a distributed query campaign.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedQueryCampaignTarget {
    pub id: u32,
    #[serde(rename = "type")]
    pub target_type: crate::target::TargetType,
    pub distributed_query_campaign_id: u32,
    pub target_id: u32,
}

/// ResultHostData holds the host's data from where a query result comes from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultHostData {
    pub id: u32,
    pub hostname: String,
    pub display_name: String,
}

/// Stats contains the performance statistics about the execution of an osquery query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub wall_time_ms: u64,
    pub user_time: u64,
    pub system_time: u64,
    pub memory: u64,
}

/// DistributedQueryResult is the result from executing a distributed query on a single host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedQueryResult {
    pub distributed_query_execution_id: u32,
    pub host: ResultHostData,
    pub rows: Vec<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<Stats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// QueryResult holds the result of a query on a single host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub host_id: u32,
    pub rows: Vec<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// QueryCampaignResult holds the aggregate result for a query campaign.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCampaignResult {
    pub query_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub results: Vec<QueryResult>,
}
