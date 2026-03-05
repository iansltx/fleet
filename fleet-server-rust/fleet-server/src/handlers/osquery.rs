//! Osquery protocol endpoints.
//!
//! Implements the osquery TLS API: enroll, config, distributed read/write,
//! log submission, and YARA rule retrieval.

use axum::extract::{Json, Path, State};
use serde::Deserialize;
use std::collections::HashMap;

use crate::response::{encode_service_error, fleet_ok, FleetResponse};
use crate::AppState;

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
pub async fn enroll_agent(
    State(state): State<AppState>,
    Json(body): Json<EnrollAgentBody>,
) -> FleetResponse {
    let host_details: HashMap<String, HashMap<String, String>> = body
        .host_details
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    match state
        .service
        .enroll_agent(
            &body.enroll_secret,
            body.host_identifier.as_deref().unwrap_or(""),
            &host_details,
        )
        .await
    {
        Ok(node_key) => fleet_ok("node_key", serde_json::json!(node_key)),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/osquery/config
/// POST /api/v1/osquery/config
///
/// Returns the osquery configuration for the enrolled host, including
/// packs, scheduled queries, options, and decorators.
pub async fn get_client_config(
    State(state): State<AppState>,
    Json(body): Json<GetClientConfigBody>,
) -> FleetResponse {
    let (host, _debug) = match state.service.authenticate_host(&body.node_key).await {
        Ok(result) => result,
        Err(e) => return encode_service_error(&e),
    };
    match state.service.get_client_config(&host).await {
        Ok(config) => (
            axum::http::StatusCode::OK,
            Json(serde_json::to_value(&config).unwrap_or_default()),
        ),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/osquery/distributed/read
/// POST /api/v1/osquery/distributed/read
///
/// Returns pending queries for the host to execute. This includes
/// live queries and label queries.
pub async fn get_distributed_queries(
    State(state): State<AppState>,
    Json(body): Json<GetDistributedQueriesBody>,
) -> FleetResponse {
    let (host, _debug) = match state.service.authenticate_host(&body.node_key).await {
        Ok(result) => result,
        Err(e) => return encode_service_error(&e),
    };
    match state.service.get_distributed_queries(&host).await {
        Ok(result) => fleet_ok(
            "",
            serde_json::json!({
                "queries": result.queries,
                "discovery": result.discovery,
                "accelerate": result.accelerate,
            }),
        ),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/osquery/distributed/write
/// POST /api/v1/osquery/distributed/write
///
/// Receives the results of distributed queries from the host.
pub async fn submit_distributed_query_results(
    State(state): State<AppState>,
    Json(body): Json<SubmitDistributedQueryResultsBody>,
) -> FleetResponse {
    let (host, _debug) = match state.service.authenticate_host(&body.node_key).await {
        Ok(result) => result,
        Err(e) => return encode_service_error(&e),
    };
    let results: HashMap<String, Vec<HashMap<String, String>>> = body
        .queries
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    let statuses: HashMap<String, i32> = body
        .statuses
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    let messages: HashMap<String, String> = body
        .messages
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    match state
        .service
        .submit_distributed_query_results(&host, &results, &statuses, &messages)
        .await
    {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/osquery/log
/// POST /api/v1/osquery/log
///
/// Receives status or result logs from the osquery agent.
pub async fn submit_logs(
    State(state): State<AppState>,
    Json(body): Json<SubmitLogsBody>,
) -> FleetResponse {
    let (host, _debug) = match state.service.authenticate_host(&body.node_key).await {
        Ok(result) => result,
        Err(e) => return encode_service_error(&e),
    };
    let logs: Vec<serde_json::Value> = match &body.data {
        serde_json::Value::Array(arr) => arr.clone(),
        other => vec![other.clone()],
    };
    let result = match body.log_type.as_str() {
        "status" => state.service.submit_status_logs(&host, &logs).await,
        "result" => state.service.submit_result_logs(&host, &logs).await,
        _ => Err(fleet_service::ServiceError::invalid_argument(
            "log_type",
            "unknown log type",
        )),
    };
    match result {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/osquery/yara/{name}
/// POST /api/v1/osquery/yara/{name}
///
/// Returns YARA rules for the specified rule name.
pub async fn get_yara(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<GetYaraBody>,
) -> FleetResponse {
    let (host, debug) = match state.service.authenticate_host(&body.node_key).await {
        Ok(result) => result,
        Err(e) => return encode_service_error(&e),
    };
    let _ = (&host, debug, &name);
    // YARA rule lookup is not yet implemented in the service layer.
    // Authenticate the host but return empty rules for now.
    fleet_ok("rules", serde_json::json!(""))
}
