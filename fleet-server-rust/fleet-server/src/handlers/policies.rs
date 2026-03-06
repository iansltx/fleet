//! Policy management endpoints.
//!
//! Handles global and team policy CRUD, specs, and automations.

use axum::extract::{Json, Path, Query, State};
use serde::Deserialize;

use crate::middleware::auth::AuthenticatedUser;
use crate::response::{encode_service_error, fleet_error, fleet_ok, FleetResponse};
use crate::AppState;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListGlobalPoliciesParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GlobalPolicyBody {
    pub name: Option<String>,
    pub query: Option<String>,
    pub description: Option<String>,
    pub resolution: Option<String>,
    pub platform: Option<String>,
    pub critical: Option<bool>,
    pub calendar_events_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ModifyGlobalPolicyBody {
    pub name: Option<String>,
    pub query: Option<String>,
    pub description: Option<String>,
    pub resolution: Option<String>,
    pub platform: Option<String>,
    pub critical: Option<bool>,
    pub calendar_events_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteGlobalPoliciesBody {
    pub ids: Vec<u64>,
}

#[derive(Debug, Deserialize)]
pub struct TeamPolicyBody {
    pub name: Option<String>,
    pub query: Option<String>,
    pub description: Option<String>,
    pub resolution: Option<String>,
    pub platform: Option<String>,
    pub critical: Option<bool>,
    pub calendar_events_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteTeamPoliciesBody {
    pub ids: Vec<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ApplyPolicySpecsBody {
    pub specs: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ResetAutomationBody {
    pub team_ids: Option<Vec<u64>>,
    pub policy_ids: Option<Vec<u64>>,
}

#[derive(Debug, Deserialize)]
pub struct CountGlobalPoliciesParams {
    pub query: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CountTeamPoliciesParams {
    pub query: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListTeamPoliciesParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_key: Option<String>,
    pub order_direction: Option<String>,
    pub query: Option<String>,
    pub inherited_page: Option<u64>,
    pub inherited_per_page: Option<u64>,
    pub inherited_order_key: Option<String>,
    pub inherited_order_direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AutofillPoliciesBody {
    pub query: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/global/policies  (v1)
/// POST /api/_version_/fleet/policies  (2022-04)
pub async fn create_global_policy(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<GlobalPolicyBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_types::policy::PolicyPayload {
        name: body.name.unwrap_or_default(),
        query: body.query.unwrap_or_default(),
        description: body.description.unwrap_or_default(),
        resolution: body.resolution.unwrap_or_default(),
        platform: body.platform.unwrap_or_default(),
        critical: body.critical.unwrap_or(false),
        calendar_events_enabled: body.calendar_events_enabled.unwrap_or(false),
        query_id: None,
        software_installer_id: None,
        vpp_apps_teams_id: None,
        script_id: None,
        labels_include_any: Vec::new(),
        labels_exclude_any: Vec::new(),
        conditional_access_enabled: false,
        conditional_access_bypass_enabled: None,
    };
    match state.service.new_global_policy(&viewer, payload).await {
        Ok(policy) => fleet_ok("policy", serde_json::to_value(&policy).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/global/policies  (v1)
/// GET /api/_version_/fleet/policies  (2022-04)
pub async fn list_global_policies(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListGlobalPoliciesParams>,
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
    match state.service.list_global_policies(&viewer, opts).await {
        Ok(policies) => fleet_ok("policies", serde_json::to_value(&policies).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/policies/count
pub async fn count_global_policies(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<CountGlobalPoliciesParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let opts = fleet_types::ListOptions {
        match_query: params.query.unwrap_or_default(),
        ..Default::default()
    };
    match state.service.list_global_policies(&viewer, opts).await {
        Ok(policies) => fleet_ok("count", serde_json::json!(policies.len())),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/global/policies/{policy_id}  (v1)
/// GET /api/_version_/fleet/policies/{policy_id}  (2022-04)
pub async fn get_policy(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(policy_id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_policy(&viewer, policy_id as u32).await {
        Ok(policy) => fleet_ok("policy", serde_json::to_value(&policy).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/global/policies/delete  (v1)
/// POST /api/_version_/fleet/policies/delete  (2022-04)
pub async fn delete_global_policies(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<DeleteGlobalPoliciesBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let ids: Vec<u32> = body.ids.iter().map(|&id| id as u32).collect();
    match state.service.delete_global_policies(&viewer, &ids).await {
        Ok(deleted) => fleet_ok("ids", serde_json::to_value(&deleted).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/global/policies/{policy_id}  (v1)
/// PATCH /api/_version_/fleet/policies/{policy_id}  (2022-04)
pub async fn modify_global_policy(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(policy_id): Path<u64>,
    Json(body): Json<ModifyGlobalPolicyBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_types::policy::ModifyPolicyPayload {
        name: body.name,
        query: body.query,
        description: body.description,
        resolution: body.resolution,
        platform: body.platform,
        critical: body.critical,
        calendar_events_enabled: body.calendar_events_enabled,
        software_title_id: None,
        script_id: None,
        labels_include_any: Vec::new(),
        labels_exclude_any: Vec::new(),
        conditional_access_enabled: None,
        conditional_access_bypass_enabled: None,
    };
    match state.service.modify_policy(&viewer, policy_id as u32, payload).await {
        Ok(policy) => fleet_ok("policy", serde_json::to_value(&policy).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/automations/reset
pub async fn reset_automation(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<ResetAutomationBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };

    // Validate that referenced teams exist.
    if let Some(ref team_ids) = body.team_ids {
        for &tid in team_ids {
            if let Err(e) = state.service.get_team(&viewer, tid as u32).await {
                return encode_service_error(&e);
            }
        }
    }

    // Validate that referenced policies exist.
    if let Some(ref policy_ids) = body.policy_ids {
        for &pid in policy_ids {
            if let Err(e) = state.service.get_policy(&viewer, pid as u32).await {
                return encode_service_error(&e);
            }
        }
    }

    tracing::info!(
        team_ids = ?body.team_ids,
        policy_ids = ?body.policy_ids,
        "reset_automation requested (automation iteration increment is not yet implemented in Rust server)"
    );

    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/fleets/{fleet_id}/policies
pub async fn create_team_policy(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(fleet_id): Path<u64>,
    Json(body): Json<TeamPolicyBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_types::policy::PolicyPayload {
        name: body.name.unwrap_or_default(),
        query: body.query.unwrap_or_default(),
        description: body.description.unwrap_or_default(),
        resolution: body.resolution.unwrap_or_default(),
        platform: body.platform.unwrap_or_default(),
        critical: body.critical.unwrap_or(false),
        calendar_events_enabled: body.calendar_events_enabled.unwrap_or(false),
        query_id: None,
        software_installer_id: None,
        vpp_apps_teams_id: None,
        script_id: None,
        labels_include_any: Vec::new(),
        labels_exclude_any: Vec::new(),
        conditional_access_enabled: false,
        conditional_access_bypass_enabled: None,
    };
    match state.service.new_team_policy(&viewer, fleet_id as u32, payload).await {
        Ok(policy) => fleet_ok("policy", serde_json::to_value(&policy).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/fleets/{fleet_id}/policies
pub async fn list_team_policies(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(fleet_id): Path<u64>,
    Query(params): Query<ListTeamPoliciesParams>,
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
    match state.service.list_team_policies(&viewer, fleet_id as u32, opts).await {
        Ok(policies) => fleet_ok("policies", serde_json::to_value(&policies).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/fleets/{fleet_id}/policies/count
pub async fn count_team_policies(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(fleet_id): Path<u64>,
    Query(params): Query<CountTeamPoliciesParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let opts = fleet_types::ListOptions {
        match_query: params.query.unwrap_or_default(),
        ..Default::default()
    };
    match state.service.list_team_policies(&viewer, fleet_id as u32, opts).await {
        Ok(policies) => fleet_ok("count", serde_json::json!(policies.len())),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/fleets/{fleet_id}/policies/{policy_id}
pub async fn get_team_policy(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path((_fleet_id, policy_id)): Path<(u64, u64)>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_policy(&viewer, policy_id as u32).await {
        Ok(policy) => fleet_ok("policy", serde_json::to_value(&policy).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/fleets/{fleet_id}/policies/delete
pub async fn delete_team_policies(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(fleet_id): Path<u64>,
    Json(body): Json<DeleteTeamPoliciesBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let ids: Vec<u32> = body.ids.iter().map(|&id| id as u32).collect();
    match state.service.delete_team_policies(&viewer, fleet_id as u32, &ids).await {
        Ok(deleted) => fleet_ok("ids", serde_json::to_value(&deleted).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/fleets/{fleet_id}/policies/{policy_id}
pub async fn modify_team_policy(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path((_fleet_id, policy_id)): Path<(u64, u64)>,
    Json(body): Json<ModifyGlobalPolicyBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_types::policy::ModifyPolicyPayload {
        name: body.name,
        query: body.query,
        description: body.description,
        resolution: body.resolution,
        platform: body.platform,
        critical: body.critical,
        calendar_events_enabled: body.calendar_events_enabled,
        software_title_id: None,
        script_id: None,
        labels_include_any: Vec::new(),
        labels_exclude_any: Vec::new(),
        conditional_access_enabled: None,
        conditional_access_bypass_enabled: None,
    };
    match state.service.modify_policy(&viewer, policy_id as u32, payload).await {
        Ok(policy) => fleet_ok("policy", serde_json::to_value(&policy).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/spec/policies
pub async fn apply_policy_specs(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<ApplyPolicySpecsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let specs: Vec<fleet_service::policies::PolicySpec> = body
        .specs
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();
    match state.service.apply_policy_specs(&viewer, specs).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/autofill/policy
pub async fn autofill_policies(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<AutofillPoliciesBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}
