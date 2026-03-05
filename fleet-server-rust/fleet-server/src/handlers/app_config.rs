//! App configuration, status, and miscellaneous endpoints.
//!
//! Handles app config get/modify, enroll secrets, version, certificates,
//! secret variables, SCIM, conditional access, and more.

use axum::extract::{Json, Path, Query, State};
use serde::Deserialize;

use crate::middleware::auth::AuthenticatedUser;
use crate::response::{encode_service_error, fleet_error, fleet_ok, FleetResponse};
use crate::AppState;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ModifyAppConfigBody {
    pub org_name: Option<String>,
    pub org_logo_url: Option<String>,
    pub server_url: Option<String>,
    pub live_query_disabled: Option<bool>,
    pub enable_sso: Option<bool>,
    pub sso_entity_id: Option<String>,
    pub sso_idp_name: Option<String>,
    pub sso_metadata: Option<String>,
    pub sso_metadata_url: Option<String>,
    pub smtp_configured: Option<bool>,
    pub smtp_sender_address: Option<String>,
    pub smtp_server: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_enable_ssl_tls: Option<bool>,
    pub smtp_user_name: Option<String>,
    pub smtp_password: Option<String>,
    pub host_expiry_enabled: Option<bool>,
    pub host_expiry_window: Option<i64>,
    pub agent_options: Option<serde_json::Value>,
    pub transparency_url: Option<String>,
    pub enable_host_users: Option<bool>,
    pub enable_software_inventory: Option<bool>,
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
pub async fn trigger(
    State(_state): State<AppState>,
    Json(_body): Json<TriggerBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/config/certificate
pub async fn get_certificate(State(_state): State<AppState>) -> FleetResponse {
    fleet_ok("certificate_chain", serde_json::json!(""))
}

/// GET /api/_version_/fleet/config
pub async fn get_app_config(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.app_config_obfuscated(&viewer).await {
        Ok(config) => fleet_ok("", serde_json::to_value(&config).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/config
pub async fn modify_app_config(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<ModifyAppConfigBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let payload = fleet_service::app_config::ModifyAppConfigPayload {
        org_name: body.org_name,
        org_logo_url: body.org_logo_url,
        server_url: body.server_url,
        live_query_disabled: body.live_query_disabled,
        enable_sso: body.enable_sso,
        sso_entity_id: body.sso_entity_id,
        sso_idp_name: body.sso_idp_name,
        sso_metadata: body.sso_metadata,
        sso_metadata_url: body.sso_metadata_url,
        smtp_configured: body.smtp_configured,
        smtp_sender_address: body.smtp_sender_address,
        smtp_server: body.smtp_server,
        smtp_port: body.smtp_port,
        smtp_enable_ssl_tls: body.smtp_enable_ssl_tls,
        smtp_user_name: body.smtp_user_name,
        smtp_password: body.smtp_password,
        host_expiry_enabled: body.host_expiry_enabled,
        host_expiry_window: body.host_expiry_window,
        agent_options: body.agent_options,
        transparency_url: body.transparency_url,
        enable_host_users: body.enable_host_users,
        enable_software_inventory: body.enable_software_inventory,
    };
    match state.service.modify_app_config(&viewer, payload).await {
        Ok(config) => fleet_ok("", serde_json::to_value(&config).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/spec/enroll_secret
pub async fn apply_enroll_secret_spec(
    State(_state): State<AppState>,
    Json(_body): Json<ApplyEnrollSecretSpecBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/spec/enroll_secret
pub async fn get_enroll_secret_spec(State(_state): State<AppState>) -> FleetResponse {
    fleet_ok("spec", serde_json::json!({}))
}

/// GET /api/_version_/fleet/version
pub async fn version(State(_state): State<AppState>) -> FleetResponse {
    fleet_ok(
        "version",
        serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
        }),
    )
}

/// POST /api/_version_/fleet/translate
pub async fn translate(
    State(_state): State<AppState>,
    Json(_body): Json<TranslateBody>,
) -> FleetResponse {
    fleet_ok("list", serde_json::json!([]))
}

/// POST /api/_version_/fleet/certificates
pub async fn create_certificate_template(
    State(_state): State<AppState>,
    Json(_body): Json<CreateCertificateTemplateBody>,
) -> FleetResponse {
    fleet_ok("certificate_template", serde_json::json!({}))
}

/// GET /api/_version_/fleet/certificates
pub async fn list_certificate_templates(State(_state): State<AppState>) -> FleetResponse {
    fleet_ok("certificate_templates", serde_json::json!([]))
}

/// GET /api/_version_/fleet/certificates/{id}
pub async fn get_certificate_template(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("certificate_template", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/certificates/{id}
pub async fn delete_certificate_template(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/spec/certificates
pub async fn apply_certificate_template_specs(
    State(_state): State<AppState>,
    Json(_body): Json<ApplyCertificateTemplateSpecsBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/spec/certificates
pub async fn delete_certificate_template_specs(State(_state): State<AppState>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/status/result_store
pub async fn status_result_store(State(_state): State<AppState>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/status/live_query
pub async fn status_live_query(State(_state): State<AppState>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// PUT /api/_version_/fleet/spec/secret_variables
pub async fn create_secret_variables(
    State(_state): State<AppState>,
    Json(_body): Json<CreateSecretVariablesBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/custom_variables
pub async fn create_secret_variable(
    State(_state): State<AppState>,
    Json(_body): Json<CreateSecretVariableBody>,
) -> FleetResponse {
    fleet_ok("secret_variable", serde_json::json!({}))
}

/// GET /api/_version_/fleet/custom_variables
pub async fn list_secret_variables(
    State(_state): State<AppState>,
    Query(_params): Query<ListSecretVariablesParams>,
) -> FleetResponse {
    fleet_ok("secret_variables", serde_json::json!([]))
}

/// DELETE /api/_version_/fleet/custom_variables/{id}
pub async fn delete_secret_variable(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/scim/details
pub async fn get_scim_details(State(_state): State<AppState>) -> FleetResponse {
    fleet_ok("scim", serde_json::json!({}))
}

/// POST /api/_version_/fleet/conditional-access/microsoft
pub async fn conditional_access_microsoft_create(
    State(_state): State<AppState>,
    Json(_body): Json<ConditionalAccessMicrosoftCreateBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/conditional-access/microsoft/confirm
pub async fn conditional_access_microsoft_confirm(
    State(_state): State<AppState>,
    Json(_body): Json<ConditionalAccessMicrosoftConfirmBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/conditional-access/microsoft
pub async fn conditional_access_microsoft_delete(State(_state): State<AppState>) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/conditional_access/idp/signing_cert
pub async fn conditional_access_get_idp_signing_cert(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("signing_cert", serde_json::json!(""))
}

/// GET /api/_version_/fleet/conditional_access/idp/apple/profile
pub async fn conditional_access_get_idp_apple_profile(
    State(_state): State<AppState>,
) -> FleetResponse {
    fleet_ok("profile", serde_json::json!({}))
}

/// POST /api/_version_/fleet/certificate_authorities
pub async fn create_certificate_authority(
    State(_state): State<AppState>,
    Json(_body): Json<CreateCertificateAuthorityBody>,
) -> FleetResponse {
    fleet_ok("certificate_authority", serde_json::json!({}))
}

/// GET /api/_version_/fleet/certificate_authorities
pub async fn list_certificate_authorities(State(_state): State<AppState>) -> FleetResponse {
    fleet_ok("certificate_authorities", serde_json::json!([]))
}

/// GET /api/_version_/fleet/certificate_authorities/{id}
pub async fn get_certificate_authority(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("certificate_authority", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/certificate_authorities/{id}
pub async fn delete_certificate_authority(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// PATCH /api/_version_/fleet/certificate_authorities/{id}
pub async fn update_certificate_authority(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
    Json(_body): Json<UpdateCertificateAuthorityBody>,
) -> FleetResponse {
    fleet_ok("certificate_authority", serde_json::json!({}))
}

/// POST /api/_version_/fleet/certificate_authorities/{id}/request_certificate
pub async fn request_certificate(
    State(_state): State<AppState>,
    Path(_id): Path<u64>,
    Json(_body): Json<RequestCertificateBody>,
) -> FleetResponse {
    fleet_ok("certificate", serde_json::json!({}))
}

/// POST /api/_version_/fleet/spec/certificate_authorities
pub async fn batch_apply_certificate_authorities(
    State(_state): State<AppState>,
    Json(_body): Json<BatchApplyCertificateAuthoritiesBody>,
) -> FleetResponse {
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/spec/certificate_authorities
pub async fn get_certificate_authorities_spec(State(_state): State<AppState>) -> FleetResponse {
    fleet_ok("specs", serde_json::json!([]))
}

/// POST /api/_version_/fleet/calendar/webhook/{event_uuid}
pub async fn calendar_webhook(
    State(_state): State<AppState>,
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
