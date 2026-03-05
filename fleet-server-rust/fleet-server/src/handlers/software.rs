//! Software management endpoints.
//!
//! Handles software versions, titles, installers, app store apps,
//! Fleet-maintained apps, VPP associations, vulnerabilities, and icons.

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

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
    Query(_params): Query<ListSoftwareTitlesParams>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("software_titles", serde_json::json!([]))
}

/// GET /api/_version_/fleet/software/titles/{id}
pub async fn get_software_title(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_id): Path<u64>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("software_title", serde_json::json!({}))
}

/// POST /api/_version_/fleet/hosts/{host_id}/software/{software_title_id}/install
pub async fn install_software_title(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path((_host_id, _software_title_id)): Path<(u64, u64)>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/hosts/{host_id}/software/{software_title_id}/uninstall
pub async fn uninstall_software_title(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path((_host_id, _software_title_id)): Path<(u64, u64)>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/software/titles/{title_id}/package
pub async fn get_software_installer(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_title_id): Path<u64>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // TODO: return binary installer content
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/software/titles/{title_id}/package/token
pub async fn get_software_installer_token(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_title_id): Path<u64>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("token", serde_json::json!(""))
}

/// POST /api/_version_/fleet/software/package
pub async fn upload_software_installer(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // TODO: handle multipart upload
    fleet_ok("", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/software/titles/{id}/name
pub async fn update_software_name(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_id): Path<u64>,
    Json(_body): Json<UpdateSoftwareNameBody>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/software/titles/{id}/package
pub async fn update_software_installer(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_id): Path<u64>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // TODO: handle multipart upload
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/software/titles/{title_id}/available_for_install
pub async fn delete_software_installer(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_title_id): Path<u64>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/software/install/{install_uuid}/results
pub async fn get_software_install_results(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_install_uuid): Path<String>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("results", serde_json::json!({}))
}

/// POST /api/_version_/fleet/software/batch
pub async fn batch_set_software_installers(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(_body): Json<BatchSetSoftwareInstallersBody>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("request_uuid", serde_json::json!(""))
}

/// GET /api/_version_/fleet/software/batch/{request_uuid}
pub async fn batch_set_software_installers_result(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_request_uuid): Path<String>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("status", serde_json::json!({}))
}

/// GET /api/_version_/fleet/software/titles/{title_id}/icon
pub async fn get_software_title_icon(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_title_id): Path<u64>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // TODO: return icon binary
    fleet_ok("", serde_json::json!({}))
}

/// PUT /api/_version_/fleet/software/titles/{title_id}/icon
pub async fn put_software_title_icon(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_title_id): Path<u64>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // TODO: handle multipart upload
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/software/titles/{title_id}/icon
pub async fn delete_software_title_icon(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(_title_id): Path<u64>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/software/app_store_apps
pub async fn get_app_store_apps(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(_params): Query<GetAppStoreAppsParams>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("app_store_apps", serde_json::json!([]))
}

/// POST /api/_version_/fleet/software/app_store_apps
pub async fn add_app_store_app(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(_body): Json<AddAppStoreAppBody>,
) -> FleetResponse {
    let _viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    fleet_ok("", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/software/titles/{title_id}/app_store_app
pub async fn update_app_store_app(
    State(_state): State<AppState>,
    Path(_title_id): Path<u64>,
    Json(_body): Json<UpdateAppStoreAppBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/software/fleet_maintained_apps
pub async fn add_fleet_maintained_app(
    State(_state): State<AppState>,
    Json(_body): Json<AddFleetMaintainedAppBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/software/fleet_maintained_apps
pub async fn list_fleet_maintained_apps(
    State(_state): State<AppState>,
    Query(_params): Query<ListFleetMaintainedAppsParams>,
) -> FleetResponse {
    fleet_ok("fleet_maintained_apps", serde_json::json!([]))
}

/// GET /api/_version_/fleet/software/fleet_maintained_apps/{app_id}
pub async fn get_fleet_maintained_app(
    State(_state): State<AppState>,
    Path(_app_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("fleet_maintained_app", serde_json::json!({}))
}

/// POST /api/_version_/fleet/software/app_store_apps/batch
pub async fn batch_associate_app_store_apps(
    State(_state): State<AppState>,
    Json(_body): Json<BatchAssociateAppStoreAppsBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/software/web_apps
pub async fn create_android_web_app(
    State(_state): State<AppState>,
    Json(_body): Json<CreateAndroidWebAppBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/vulnerabilities
pub async fn list_vulnerabilities(
    State(_state): State<AppState>,
    Query(_params): Query<ListVulnerabilitiesParams>,
) -> FleetResponse {
    fleet_ok("vulnerabilities", serde_json::json!([]))
}

/// GET /api/_version_/fleet/vulnerabilities/{cve}
pub async fn get_vulnerability(
    State(_state): State<AppState>,
    Path(_cve): Path<String>,
) -> FleetResponse {
    fleet_ok("vulnerability", serde_json::json!({}))
}

/// GET /api/_version_/fleet/software/titles/{title_id}/package/token/{token}
pub async fn download_software_installer(
    State(_state): State<AppState>,
    Path((_title_id, _token)): Path<(u64, String)>,
) -> FleetResponse {
    // TODO: validate token, return binary installer
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/software/titles/{title_id}/in_house_app
pub async fn get_in_house_app_package(
    State(_state): State<AppState>,
    Path(_title_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/software/titles/{title_id}/in_house_app/manifest
pub async fn get_in_house_app_manifest(
    State(_state): State<AppState>,
    Path(_title_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}
