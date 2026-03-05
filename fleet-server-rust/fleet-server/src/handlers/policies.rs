//! Policy management endpoints.
//!
//! Handles global and team policy CRUD, specs, and automations.

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
pub async fn create_global_policy(Json(_body): Json<GlobalPolicyBody>) -> FleetResponse {
    fleet_ok("policy", serde_json::json!({}))
}

/// GET /api/_version_/fleet/global/policies  (v1)
/// GET /api/_version_/fleet/policies  (2022-04)
pub async fn list_global_policies(
    Query(_params): Query<ListGlobalPoliciesParams>,
) -> FleetResponse {
    fleet_ok("policies", serde_json::json!([]))
}

/// GET /api/_version_/fleet/policies/count
pub async fn count_global_policies(
    Query(_params): Query<CountGlobalPoliciesParams>,
) -> FleetResponse {
    fleet_ok("count", serde_json::json!(0))
}

/// GET /api/_version_/fleet/global/policies/{policy_id}  (v1)
/// GET /api/_version_/fleet/policies/{policy_id}  (2022-04)
pub async fn get_policy(Path(_policy_id): Path<u64>) -> FleetResponse {
    fleet_ok("policy", serde_json::json!({}))
}

/// POST /api/_version_/fleet/global/policies/delete  (v1)
/// POST /api/_version_/fleet/policies/delete  (2022-04)
pub async fn delete_global_policies(Json(_body): Json<DeleteGlobalPoliciesBody>) -> FleetResponse {
    fleet_ok("ids", serde_json::json!([]))
}

/// PATCH /api/_version_/fleet/global/policies/{policy_id}  (v1)
/// PATCH /api/_version_/fleet/policies/{policy_id}  (2022-04)
pub async fn modify_global_policy(
    Path(_policy_id): Path<u64>,
    Json(_body): Json<ModifyGlobalPolicyBody>,
) -> FleetResponse {
    fleet_ok("policy", serde_json::json!({}))
}

/// POST /api/_version_/fleet/automations/reset
pub async fn reset_automation(Json(_body): Json<ResetAutomationBody>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/fleets/{fleet_id}/policies
pub async fn create_team_policy(
    Path(_fleet_id): Path<u64>,
    Json(_body): Json<TeamPolicyBody>,
) -> FleetResponse {
    fleet_ok("policy", serde_json::json!({}))
}

/// GET /api/_version_/fleet/fleets/{fleet_id}/policies
pub async fn list_team_policies(
    Path(_fleet_id): Path<u64>,
    Query(_params): Query<ListTeamPoliciesParams>,
) -> FleetResponse {
    fleet_ok("policies", serde_json::json!([]))
}

/// GET /api/_version_/fleet/fleets/{fleet_id}/policies/count
pub async fn count_team_policies(
    Path(_fleet_id): Path<u64>,
    Query(_params): Query<CountTeamPoliciesParams>,
) -> FleetResponse {
    fleet_ok("count", serde_json::json!(0))
}

/// GET /api/_version_/fleet/fleets/{fleet_id}/policies/{policy_id}
pub async fn get_team_policy(
    Path((_fleet_id, _policy_id)): Path<(u64, u64)>,
) -> FleetResponse {
    fleet_ok("policy", serde_json::json!({}))
}

/// POST /api/_version_/fleet/fleets/{fleet_id}/policies/delete
pub async fn delete_team_policies(
    Path(_fleet_id): Path<u64>,
    Json(_body): Json<DeleteTeamPoliciesBody>,
) -> FleetResponse {
    fleet_ok("ids", serde_json::json!([]))
}

/// PATCH /api/_version_/fleet/fleets/{fleet_id}/policies/{policy_id}
pub async fn modify_team_policy(
    Path((_fleet_id, _policy_id)): Path<(u64, u64)>,
    Json(_body): Json<ModifyGlobalPolicyBody>,
) -> FleetResponse {
    fleet_ok("policy", serde_json::json!({}))
}

/// POST /api/_version_/fleet/spec/policies
pub async fn apply_policy_specs(Json(_body): Json<ApplyPolicySpecsBody>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/autofill/policy
pub async fn autofill_policies(Json(_body): Json<AutofillPoliciesBody>) -> FleetResponse {
    fleet_ok("policy", serde_json::json!({}))
}
