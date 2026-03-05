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
    // TODO: extract authenticated user from request context
    fleet_ok("user", serde_json::json!({}))
}

/// GET /api/_version_/fleet/sessions/{id}
pub async fn get_session_info(Path(_id): Path<u64>) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("session", serde_json::json!({}))
}

/// DELETE /api/_version_/fleet/sessions/{id}
pub async fn delete_session(Path(_id): Path<u64>) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/login
pub async fn login(Json(_body): Json<LoginBody>) -> FleetResponse {
    // TODO: call service layer - authenticate user, create session, return token
    fleet_ok("user", serde_json::json!({"token": ""}))
}

/// POST /api/_version_/fleet/sessions (MFA-aware login)
pub async fn session_create(Json(_body): Json<SessionCreateBody>) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("user", serde_json::json!({"token": ""}))
}

/// POST /api/_version_/fleet/logout
pub async fn logout() -> FleetResponse {
    // TODO: call service layer
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/forgot_password
pub async fn forgot_password(Json(_body): Json<ForgotPasswordBody>) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/reset_password
pub async fn reset_password(Json(_body): Json<ResetPasswordBody>) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("", serde_json::json!({}))
}

/// POST /api/_version_/fleet/perform_required_password_reset
pub async fn perform_required_password_reset(
    Json(_body): Json<PerformRequiredPasswordResetBody>,
) -> FleetResponse {
    // TODO: call service layer
    fleet_ok("user", serde_json::json!({}))
}

/// POST /api/v1/fleet/sso
pub async fn initiate_sso(Json(_body): Json<InitiateSSOBody>) -> FleetResponse {
    // TODO: call service layer - return SSO redirect URL
    fleet_ok("url", serde_json::json!(""))
}

/// POST /api/v1/fleet/sso/callback
pub async fn callback_sso(Json(_body): Json<CallbackSSOBody>) -> FleetResponse {
    // TODO: call service layer - validate SAML response, create session
    fleet_ok("user", serde_json::json!({"token": ""}))
}

/// GET /api/v1/fleet/sso
pub async fn settings_sso() -> FleetResponse {
    // TODO: call service layer - return SSO settings for the login page
    fleet_ok("settings", serde_json::json!({}))
}
