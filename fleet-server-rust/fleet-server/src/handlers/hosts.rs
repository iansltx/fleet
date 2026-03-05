//! Host management endpoints.
//!
//! Handles host listing, details, transfers, refetch, device mappings,
//! OS versions, macadmins data, scripts, activities, and host actions.

use axum::{
    extract::{Json, Path, Query},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::response::{fleet_error, fleet_ok, FleetResponse};

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListHostsParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
    pub status: Option<String>,
    pub query: Option<String>,
    pub team_id: Option<u64>,
    pub label_id: Option<u64>,
    pub policy_id: Option<u64>,
    pub policy_response: Option<String>,
    pub software_id: Option<u64>,
    pub software_version_id: Option<u64>,
    pub software_title_id: Option<u64>,
    pub os_id: Option<u64>,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub vulnerability: Option<String>,
    pub mdm_id: Option<u64>,
    pub mdm_name: Option<String>,
    pub mdm_enrollment_status: Option<String>,
    pub macos_settings: Option<String>,
    pub munkis_issue_id: Option<u64>,
    pub low_disk_space: Option<u64>,
    pub disable_failing_policies: Option<bool>,
    pub macos_settings_disk_encryption: Option<String>,
    pub bootstrap_package: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteHostsBody {
    pub ids: Option<Vec<u64>>,
    pub filters: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SearchHostsBody {
    pub query: Option<String>,
    pub excluded_host_ids: Option<Vec<u64>>,
}

#[derive(Debug, Deserialize)]
pub struct TransferHostsBody {
    pub team_id: Option<u64>,
    pub hosts: Vec<u64>,
}

#[derive(Debug, Deserialize)]
pub struct TransferHostsByFilterBody {
    pub team_id: Option<u64>,
    pub filters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct PutHostDeviceMappingBody {
    pub custom_email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddLabelsToHostBody {
    pub labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchTargetsBody {
    pub query: Option<String>,
    pub query_id: Option<u64>,
    pub selected: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CountTargetsBody {
    pub query_id: Option<u64>,
    pub selected: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct RunLiveQueryOnHostBody {
    pub query: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/host_summary
pub async fn get_host_summary() -> FleetResponse {
    fleet_ok("host_summary", serde_json::json!({}))
}

/// GET /api/_version_/fleet/hosts
pub async fn list_hosts(Query(_params): Query<ListHostsParams>) -> FleetResponse {
    fleet_ok("hosts", serde_json::json!([]))
}

/// POST /api/_version_/fleet/hosts/delete
pub async fn delete_hosts(Json(_body): Json<DeleteHostsBody>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/hosts/{id}
pub async fn get_host(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("host", serde_json::json!({}))
}

/// GET /api/_version_/fleet/hosts/count
pub async fn count_hosts(Query(_params): Query<ListHostsParams>) -> FleetResponse {
    fleet_ok("count", serde_json::json!(0))
}

/// POST /api/_version_/fleet/hosts/search
pub async fn search_hosts(Json(_body): Json<SearchHostsBody>) -> FleetResponse {
    fleet_ok("hosts", serde_json::json!([]))
}

/// GET /api/_version_/fleet/hosts/identifier/{identifier}
pub async fn host_by_identifier(Path(_identifier): Path<String>) -> FleetResponse {
    fleet_ok("host", serde_json::json!({}))
}

/// POST /api/_version_/fleet/hosts/identifier/{identifier}/query
pub async fn run_live_query_on_host(
    Path(_identifier): Path<String>,
    Json(_body): Json<RunLiveQueryOnHostBody>,
) -> FleetResponse {
    fleet_ok("results", serde_json::json!([]))
}

/// POST /api/_version_/fleet/hosts/{id}/query
pub async fn run_live_query_on_host_by_id(
    Path(_id): Path<u64>,
    Json(_body): Json<RunLiveQueryOnHostBody>,
) -> FleetResponse {
    fleet_ok("results", serde_json::json!([]))
}

/// DELETE /api/_version_/fleet/hosts/{id}
pub async fn delete_host(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/hosts/transfer
pub async fn add_hosts_to_team(Json(_body): Json<TransferHostsBody>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/hosts/transfer/filter
pub async fn add_hosts_to_team_by_filter(
    Json(_body): Json<TransferHostsByFilterBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/hosts/{id}/refetch
pub async fn refetch_host(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/hosts/{id}/device_mapping
pub async fn list_host_device_mapping(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("device_mapping", serde_json::json!([]))
}

/// PUT /api/_version_/fleet/hosts/{id}/device_mapping
pub async fn put_host_device_mapping(
    Path(_id): Path<u64>,
    Json(_body): Json<PutHostDeviceMappingBody>,
) -> FleetResponse {
    fleet_ok("device_mapping", serde_json::json!([]))
}

/// DELETE /api/_version_/fleet/hosts/{id}/device_mapping/idp
pub async fn delete_host_idp(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/hosts/report
pub async fn hosts_report() -> FleetResponse {
    fleet_ok("hosts", serde_json::json!([]))
}

/// GET /api/_version_/fleet/os_versions
pub async fn os_versions() -> FleetResponse {
    fleet_ok("os_versions", serde_json::json!([]))
}

/// GET /api/_version_/fleet/os_versions/{id}
pub async fn get_os_version(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("os_version", serde_json::json!({}))
}

/// GET /api/_version_/fleet/hosts/{id}/reports/{report_id}
pub async fn get_host_query_report(Path((_id, _report_id)): Path<(u64, u64)>) -> FleetResponse {
    fleet_ok("report", serde_json::json!({}))
}

/// GET /api/_version_/fleet/hosts/{id}/health
pub async fn get_host_health(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("host_health", serde_json::json!({}))
}

/// POST /api/_version_/fleet/hosts/{id}/labels
pub async fn add_labels_to_host(
    Path(_id): Path<u64>,
    Json(_body): Json<AddLabelsToHostBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/hosts/{id}/labels
pub async fn remove_labels_from_host(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/hosts/{id}/software
pub async fn get_host_software(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("software", serde_json::json!([]))
}

/// GET /api/_version_/fleet/hosts/{id}/certificates
pub async fn list_host_certificates(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("certificates", serde_json::json!([]))
}

/// GET /api/_version_/fleet/hosts/summary/mdm
pub async fn get_host_mdm_summary() -> FleetResponse {
    fleet_ok("mdm_summary", serde_json::json!({}))
}

/// GET /api/_version_/fleet/hosts/{id}/mdm
pub async fn get_host_mdm(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("host_mdm", serde_json::json!({}))
}

/// GET /api/_version_/fleet/hosts/{id}/macadmins
pub async fn get_macadmins_data(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("macadmins", serde_json::json!({}))
}

/// GET /api/_version_/fleet/macadmins
pub async fn get_aggregated_macadmins_data() -> FleetResponse {
    fleet_ok("macadmins", serde_json::json!({}))
}

/// GET /api/_version_/fleet/hosts/{id}/scripts
pub async fn get_host_script_details(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("scripts", serde_json::json!([]))
}

/// GET /api/_version_/fleet/hosts/{id}/activities/upcoming
pub async fn list_host_upcoming_activities(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("activities", serde_json::json!([]))
}

/// DELETE /api/_version_/fleet/hosts/{id}/activities/upcoming/{activity_id}
pub async fn cancel_host_upcoming_activity(
    Path((_id, _activity_id)): Path<(u64, u64)>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/hosts/{id}/lock
pub async fn lock_host(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/hosts/{id}/unlock
pub async fn unlock_host(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/hosts/{id}/wipe
pub async fn wipe_host(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/targets
pub async fn search_targets(Json(_body): Json<SearchTargetsBody>) -> FleetResponse {
    fleet_ok("targets", serde_json::json!({}))
}

/// POST /api/_version_/fleet/targets/count
pub async fn count_targets(Json(_body): Json<CountTargetsBody>) -> FleetResponse {
    fleet_ok("targets_count", serde_json::json!(0))
}
