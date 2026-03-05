//! Session and authentication endpoints.
//!
//! Handles login, logout, SSO, password reset, and session management.

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
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SessionCreateBody {
    pub email: String,
    pub password: String,
    pub mfa_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordBody {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordBody {
    pub new_password: String,
    pub password_reset_token: String,
}

#[derive(Debug, Deserialize)]
pub struct PerformRequiredPasswordResetBody {
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct InitiateSSOBody {
    pub relay_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CallbackSSOBody {
    #[serde(rename = "SAMLResponse")]
    pub saml_response: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/_version_/fleet/me
pub async fn me() -> FleetResponse {
    // TODO: extract AppState and auth from request context
    // let viewer = auth.to_viewer();
    // match state.service.authenticated_user(&viewer).await {
    //     Ok(user) => fleet_ok("user", serde_json::to_value(&user).unwrap()),
    //     Err(e) => encode_service_error(&e),
    // }
    fleet_ok("user", serde_json::json!({}))
}

/// GET /api/_version_/fleet/sessions/{id}
pub async fn get_session_info(Path(_id): Path<u64>) -> FleetResponse {
    // TODO: extract AppState and auth from request context
    // let viewer = auth.to_viewer();
    // match state.service.get_info_about_session(&viewer, id as u32).await {
    //     Ok(session) => fleet_ok("session", serde_json::to_value(&session).unwrap()),
    //     Err(e) => encode_service_error(&e),
    // }
    fleet_ok("session", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/sessions/{id}
pub async fn delete_session(Path(_id): Path<u64>) -> FleetResponse {
    // TODO: extract AppState and auth from request context
    // let viewer = auth.to_viewer();
    // match state.service.delete_session(&viewer, id as u32).await {
    //     Ok(()) => fleet_ok("", serde_json::json!({})),
    //     Err(e) => encode_service_error(&e),
    // }
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/login
pub async fn login(Json(_body): Json<LoginBody>) -> FleetResponse {
    // TODO: extract AppState from request context (no auth required)
    // match state.service.login(&body.email, &body.password).await {
    //     Ok((user, session)) => {
    //         let mut user_json = serde_json::to_value(&user).unwrap();
    //         if let Some(obj) = user_json.as_object_mut() {
    //             obj.insert("token".to_string(), serde_json::Value::String(session.key));
    //         }
    //         fleet_ok("user", user_json)
    //     }
    //     Err(e) => encode_service_error(&e),
    // }
    fleet_ok("user", serde_json::json!({"token": ""}))
}

/// POST /api/_version_/fleet/sessions (MFA-aware login)
pub async fn session_create(Json(_body): Json<SessionCreateBody>) -> FleetResponse {
    // TODO: extract AppState from request context (no auth required)
    // match state.service.login(&body.email, &body.password).await {
    //     Ok((user, session)) => {
    //         let mut user_json = serde_json::to_value(&user).unwrap();
    //         if let Some(obj) = user_json.as_object_mut() {
    //             obj.insert("token".to_string(), serde_json::Value::String(session.key));
    //         }
    //         fleet_ok("user", user_json)
    //     }
    //     Err(e) => encode_service_error(&e),
    // }
    fleet_ok("user", serde_json::json!({"token": ""}))
}

/// POST /api/_version_/fleet/logout
pub async fn logout() -> FleetResponse {
    // TODO: extract AppState and auth from request context
    // let viewer = auth.to_viewer();
    // match state.service.logout(&viewer).await {
    //     Ok(()) => fleet_ok("", serde_json::json!({})),
    //     Err(e) => encode_service_error(&e),
    // }
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/forgot_password
pub async fn forgot_password(Json(_body): Json<ForgotPasswordBody>) -> FleetResponse {
    // TODO: extract AppState from request context (no auth required)
    // Always return success to avoid leaking whether the email exists.
    // match state.service.request_password_reset(&body.email).await {
    //     Ok(()) => fleet_ok("", serde_json::json!({})),
    //     Err(e) => {
    //         // Log but don't expose the error to the client.
    //         tracing::warn!("password reset request failed: {}", e);
    //         fleet_ok("", serde_json::json!({}))
    //     }
    // }
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/reset_password
pub async fn reset_password(Json(_body): Json<ResetPasswordBody>) -> FleetResponse {
    // TODO: extract AppState from request context (no auth required)
    // match state.service.reset_password(&body.password_reset_token, &body.new_password).await {
    //     Ok(()) => fleet_ok("", serde_json::json!({})),
    //     Err(e) => encode_service_error(&e),
    // }
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/perform_required_password_reset
pub async fn perform_required_password_reset(
    Json(_body): Json<PerformRequiredPasswordResetBody>,
) -> FleetResponse {
    // TODO: extract AppState and auth from request context
    // let viewer = auth.to_viewer();
    // let user = state.service.authenticated_user(&viewer).await?;
    // match state.service.change_password(&viewer, "", &body.new_password).await {
    //     Ok(()) => {
    //         let user = state.service.get_user(&viewer, viewer.user_id()).await.unwrap();
    //         fleet_ok("user", serde_json::to_value(&user).unwrap())
    //     }
    //     Err(e) => encode_service_error(&e),
    // }
    fleet_ok("user", serde_json::json!({}))
}

/// POST /api/v1/fleet/sso
pub async fn initiate_sso(Json(_body): Json<InitiateSSOBody>) -> FleetResponse {
    // TODO: extract AppState from request context (no auth required)
    // TODO: add initiate_sso to FleetService
    // match state.service.initiate_sso(body.relay_url.as_deref()).await {
    //     Ok(url) => fleet_ok("url", serde_json::json!(url)),
    //     Err(e) => encode_service_error(&e),
    // }
    fleet_ok("url", serde_json::json!(""))
}

/// POST /api/v1/fleet/sso/callback
pub async fn callback_sso(Json(_body): Json<CallbackSSOBody>) -> FleetResponse {
    // TODO: extract AppState from request context (no auth required)
    // TODO: add callback_sso to FleetService
    // match state.service.callback_sso(body.saml_response.as_deref().unwrap_or("")).await {
    //     Ok((user, session)) => {
    //         let mut user_json = serde_json::to_value(&user).unwrap();
    //         if let Some(obj) = user_json.as_object_mut() {
    //             obj.insert("token".to_string(), serde_json::Value::String(session.key));
    //         }
    //         fleet_ok("user", user_json)
    //     }
    //     Err(e) => encode_service_error(&e),
    // }
    fleet_ok("user", serde_json::json!({"token": ""}))
}

/// GET /api/v1/fleet/sso
pub async fn settings_sso() -> FleetResponse {
    // TODO: extract AppState from request context (no auth required)
    // TODO: add get_sso_settings to FleetService
    // match state.service.get_sso_settings().await {
    //     Ok(settings) => fleet_ok("settings", serde_json::to_value(&settings).unwrap()),
    //     Err(e) => encode_service_error(&e),
    // }
    fleet_ok("settings", serde_json::json!({}))
}
