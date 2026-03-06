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
    pub list: Vec<fleet_service::translate::TranslatePayload>,
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
    State(state): State<AppState>,
    Json(body): Json<TriggerBody>,
) -> FleetResponse {
    let _ = &state;
    // Log the trigger request; actual job execution requires background worker infrastructure
    tracing::info!(trigger_name = ?body.name, "trigger requested (background workers not yet implemented)");
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/config/certificate
///
/// Returns the PEM-encoded TLS certificate chain for the configured server URL.
/// Connects to the server's own TLS endpoint to retrieve the peer certificate.
/// If TLS is not configured or the connection fails, returns an empty string.
pub async fn get_certificate(State(state): State<AppState>) -> FleetResponse {
    let server_url = &state.service.config().server.server_url;
    match fetch_certificate_chain(server_url).await {
        Ok(chain) => fleet_ok("certificate_chain", serde_json::json!(chain)),
        Err(e) => {
            tracing::warn!(error = %e, server_url = %server_url, "failed to fetch TLS certificate chain");
            fleet_ok("certificate_chain", serde_json::json!(""))
        }
    }
}

/// Connects to the given server URL via TLS and returns the PEM-encoded
/// peer certificate.
///
/// Note: The Go implementation returns the full chain minus the leaf certificate
/// (since osqueryd obtains the leaf during its own TLS handshake). The `native-tls`
/// crate only exposes the peer (leaf) certificate. If a full chain minus the leaf
/// is needed in the future, consider switching to the `openssl` or `rustls` crate
/// which expose `peer_cert_chain()`.
async fn fetch_certificate_chain(server_url: &str) -> Result<String, anyhow::Error> {
    use base64::Engine;

    let parsed = url::Url::parse(server_url)?;
    if parsed.scheme() == "http" {
        // No TLS configured
        return Ok(String::new());
    }
    let hostname = parsed.host_str().unwrap_or("localhost").to_string();
    let port = parsed.port().unwrap_or(443);
    let hostport = format!("{}:{}", hostname, port);

    // Try secure first, then fall back to accepting invalid certs (self-signed)
    let der_bytes = match connect_tls_get_cert(&hostport, &hostname, false).await {
        Ok(Some(der)) => der,
        Ok(None) => return Ok(String::new()),
        Err(_) => match connect_tls_get_cert(&hostport, &hostname, true).await? {
            Some(der) => der,
            None => return Ok(String::new()),
        },
    };

    // Encode as PEM
    let b64 = base64::engine::general_purpose::STANDARD.encode(&der_bytes);
    let mut pem = String::from("-----BEGIN CERTIFICATE-----\n");
    for chunk in b64.as_bytes().chunks(64) {
        pem.push_str(std::str::from_utf8(chunk).unwrap_or(""));
        pem.push('\n');
    }
    pem.push_str("-----END CERTIFICATE-----\n");
    Ok(pem)
}

/// Establish a TLS connection and return the peer certificate DER bytes.
async fn connect_tls_get_cert(
    hostport: &str,
    hostname: &str,
    accept_invalid: bool,
) -> Result<Option<Vec<u8>>, anyhow::Error> {
    let tcp = tokio::net::TcpStream::connect(hostport).await?;

    let mut builder = native_tls::TlsConnector::builder();
    if accept_invalid {
        builder.danger_accept_invalid_certs(true);
        builder.danger_accept_invalid_hostnames(true);
    }
    let connector = builder.build()?;
    let connector = tokio_native_tls::TlsConnector::from(connector);
    let tls_stream = connector.connect(hostname, tcp).await?;

    // native_tls exposes only the peer certificate (leaf)
    let native_stream = tls_stream.get_ref();
    match native_stream.peer_certificate()? {
        Some(cert) => Ok(Some(cert.to_der()?)),
        None => Ok(None),
    }
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
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<ApplyEnrollSecretSpecBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Parse the spec body into a list of secrets.
    let secrets: Vec<fleet_types::enroll::EnrollSecret> = match body.spec.get("secrets") {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect(),
        _ => Vec::new(),
    };
    match state.service.apply_enroll_secrets(&viewer, secrets).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/spec/enroll_secret
pub async fn get_enroll_secret_spec(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_enroll_secrets(&viewer).await {
        Ok(secrets) => fleet_ok(
            "spec",
            serde_json::json!({ "secrets": secrets }),
        ),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/version
pub async fn version(State(state): State<AppState>) -> FleetResponse {
    let _ = &state;
    fleet_ok(
        "version",
        serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
        }),
    )
}

/// POST /api/_version_/fleet/translate
pub async fn translate(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<TranslateBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.translate_identifiers(&viewer, body.list).await {
        Ok(list) => fleet_ok("list", serde_json::to_value(&list).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/certificates
pub async fn create_certificate_template(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<CreateCertificateTemplateBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let name = body.name.as_deref().unwrap_or("");
    let team_id = body.rest.get("team_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let ca_id = body.rest.get("certificate_authority_id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let subject_name = body.rest.get("subject_name").and_then(|v| v.as_str()).unwrap_or("");
    match state.service.create_certificate_template(&viewer, team_id, ca_id, name, subject_name).await {
        Ok(tmpl) => fleet_ok("certificate_template", serde_json::to_value(&tmpl).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/certificates
pub async fn list_certificate_templates(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.list_certificate_templates(&viewer).await {
        Ok(templates) => fleet_ok("certificate_templates", serde_json::to_value(&templates).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/certificates/{id}
pub async fn get_certificate_template(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_certificate_template(&viewer, id as u32).await {
        Ok(tmpl) => fleet_ok("certificate_template", serde_json::to_value(&tmpl).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/certificates/{id}
pub async fn delete_certificate_template(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_certificate_template(&viewer, id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/spec/certificates
pub async fn apply_certificate_template_specs(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<ApplyCertificateTemplateSpecsBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Apply certificate template specs: create each template from spec
    for spec in &body.specs {
        let team_id = spec.get("team_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let ca_id = spec.get("certificate_authority_id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let name = spec.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let subject_name = spec.get("subject_name").and_then(|v| v.as_str()).unwrap_or("");
        if let Err(e) = state.service.create_certificate_template(&viewer, team_id, ca_id, name, subject_name).await {
            return encode_service_error(&e);
        }
    }
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/spec/certificates
pub async fn delete_certificate_template_specs(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Delete all certificate templates
    match state.service.list_certificate_templates(&viewer).await {
        Ok(templates) => {
            for t in &templates {
                if let Err(e) = state.service.delete_certificate_template(&viewer, t.id).await {
                    return encode_service_error(&e);
                }
            }
            fleet_ok("", serde_json::json!({}))
        }
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/status/result_store
pub async fn status_result_store(State(state): State<AppState>) -> FleetResponse {
    // Check Redis connectivity for query result store
    match state.live_query.load_active_query_names().await {
        Ok(_) => fleet_ok("status", serde_json::json!("ok")),
        Err(e) => fleet_ok("status", serde_json::json!({"status": "error", "error": e.to_string()})),
    }
}

/// GET /api/_version_/fleet/status/live_query
pub async fn status_live_query(State(state): State<AppState>) -> FleetResponse {
    // Check Redis connectivity for live query store
    match state.live_query.load_active_query_names().await {
        Ok(_) => fleet_ok("status", serde_json::json!("ok")),
        Err(e) => fleet_ok("status", serde_json::json!({"status": "error", "error": e.to_string()})),
    }
}

/// PUT /api/_version_/fleet/spec/secret_variables
pub async fn create_secret_variables(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<CreateSecretVariablesBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Parse secrets as (name, value) pairs
    let secrets: Vec<(String, String)> = body.secrets.iter().filter_map(|v| {
        let name = v.get("name")?.as_str()?.to_string();
        let value = v.get("value")?.as_str()?.to_string();
        Some((name, value))
    }).collect();
    match state.service.upsert_secret_variables(&viewer, &secrets).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/custom_variables
pub async fn create_secret_variable(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<CreateSecretVariableBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let name = body.name.unwrap_or_default();
    let value = body.value.unwrap_or_default();
    match state.service.create_secret_variable(&viewer, &name, &value).await {
        Ok(sv) => fleet_ok("secret_variable", serde_json::to_value(&sv).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/custom_variables
pub async fn list_secret_variables(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListSecretVariablesParams>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.list_secret_variables(&viewer).await {
        Ok(vars) => {
            let page = params.page.unwrap_or(0) as usize;
            let per_page = params.per_page.unwrap_or(20) as usize;
            let start = page * per_page;
            let end = (start + per_page).min(vars.len());
            let paginated = if start < vars.len() { &vars[start..end] } else { &[] as &[_] };
            fleet_ok("", serde_json::json!({
                "secret_variables": paginated,
                "count": vars.len(),
            }))
        }
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/custom_variables/{id}
pub async fn delete_secret_variable(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_secret_variable(&viewer, id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/scim/details
pub async fn get_scim_details(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// POST /api/_version_/fleet/conditional-access/microsoft
pub async fn conditional_access_microsoft_create(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<ConditionalAccessMicrosoftCreateBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &body);
    // Stub: backing service not yet implemented
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/conditional-access/microsoft/confirm
pub async fn conditional_access_microsoft_confirm(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<ConditionalAccessMicrosoftConfirmBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, &body);
    // Stub: backing service not yet implemented
    fleet_ok("", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/conditional-access/microsoft
pub async fn conditional_access_microsoft_delete(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    // Stub: backing service not yet implemented
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/conditional_access/idp/signing_cert
pub async fn conditional_access_get_idp_signing_cert(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    // Stub: backing service not yet implemented
    fleet_ok("signing_cert", serde_json::json!(""))
}

/// GET /api/_version_/fleet/conditional_access/idp/apple/profile
pub async fn conditional_access_get_idp_apple_profile(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = &viewer;
    // Stub: backing service not yet implemented
    fleet_ok("profile", serde_json::json!({}))
}

/// POST /api/_version_/fleet/certificate_authorities
pub async fn create_certificate_authority(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<CreateCertificateAuthorityBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let ca_type = body.data.get("type").and_then(|v| v.as_str()).unwrap_or("custom_scep_proxy");
    let name = body.data.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let url = body.data.get("url").and_then(|v| v.as_str()).unwrap_or("");
    match state.service.create_certificate_authority(&viewer, ca_type, name, url).await {
        Ok(ca) => fleet_ok("certificate_authority", serde_json::to_value(&ca).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/certificate_authorities
pub async fn list_certificate_authorities(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.list_certificate_authorities(&viewer).await {
        Ok(cas) => fleet_ok("certificate_authorities", serde_json::to_value(&cas).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/certificate_authorities/{id}
pub async fn get_certificate_authority(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_certificate_authority(&viewer, id as i32).await {
        Ok(ca) => fleet_ok("certificate_authority", serde_json::to_value(&ca).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/certificate_authorities/{id}
pub async fn delete_certificate_authority(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_certificate_authority(&viewer, id as i32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// PATCH /api/_version_/fleet/certificate_authorities/{id}
pub async fn update_certificate_authority(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<UpdateCertificateAuthorityBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let name = body.data.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let url = body.data.get("url").and_then(|v| v.as_str()).unwrap_or("");
    match state.service.update_certificate_authority(&viewer, id as i32, name, url).await {
        Ok(ca) => fleet_ok("certificate_authority", serde_json::to_value(&ca).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/certificate_authorities/{id}/request_certificate
pub async fn request_certificate(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
    Json(body): Json<RequestCertificateBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    let _ = (&viewer, id, &body);
    encode_service_error(&fleet_service::ServiceError::MissingLicense)
}

/// POST /api/_version_/fleet/spec/certificate_authorities
pub async fn batch_apply_certificate_authorities(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<BatchApplyCertificateAuthoritiesBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Batch apply creates/updates CAs from specs
    for spec in &body.specs {
        let ca_type = spec.get("type").and_then(|v| v.as_str()).unwrap_or("custom_scep_proxy");
        let name = spec.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let url = spec.get("url").and_then(|v| v.as_str()).unwrap_or("");
        if let Err(e) = state.service.create_certificate_authority(&viewer, ca_type, name, url).await {
            return encode_service_error(&e);
        }
    }
    fleet_ok("", serde_json::json!({}))
}

/// GET /api/_version_/fleet/spec/certificate_authorities
pub async fn get_certificate_authorities_spec(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.list_certificate_authorities(&viewer).await {
        Ok(cas) => fleet_ok("specs", serde_json::to_value(&cas).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/calendar/webhook/{event_uuid}
pub async fn calendar_webhook(
    State(state): State<AppState>,
    Path(event_uuid): Path<String>,
    Json(body): Json<CalendarWebhookBody>,
) -> FleetResponse {
    let _ = (&state, &body);
    // Log the calendar webhook; full implementation requires calendar integration
    tracing::info!(event_uuid = %event_uuid, "calendar webhook received (calendar integration not yet implemented)");
    fleet_ok("", serde_json::json!({}))
}

/// GET /metrics
pub async fn metrics() -> String {
    // Basic process metrics; full Prometheus integration deferred
    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("# HELP process_start_time_seconds Start time of the process.\n# TYPE process_start_time_seconds gauge\nprocess_start_time_seconds {}\n", uptime)
}
