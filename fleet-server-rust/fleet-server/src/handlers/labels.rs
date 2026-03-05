//! Label management endpoints.
//!
//! Handles label CRUD, specs, and listing hosts within labels.

use axum::extract::{Json, Path, Query, State};
use serde::Deserialize;

use crate::middleware::auth::AuthenticatedUser;
use crate::response::{encode_service_error, fleet_error, fleet_ok, FleetResponse};
use crate::AppState;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListLabelsParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLabelBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub query: Option<String>,
    pub platform: Option<String>,
    pub label_membership_type: Option<String>,
    pub hosts: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ModifyLabelBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub hosts: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ApplyLabelSpecsBody {
    pub specs: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListHostsInLabelParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
    pub status: Option<String>,
    pub query: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/labels
pub async fn create_label(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<CreateLabelBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_types::label::LabelPayload {
        name: body.name.unwrap_or_default(),
        description: body.description.unwrap_or_default(),
        query: body.query.unwrap_or_default(),
        platform: body.platform.unwrap_or_default(),
        hosts: body.hosts.unwrap_or_default(),
        host_ids: Vec::new(),
        criteria: None,
    };
    match state.service.new_label(&viewer, payload).await {
        Ok(label) => fleet_ok("label", serde_json::to_value(&label).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/labels/{id}
pub async fn modify_label(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<ModifyLabelBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_types::label::ModifyLabelPayload {
        name: body.name,
        description: body.description,
        hosts: None,
        host_ids: None,
    };
    match state.service.modify_label(&viewer, id as u32, payload).await {
        Ok(label) => fleet_ok("label", serde_json::to_value(&label).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/labels/{id}
pub async fn get_label(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_label(&viewer, id as u32).await {
        Ok(label) => fleet_ok("label", serde_json::to_value(&label).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/labels
pub async fn list_labels(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListLabelsParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let opts = fleet_types::ListOptions {
        page: params.page.unwrap_or(0) as u32,
        per_page: params.per_page.unwrap_or(0) as u32,
        order_key: params.order_key.unwrap_or_default(),
        match_query: params.query.unwrap_or_default(),
        ..Default::default()
    };
    match state.service.list_labels(&viewer, opts).await {
        Ok(labels) => fleet_ok("labels", serde_json::to_value(&labels).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/labels/summary
pub async fn get_labels_summary(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.labels_summary(&viewer).await {
        Ok(labels) => fleet_ok("labels", serde_json::to_value(&labels).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/labels/{id}/hosts
pub async fn list_hosts_in_label(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Query(params): Query<ListHostsInLabelParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Verify label exists
    if let Err(e) = state.service.get_label(&viewer, id as u32).await {
        return encode_service_error(&e);
    }
    // List hosts with label filter
    let opts = fleet_types::HostListOptions {
        list_options: fleet_types::ListOptions {
            page: params.page.unwrap_or(0) as u32,
            per_page: params.per_page.unwrap_or(0) as u32,
            order_key: params.order_key.unwrap_or_default(),
            match_query: params.query.unwrap_or_default(),
            ..Default::default()
        },
        label_id_filter: Some(id as u32),
        ..Default::default()
    };
    match state.service.list_hosts(&viewer, opts).await {
        Ok(hosts) => fleet_ok("hosts", serde_json::to_value(&hosts).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/labels/{name}
pub async fn delete_label(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(name): Path<String>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_label(&viewer, &name).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/labels/id/{id}
pub async fn delete_label_by_id(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_label_by_id(&viewer, id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/spec/labels
pub async fn apply_label_specs(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<ApplyLabelSpecsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let specs: Vec<fleet_service::labels::LabelSpec> = body.specs.into_iter().filter_map(|v| {
        serde_json::from_value(v).ok()
    }).collect();
    match state.service.apply_label_specs(&viewer, specs).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/spec/labels
pub async fn get_label_specs(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_label_specs(&viewer).await {
        Ok(specs) => fleet_ok("specs", serde_json::to_value(&specs).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/spec/labels/{name}
pub async fn get_label_spec(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(name): Path<String>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_label_spec(&viewer, &name).await {
        Ok(spec) => fleet_ok("spec", serde_json::to_value(&spec).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}
