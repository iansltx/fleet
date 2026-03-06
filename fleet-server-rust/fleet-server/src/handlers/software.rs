//! Software management endpoints.
//!
//! Handles software versions, titles, installers, app store apps,
//! Fleet-maintained apps, VPP associations, vulnerabilities, and icons.

use axum::extract::{Json, Path, Query, State};
use serde::Deserialize;

use crate::middleware::auth::AuthenticatedUser;
use crate::response::{fleet_error, fleet_ok, encode_service_error, FleetResponse};
use crate::AppState;


// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListSoftwareParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
    pub query: Option<String>,
    pub team_id: Option<u64>,
    pub vulnerable: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListSoftwareTitlesParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
    pub query: Option<String>,
    pub team_id: Option<u64>,
    pub available_for_install: Option<bool>,
    pub self_service: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListVulnerabilitiesParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
    pub query: Option<String>,
    pub team_id: Option<u64>,
    pub exploit: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AddAppStoreAppBody {
    pub app_store_id: String,
    pub team_id: Option<u64>,
    pub platform: Option<String>,
    pub self_service: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAppStoreAppBody {
    pub team_id: Option<u64>,
    pub self_service: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct GetAppStoreAppsParams {
    pub team_id: Option<u64>,
    pub platform: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddFleetMaintainedAppBody {
    pub fleet_maintained_app_id: Option<u64>,
    pub team_id: Option<u64>,
    pub self_service: Option<bool>,
    pub pre_install_query: Option<String>,
    pub install_script: Option<String>,
    pub post_install_script: Option<String>,
    pub uninstall_script: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListFleetMaintainedAppsParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub team_id: Option<u64>,
    pub query: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BatchSetSoftwareInstallersBody {
    pub software: Vec<serde_json::Value>,
    pub team_id: Option<u64>,
    pub dry_run: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct BatchAssociateAppStoreAppsBody {
    pub app_store_apps: Vec<serde_json::Value>,
    pub team_id: Option<u64>,
    pub dry_run: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSoftwareNameBody {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAndroidWebAppBody {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/software/versions
pub async fn list_software_versions(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListSoftwareParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let opts = fleet_types::ListOptions {
        page: params.page.unwrap_or(0) as u32,
        per_page: params.per_page.unwrap_or(20) as u32,
        order_key: params.order_key.unwrap_or_default(),
        match_query: params.query.unwrap_or_default(),
        ..Default::default()
    };
    let team_id = params.team_id.map(|v| v as u32);
    match state.service.list_software(&viewer, opts, team_id).await {
        Ok(software) => fleet_ok("software", serde_json::to_value(&software).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/software/versions/{id}
/// GET /api/_version_/fleet/software/{id}
pub async fn get_software(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_software(&viewer, id as u32).await {
        Ok(software) => fleet_ok("software", serde_json::to_value(&software).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/software (deprecated)
pub async fn list_software(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListSoftwareParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let opts = fleet_types::ListOptions {
        page: params.page.unwrap_or(0) as u32,
        per_page: params.per_page.unwrap_or(20) as u32,
        ..Default::default()
    };
    let team_id = params.team_id.map(|v| v as u32);
    match state.service.list_software(&viewer, opts, team_id).await {
        Ok(software) => fleet_ok("software", serde_json::to_value(&software).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/software/count (deprecated)
pub async fn count_software(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListSoftwareParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let opts = fleet_types::ListOptions {
        match_query: params.query.unwrap_or_default(),
        ..Default::default()
    };
    let team_id = params.team_id.map(|v| v as u32);
    match state.service.list_software(&viewer, opts, team_id).await {
        Ok(software) => fleet_ok("count", serde_json::json!(software.len())),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/software/titles
pub async fn list_software_titles(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListSoftwareTitlesParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = params.team_id.map(|v| v as u32);
    let limit = params.per_page.unwrap_or(20) as u32;
    let offset = params.page.unwrap_or(0) as u32 * limit;
    match state.service.list_software_titles(&viewer, team_id, limit, offset).await {
        Ok(titles) => fleet_ok("software_titles", serde_json::to_value(&titles).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/software/titles/{id}
pub async fn get_software_title(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_software_title(&viewer, id as u32).await {
        Ok(title) => fleet_ok("software_title", serde_json::to_value(&title).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/hosts/{host_id}/software/{software_title_id}/install
pub async fn install_software_title(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path((host_id, software_title_id)): Path<(u64, u64)>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let execution_id = uuid::Uuid::new_v4().to_string();
    // Create a tracking record for the software install
    let result = fleet_types::script::HostScriptResult {
        id: 0,
        host_id: host_id as u32,
        execution_id: execution_id.clone(),
        script_id: None,
        script_contents: format!("install_software_title:{}", software_title_id),
        output: String::new(),
        runtime: 0,
        exit_code: None,
        message: Some(format!("Installing software title {}", software_title_id)),
        host_timeout: false,
        host_deleted_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    if let Err(e) = state.service.save_host_script_result(&viewer, &result).await {
        return encode_service_error(&e);
    }
    fleet_ok("execution_id", serde_json::json!(execution_id))
}

/// POST /api/_version_/fleet/hosts/{host_id}/software/{software_title_id}/uninstall
pub async fn uninstall_software_title(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path((host_id, software_title_id)): Path<(u64, u64)>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let execution_id = uuid::Uuid::new_v4().to_string();
    let result = fleet_types::script::HostScriptResult {
        id: 0,
        host_id: host_id as u32,
        execution_id: execution_id.clone(),
        script_id: None,
        script_contents: format!("uninstall_software_title:{}", software_title_id),
        output: String::new(),
        runtime: 0,
        exit_code: None,
        message: Some(format!("Uninstalling software title {}", software_title_id)),
        host_timeout: false,
        host_deleted_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    if let Err(e) = state.service.save_host_script_result(&viewer, &result).await {
        return encode_service_error(&e);
    }
    fleet_ok("execution_id", serde_json::json!(execution_id))
}

/// GET /api/_version_/fleet/software/titles/{title_id}/package
pub async fn get_software_installer(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(title_id): Path<u64>,
) -> impl axum::response::IntoResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_software_installer(&viewer, title_id as u32).await {
        Ok(data) => fleet_ok("installer", serde_json::json!(data)),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/software/titles/{title_id}/package/token
pub async fn get_software_installer_token(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(title_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_software_installer_token(&viewer, title_id as u32).await {
        Ok(token) => fleet_ok("token", serde_json::json!(token)),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/software/package
pub async fn upload_software_installer(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    _multipart: axum::extract::Multipart,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.upload_software_installer(&viewer).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/software/titles/{id}/name
pub async fn update_software_name(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<UpdateSoftwareNameBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.update_software_title_name(&viewer, id as u32, &body.name).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/software/titles/{id}/package
pub async fn update_software_installer(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    _multipart: axum::extract::Multipart,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.update_software_installer(&viewer, id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/software/titles/{title_id}/available_for_install
pub async fn delete_software_installer(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(title_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_software_installer(&viewer, title_id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/software/install/{install_uuid}/results
pub async fn get_software_install_results(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(install_uuid): Path<String>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_software_install_result(&viewer, &install_uuid).await {
        Ok(result) => fleet_ok("results", serde_json::to_value(&result).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/software/batch
pub async fn batch_set_software_installers(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<BatchSetSoftwareInstallersBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = body.team_id.map(|v| v as u32);
    let dry_run = body.dry_run.unwrap_or(false);
    match state.service.batch_set_software_installers(&viewer, &body.software, team_id, dry_run).await {
        Ok(request_uuid) => fleet_ok("request_uuid", serde_json::json!(request_uuid)),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/software/batch/{request_uuid}
pub async fn batch_set_software_installers_result(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(request_uuid): Path<String>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.batch_set_software_installers_result(&viewer, &request_uuid).await {
        Ok(result) => fleet_ok("result", serde_json::to_value(&result).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/software/titles/{title_id}/icon
pub async fn get_software_title_icon(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(title_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_software_title_icon(&viewer, title_id as u32).await {
        Ok(data) => fleet_ok("icon", serde_json::json!(data)),
        Err(e) => encode_service_error(&e),
    }
}

/// PUT /api/_version_/fleet/software/titles/{title_id}/icon
pub async fn put_software_title_icon(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(title_id): Path<u64>,
    _multipart: axum::extract::Multipart,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.put_software_title_icon(&viewer, title_id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/software/titles/{title_id}/icon
pub async fn delete_software_title_icon(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(title_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_software_title_icon(&viewer, title_id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/software/app_store_apps
pub async fn get_app_store_apps(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<GetAppStoreAppsParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = params.team_id.map(|v| v as u32);
    let platform = params.platform.as_deref();
    match state.service.get_app_store_apps(&viewer, team_id, platform).await {
        Ok(apps) => fleet_ok("app_store_apps", serde_json::to_value(&apps).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/software/app_store_apps
pub async fn add_app_store_app(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<AddAppStoreAppBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = body.team_id.map(|v| v as u32);
    let platform = body.platform.as_deref();
    let self_service = body.self_service.unwrap_or(false);
    match state.service.add_app_store_app(&viewer, &body.app_store_id, team_id, platform, self_service).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/software/titles/{title_id}/app_store_app
pub async fn update_app_store_app(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(title_id): Path<u64>,
    Json(body): Json<UpdateAppStoreAppBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = body.team_id.map(|v| v as u32);
    match state.service.update_app_store_app(&viewer, title_id as u32, team_id, body.self_service).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/software/fleet_maintained_apps
pub async fn add_fleet_maintained_app(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<AddFleetMaintainedAppBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = body.team_id.map(|v| v as u32);
    let self_service = body.self_service.unwrap_or(false);
    match state.service.add_fleet_maintained_app_installer(&viewer, body.fleet_maintained_app_id, team_id, self_service).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/software/fleet_maintained_apps
pub async fn list_fleet_maintained_apps(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListFleetMaintainedAppsParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let per_page = params.per_page.unwrap_or(20) as u32;
    let page = params.page.unwrap_or(0) as u32;
    let offset = page * per_page;
    let query_str = params.query.as_deref();
    match state.service.list_fleet_maintained_apps(&viewer, query_str, per_page, offset).await {
        Ok(apps) => fleet_ok("fleet_maintained_apps", serde_json::to_value(&apps).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/software/fleet_maintained_apps/{app_id}
pub async fn get_fleet_maintained_app(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(app_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_fleet_maintained_app(&viewer, app_id as u32).await {
        Ok(app) => fleet_ok("fleet_maintained_app", serde_json::to_value(&app).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/software/app_store_apps/batch
pub async fn batch_associate_app_store_apps(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<BatchAssociateAppStoreAppsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = body.team_id.map(|v| v as u32);
    let dry_run = body.dry_run.unwrap_or(false);
    match state.service.batch_associate_app_store_apps(&viewer, &body.app_store_apps, team_id, dry_run).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/software/web_apps
pub async fn create_android_web_app(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<CreateAndroidWebAppBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.create_android_web_app(&viewer, &body.data).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/vulnerabilities
pub async fn list_vulnerabilities(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListVulnerabilitiesParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let team_id = params.team_id.map(|t| t as u32);
    let per_page = params.per_page.unwrap_or(20) as u32;
    let page = params.page.unwrap_or(0) as u32;
    let offset = page * per_page;
    let query_str = params.query.as_deref();
    match state.service.list_vulnerabilities(&viewer, team_id, query_str, params.exploit, per_page, offset).await {
        Ok(vulns) => fleet_ok("vulnerabilities", serde_json::json!(vulns)),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/vulnerabilities/{cve}
pub async fn get_vulnerability(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(cve): Path<String>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_vulnerability(&viewer, &cve, None).await {
        Ok(vuln) => fleet_ok("vulnerability", serde_json::json!(vuln)),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/software/titles/{title_id}/package/token/{token}
pub async fn download_software_installer(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path((title_id, token)): Path<(u64, String)>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.download_software_installer(&viewer, title_id as u32, &token).await {
        Ok(data) => fleet_ok("installer", serde_json::json!(data)),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/software/titles/{title_id}/in_house_app
pub async fn get_in_house_app_package(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(title_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_in_house_app_package(&viewer, title_id as u32).await {
        Ok(data) => fleet_ok("package", serde_json::json!(data)),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/software/titles/{title_id}/in_house_app/manifest
pub async fn get_in_house_app_manifest(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(title_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_in_house_app_manifest(&viewer, title_id as u32).await {
        Ok(manifest) => fleet_ok("manifest", serde_json::to_value(&manifest).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}
