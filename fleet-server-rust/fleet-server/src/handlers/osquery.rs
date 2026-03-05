//! Osquery protocol endpoints.
//!
//! Implements the osquery TLS API: enroll, config, distributed read/write,
//! log submission, and YARA rule retrieval.

use axum::{
    extract::{Json, Path},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::response::{fleet_error, fleet_ok, FleetResponse};

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

/// Osquery agent enrollment request.
#[derive(Debug, Deserialize)]
pub struct EnrollAgentBody {
    pub enroll_secret: String,
    pub host_identifier: Option<String>,
    pub host_details: Option<serde_json::Value>,
}

/// Osquery config request (sent by the agent periodically).
#[derive(Debug, Deserialize)]
pub struct GetClientConfigBody {
    pub node_key: String,
}

/// Distributed read request: osquery asks for queries to run.
#[derive(Debug, Deserialize)]
pub struct GetDistributedQueriesBody {
    pub node_key: String,
}

/// Distributed write request: osquery submits query results.
#[derive(Debug, Deserialize)]
pub struct SubmitDistributedQueryResultsBody {
    pub node_key: String,
    pub queries: Option<serde_json::Value>,
    pub statuses: Option<serde_json::Value>,
    pub messages: Option<serde_json::Value>,
    pub stats: Option<serde_json::Value>,
}

/// Log submission request: osquery sends status or result logs.
#[derive(Debug, Deserialize)]
pub struct SubmitLogsBody {
    pub node_key: String,
    pub log_type: String,
    pub data: serde_json::Value,
}

/// YARA rule request.
#[derive(Debug, Deserialize)]
pub struct GetYaraBody {
    pub node_key: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/osquery/enroll
/// POST /api/v1/osquery/enroll
///
/// Enrolls a new osquery agent. The agent provides an enroll secret and
/// receives a node_key for subsequent requests.
pub async fn enroll_agent(Json(_body): Json<EnrollAgentBody>) -> FleetResponse {
    // TODO: validate enroll secret, create or lookup host, return node_key
    fleet_ok("node_key", serde_json::json!(""))
}

/// POST /api/osquery/config
/// POST /api/v1/osquery/config
///
/// Returns the osquery configuration for the enrolled host, including
/// packs, scheduled queries, options, and decorators.
pub async fn get_client_config(Json(_body): Json<GetClientConfigBody>) -> FleetResponse {
    // TODO: authenticate node_key, build osquery config
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/osquery/distributed/read
/// POST /api/v1/osquery/distributed/read
///
/// Returns pending queries for the host to execute. This includes
/// live queries and label queries.
pub async fn get_distributed_queries(
    Json(_body): Json<GetDistributedQueriesBody>,
) -> FleetResponse {
    // TODO: authenticate node_key, return pending queries
    fleet_ok("queries", serde_json::json!({}))
}

/// POST /api/osquery/distributed/write
/// POST /api/v1/osquery/distributed/write
///
/// Receives the results of distributed queries from the host.
pub async fn submit_distributed_query_results(
    Json(_body): Json<SubmitDistributedQueryResultsBody>,
) -> FleetResponse {
    // TODO: authenticate node_key, process query results
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/osquery/log
/// POST /api/v1/osquery/log
///
/// Receives status or result logs from the osquery agent.
pub async fn submit_logs(Json(_body): Json<SubmitLogsBody>) -> FleetResponse {
    // TODO: authenticate node_key, forward logs to configured plugin
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/osquery/yara/{name}
/// POST /api/v1/osquery/yara/{name}
///
/// Returns YARA rules for the specified rule name.
pub async fn get_yara(
    Path(_name): Path<String>,
    Json(_body): Json<GetYaraBody>,
) -> FleetResponse {
    // TODO: authenticate node_key, return YARA rules
    fleet_ok("rules", serde_json::json!(""))
}
