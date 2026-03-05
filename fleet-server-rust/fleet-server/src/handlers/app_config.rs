//! App configuration, status, and miscellaneous endpoints.
//!
//! Handles app config get/modify, enroll secrets, version, certificates,
//! secret variables, SCIM, conditional access, and more.

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
pub struct ModifyAppConfigBody {
    #[serde(flatten)]
    pub config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ApplyEnrollSecretSpecBody {
    pub spec: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct TranslateBody {
    pub list: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct TriggerBody {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCertificateTemplateBody {
    pub name: Option<String>,
    #[serde(flatten)]
    pub rest: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ApplyCertificateTemplateSpecsBody {
    pub specs: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSecretVariablesBody {
    pub secrets: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSecretVariableBody {
    pub name: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListSecretVariablesParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ConditionalAccessMicrosoftCreateBody {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ConditionalAccessMicrosoftConfirmBody {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CreateCertificateAuthorityBody {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCertificateAuthorityBody {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct RequestCertificateBody {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct BatchApplyCertificateAuthoritiesBody {
    pub specs: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CalendarWebhookBody {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/_version_/fleet/trigger
pub async fn trigger(Json(_body): Json<TriggerBody>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/config/certificate
pub async fn get_certificate() -> FleetResponse {
    fleet_ok("certificate_chain", serde_json::json!(""))
}

/// GET /api/_version_/fleet/config
pub async fn get_app_config() -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/config
pub async fn modify_app_config(Json(_body): Json<ModifyAppConfigBody>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/spec/enroll_secret
pub async fn apply_enroll_secret_spec(
    Json(_body): Json<ApplyEnrollSecretSpecBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/spec/enroll_secret
pub async fn get_enroll_secret_spec() -> FleetResponse {
    fleet_ok("spec", serde_json::json!({}))
}

/// GET /api/_version_/fleet/version
pub async fn version() -> FleetResponse {
    fleet_ok(
        "version",
        serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
        }),
    )
}

/// POST /api/_version_/fleet/translate
pub async fn translate(Json(_body): Json<TranslateBody>) -> FleetResponse {
    fleet_ok("list", serde_json::json!([]))
}

/// POST /api/_version_/fleet/certificates
pub async fn create_certificate_template(
    Json(_body): Json<CreateCertificateTemplateBody>,
) -> FleetResponse {
    fleet_ok("certificate_template", serde_json::json!({}))
}

/// GET /api/_version_/fleet/certificates
pub async fn list_certificate_templates() -> FleetResponse {
    fleet_ok("certificate_templates", serde_json::json!([]))
}

/// GET /api/_version_/fleet/certificates/{id}
pub async fn get_certificate_template(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("certificate_template", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/certificates/{id}
pub async fn delete_certificate_template(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/spec/certificates
pub async fn apply_certificate_template_specs(
    Json(_body): Json<ApplyCertificateTemplateSpecsBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/spec/certificates
pub async fn delete_certificate_template_specs() -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/status/result_store
pub async fn status_result_store() -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/status/live_query
pub async fn status_live_query() -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// PUT /api/_version_/fleet/spec/secret_variables
pub async fn create_secret_variables(
    Json(_body): Json<CreateSecretVariablesBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/custom_variables
pub async fn create_secret_variable(
    Json(_body): Json<CreateSecretVariableBody>,
) -> FleetResponse {
    fleet_ok("secret_variable", serde_json::json!({}))
}

/// GET /api/_version_/fleet/custom_variables
pub async fn list_secret_variables(
    Query(_params): Query<ListSecretVariablesParams>,
) -> FleetResponse {
    fleet_ok("secret_variables", serde_json::json!([]))
}

/// DELETE /api/_version_/fleet/custom_variables/{id}
pub async fn delete_secret_variable(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/scim/details
pub async fn get_scim_details() -> FleetResponse {
    fleet_ok("scim", serde_json::json!({}))
}

/// POST /api/_version_/fleet/conditional-access/microsoft
pub async fn conditional_access_microsoft_create(
    Json(_body): Json<ConditionalAccessMicrosoftCreateBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/conditional-access/microsoft/confirm
pub async fn conditional_access_microsoft_confirm(
    Json(_body): Json<ConditionalAccessMicrosoftConfirmBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/conditional-access/microsoft
pub async fn conditional_access_microsoft_delete() -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/conditional_access/idp/signing_cert
pub async fn conditional_access_get_idp_signing_cert() -> FleetResponse {
    fleet_ok("signing_cert", serde_json::json!(""))
}

/// GET /api/_version_/fleet/conditional_access/idp/apple/profile
pub async fn conditional_access_get_idp_apple_profile() -> FleetResponse {
    fleet_ok("profile", serde_json::json!({}))
}

/// POST /api/_version_/fleet/certificate_authorities
pub async fn create_certificate_authority(
    Json(_body): Json<CreateCertificateAuthorityBody>,
) -> FleetResponse {
    fleet_ok("certificate_authority", serde_json::json!({}))
}

/// GET /api/_version_/fleet/certificate_authorities
pub async fn list_certificate_authorities() -> FleetResponse {
    fleet_ok("certificate_authorities", serde_json::json!([]))
}

/// GET /api/_version_/fleet/certificate_authorities/{id}
pub async fn get_certificate_authority(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("certificate_authority", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/certificate_authorities/{id}
pub async fn delete_certificate_authority(Path(_id): Path<u64>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/certificate_authorities/{id}
pub async fn update_certificate_authority(
    Path(_id): Path<u64>,
    Json(_body): Json<UpdateCertificateAuthorityBody>,
) -> FleetResponse {
    fleet_ok("certificate_authority", serde_json::json!({}))
}

/// POST /api/_version_/fleet/certificate_authorities/{id}/request_certificate
pub async fn request_certificate(
    Path(_id): Path<u64>,
    Json(_body): Json<RequestCertificateBody>,
) -> FleetResponse {
    fleet_ok("certificate", serde_json::json!({}))
}

/// POST /api/_version_/fleet/spec/certificate_authorities
pub async fn batch_apply_certificate_authorities(
    Json(_body): Json<BatchApplyCertificateAuthoritiesBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/spec/certificate_authorities
pub async fn get_certificate_authorities_spec() -> FleetResponse {
    fleet_ok("specs", serde_json::json!([]))
}

/// POST /api/_version_/fleet/calendar/webhook/{event_uuid}
pub async fn calendar_webhook(
    Path(_event_uuid): Path<String>,
    Json(_body): Json<CalendarWebhookBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /metrics
pub async fn metrics() -> String {
    // TODO: collect and encode prometheus metrics
    String::new()
}
