//! Host management endpoints.
//!
//! Handles host listing, details, transfers, refetch, device mappings,
//! OS versions, macadmins data, scripts, activities, and host actions.

use axum::extract::{Json, Path, Query, State};
use serde::Deserialize;

use crate::middleware::auth::AuthenticatedUser;
use crate::response::{encode_service_error, fleet_error, fleet_ok, FleetResponse};
use crate::AppState;

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
pub struct HostMDMSummaryParams {
    pub team_id: Option<u32>,
    pub platform: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AggregatedMacadminsParams {
    pub team_id: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct RunLiveQueryOnHostBody {
    pub query: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/host_summary
pub async fn get_host_summary(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.host_summary(&viewer).await {
        Ok(summary) => fleet_ok("host_summary", serde_json::to_value(&summary).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/hosts
pub async fn list_hosts(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListHostsParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let status_filter = params.status.as_deref().and_then(|s| match s {
        "online" => Some(fleet_types::HostStatus::Online),
        "offline" => Some(fleet_types::HostStatus::Offline),
        "mia" | "missing" => Some(fleet_types::HostStatus::MIA),
        "new" => Some(fleet_types::HostStatus::New),
        _ => None,
    });
    let opts = fleet_types::HostListOptions {
        list_options: fleet_types::ListOptions {
            page: params.page.unwrap_or(0) as u32,
            per_page: params.per_page.unwrap_or(0) as u32,
            order_key: params.order_key.unwrap_or_default(),
            match_query: params.query.unwrap_or_default(),
            ..Default::default()
        },
        status_filter,
        team_filter: params.team_id.map(|v| v as u32),
        label_id_filter: params.label_id.map(|v| v as u32),
        policy_id_filter: params.policy_id.map(|v| v as u32),
        software_id_filter: params.software_id.map(|v| v as u32),
        software_version_id_filter: params.software_version_id.map(|v| v as u32),
        software_title_id_filter: params.software_title_id.map(|v| v as u32),
        os_id_filter: params.os_id.map(|v| v as u32),
        os_name_filter: params.os_name,
        os_version_filter: params.os_version,
        vulnerability_filter: params.vulnerability,
        mdm_id_filter: params.mdm_id.map(|v| v as u32),
        mdm_name_filter: params.mdm_name,
        munki_issue_id_filter: params.munkis_issue_id.map(|v| v as u32),
        low_disk_space_filter: params.low_disk_space.map(|v| v as i32),
        ..Default::default()
    };
    match state.service.list_hosts(&viewer, opts).await {
        Ok(hosts) => fleet_ok("hosts", serde_json::to_value(&hosts).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/hosts/delete
pub async fn delete_hosts(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<DeleteHostsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    if let Some(ids) = body.ids {
        let ids: Vec<u32> = ids.iter().map(|&id| id as u32).collect();
        match state.service.delete_hosts(&viewer, &ids).await {
            Ok(()) => fleet_ok("", serde_json::json!({})),
            Err(e) => encode_service_error(&e),
        }
    } else if let Some(filters) = body.filters {
        // Parse filter fields to build HostListOptions
        let team_id = filters.get("team_id").and_then(|v| v.as_u64()).map(|v| v as u32);
        let label_id = filters.get("label_id").and_then(|v| v.as_u64()).map(|v| v as u32);
        let status = filters.get("status").and_then(|v| v.as_str()).and_then(|s| match s {
            "online" => Some(fleet_types::HostStatus::Online),
            "offline" => Some(fleet_types::HostStatus::Offline),
            "mia" | "missing" => Some(fleet_types::HostStatus::MIA),
            "new" => Some(fleet_types::HostStatus::New),
            _ => None,
        });
        let query = filters.get("query").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let opts = fleet_types::HostListOptions {
            list_options: fleet_types::ListOptions {
                per_page: 0, // unlimited
                match_query: query,
                ..Default::default()
            },
            team_filter: team_id,
            label_id_filter: label_id,
            status_filter: status,
            ..Default::default()
        };
        match state.service.delete_hosts_by_filter(&viewer, opts).await {
            Ok(count) => fleet_ok("", serde_json::json!({"hosts_deleted": count})),
            Err(e) => encode_service_error(&e),
        }
    } else {
        fleet_ok("", serde_json::json!({}))
    }
}

/// GET /api/_version_/fleet/hosts/{id}
pub async fn get_host(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_host(&viewer, id as u32).await {
        Ok(host_detail) => fleet_ok("host", serde_json::to_value(&host_detail).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/hosts/count
pub async fn count_hosts(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListHostsParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let status_filter = params.status.as_deref().and_then(|s| match s {
        "online" => Some(fleet_types::HostStatus::Online),
        "offline" => Some(fleet_types::HostStatus::Offline),
        "mia" | "missing" => Some(fleet_types::HostStatus::MIA),
        "new" => Some(fleet_types::HostStatus::New),
        _ => None,
    });
    let opts = fleet_types::HostListOptions {
        list_options: fleet_types::ListOptions {
            match_query: params.query.unwrap_or_default(),
            ..Default::default()
        },
        status_filter,
        team_filter: params.team_id.map(|v| v as u32),
        label_id_filter: params.label_id.map(|v| v as u32),
        ..Default::default()
    };
    match state.service.count_hosts(&viewer, opts).await {
        Ok(count) => fleet_ok("count", serde_json::json!(count)),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/hosts/search
pub async fn search_hosts(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<SearchHostsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let opts = fleet_types::HostListOptions {
        list_options: fleet_types::ListOptions {
            match_query: body.query.unwrap_or_default(),
            ..Default::default()
        },
        ..Default::default()
    };
    match state.service.list_hosts(&viewer, opts).await {
        Ok(hosts) => fleet_ok("hosts", serde_json::to_value(&hosts).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/hosts/identifier/{identifier}
pub async fn host_by_identifier(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(identifier): Path<String>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_host_by_identifier(&viewer, &identifier).await {
        Ok(host_detail) => fleet_ok("host", serde_json::to_value(&host_detail).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/hosts/identifier/{identifier}/query
pub async fn run_live_query_on_host(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(identifier): Path<String>,
    Json(body): Json<RunLiveQueryOnHostBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Look up the host by identifier to get the host ID
    let host_detail = match state.service.get_host_by_identifier(&viewer, &identifier).await {
        Ok(h) => h,
        Err(e) => return encode_service_error(&e),
    };
    let host_id = host_detail.host.id;
    match state.service.run_live_query_on_host(&viewer, &body.query, host_id).await {
        Ok(results) => fleet_ok("results", serde_json::to_value(&results).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/hosts/{id}/query
pub async fn run_live_query_on_host_by_id(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<RunLiveQueryOnHostBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.run_live_query_on_host(&viewer, &body.query, id as u32).await {
        Ok(results) => fleet_ok("results", serde_json::to_value(&results).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/hosts/{id}
pub async fn delete_host(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_host(&viewer, id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/hosts/transfer
pub async fn add_hosts_to_team(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<TransferHostsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let host_ids: Vec<u32> = body.hosts.iter().map(|&id| id as u32).collect();
    let team_id = body.team_id.map(|t| t as u32);
    match state.service.add_hosts_to_team(&viewer, &host_ids, team_id).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/hosts/transfer/filter
pub async fn add_hosts_to_team_by_filter(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<TransferHostsByFilterBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Parse filter fields to build HostListOptions
    let team_id = body.filters.get("team_id").and_then(|v| v.as_u64()).map(|v| v as u32);
    let label_id = body.filters.get("label_id").and_then(|v| v.as_u64()).map(|v| v as u32);
    let status = body.filters.get("status").and_then(|v| v.as_str()).and_then(|s| match s {
        "online" => Some(fleet_types::HostStatus::Online),
        "offline" => Some(fleet_types::HostStatus::Offline),
        "mia" | "missing" => Some(fleet_types::HostStatus::MIA),
        "new" => Some(fleet_types::HostStatus::New),
        _ => None,
    });
    let query = body.filters.get("query").and_then(|v| v.as_str()).unwrap_or_default().to_string();
    let opts = fleet_types::HostListOptions {
        list_options: fleet_types::ListOptions {
            per_page: 0,
            match_query: query,
            ..Default::default()
        },
        team_filter: team_id,
        label_id_filter: label_id,
        status_filter: status,
        ..Default::default()
    };
    let target_team_id = body.team_id.map(|t| t as u32);
    match state.service.add_hosts_to_team_by_filter(&viewer, opts, target_team_id).await {
        Ok(count) => fleet_ok("", serde_json::json!({"hosts_transferred": count})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/hosts/{id}/refetch
pub async fn refetch_host(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.refetch_host(&viewer, id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/hosts/{id}/device_mapping
pub async fn list_host_device_mapping(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.list_host_device_mapping(&viewer, id as u32).await {
        Ok(mapping) => fleet_ok("device_mapping", mapping),
        Err(e) => encode_service_error(&e),
    }
}

/// PUT /api/_version_/fleet/hosts/{id}/device_mapping
pub async fn put_host_device_mapping(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<PutHostDeviceMappingBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let email = body.custom_email.as_deref().unwrap_or("");
    match state.service.put_host_device_mapping(&viewer, id as u32, email).await {
        Ok(mapping) => fleet_ok("device_mapping", mapping),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/hosts/{id}/device_mapping/idp
pub async fn delete_host_idp(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_host_idp_device_mapping(&viewer, id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/hosts/report
pub async fn hosts_report(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.hosts_report(&viewer).await {
        Ok(hosts) => fleet_ok("hosts", serde_json::to_value(&hosts).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/os_versions
pub async fn os_versions(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.list_os_versions(&viewer).await {
        Ok(versions) => fleet_ok("os_versions", serde_json::to_value(&versions).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/os_versions/{id}
pub async fn get_os_version(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_os_version(&viewer, id as u32).await {
        Ok(version) => fleet_ok("os_version", serde_json::to_value(&version).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/hosts/{id}/reports/{report_id}
pub async fn get_host_query_report(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path((id, report_id)): Path<(u64, u64)>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Verify host exists
    if let Err(e) = state.service.get_host_lite(&viewer, id as u32).await {
        return encode_service_error(&e);
    }
    match state.service.get_query_report(&viewer, report_id as u32).await {
        Ok(report) => fleet_ok("report", serde_json::to_value(&report).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/hosts/{id}/health
pub async fn get_host_health(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_host_health(&viewer, id as u32).await {
        Ok(health) => fleet_ok("host_health", serde_json::to_value(&health).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/hosts/{id}/labels
pub async fn add_labels_to_host(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<AddLabelsToHostBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let host_id = id as u32;
    match state.service.add_labels_to_host(&viewer, host_id, &body.labels).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/hosts/{id}/labels
pub async fn remove_labels_from_host(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<AddLabelsToHostBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let host_id = id as u32;
    match state.service.remove_labels_from_host(&viewer, host_id, &body.labels).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/hosts/{id}/software
pub async fn get_host_software(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.list_host_software(&viewer, id as u32).await {
        Ok(software) => fleet_ok("software", serde_json::to_value(&software).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/hosts/{id}/certificates
pub async fn list_host_certificates(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.list_host_certificates(&viewer, id as u32).await {
        Ok(certs) => fleet_ok("certificates", serde_json::to_value(&certs).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/hosts/summary/mdm
pub async fn get_host_mdm_summary(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<HostMDMSummaryParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let platform = params.platform.as_deref().unwrap_or("");
    match state.service.aggregated_mdm_data(&viewer, params.team_id, platform).await {
        Ok(data) => fleet_ok("", serde_json::to_value(&data).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/hosts/{id}/mdm
pub async fn get_host_mdm(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_host_mdm_data(&viewer, id as u32).await {
        Ok(data) => fleet_ok("host_mdm", serde_json::to_value(&data).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/hosts/{id}/macadmins
pub async fn get_macadmins_data(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_macadmins_data(&viewer, id as u32).await {
        Ok(data) => fleet_ok("macadmins", serde_json::to_value(&data).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/macadmins
pub async fn get_aggregated_macadmins_data(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<AggregatedMacadminsParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.aggregated_macadmins_data(&viewer, params.team_id).await {
        Ok(data) => fleet_ok("macadmins", serde_json::to_value(&data).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/hosts/{id}/scripts
pub async fn get_host_script_details(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Get the host to find its team, then list scripts for that team
    match state.service.get_host_lite(&viewer, id as u32).await {
        Ok(host) => {
            match state.service.list_scripts(&viewer, host.team_id).await {
                Ok(scripts) => fleet_ok("scripts", serde_json::to_value(&scripts).unwrap_or_default()),
                Err(e) => encode_service_error(&e),
            }
        }
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/hosts/{id}/activities/upcoming
pub async fn list_host_upcoming_activities(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.list_host_upcoming_activities(&viewer, id as u32).await {
        Ok(activities) => {
            let count = activities.len();
            fleet_ok("", serde_json::json!({
                "count": count,
                "host_id": id,
                "activities": serde_json::to_value(&activities).unwrap_or_default(),
            }))
        }
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/hosts/{id}/activities/upcoming/{activity_id}
pub async fn cancel_host_upcoming_activity(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path((id, activity_id)): Path<(u64, u64)>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.cancel_host_upcoming_activity(&viewer, id as u32, activity_id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/hosts/{id}/lock
pub async fn lock_host(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.lock_host(&viewer, id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/hosts/{id}/unlock
pub async fn unlock_host(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.unlock_host(&viewer, id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/hosts/{id}/wipe
pub async fn wipe_host(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.wipe_host(&viewer, id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/targets
pub async fn search_targets(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<SearchTargetsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let query_str = body.query.as_deref().unwrap_or("");
    match state.service.search_targets(&viewer, query_str, &[]).await {
        Ok(hosts) => fleet_ok("targets", serde_json::json!({
            "hosts": serde_json::to_value(&hosts).unwrap_or_default(),
        })),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/targets/count
pub async fn count_targets(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<CountTargetsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &body;
    match state.service.search_targets(&viewer, "", &[]).await {
        Ok(hosts) => fleet_ok("targets_count", serde_json::json!(hosts.len())),
        Err(e) => encode_service_error(&e),
    }
}
