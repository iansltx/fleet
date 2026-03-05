//! Session and authentication endpoints.
//!
//! Handles login, logout, SSO, password reset, and session management.

use axum::extract::{Json, Path, State};
use serde::Deserialize;

use crate::middleware::auth::AuthenticatedUser;
use crate::response::{fleet_error, fleet_ok, encode_service_error, FleetResponse};
use crate::AppState;

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
pub async fn me(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.authenticated_user(&viewer).await {
        Ok(user) => fleet_ok("user", serde_json::to_value(&user).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// GET /api/_version_/fleet/sessions/{id}
pub async fn get_session_info(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.get_info_about_session(&viewer, id as u32).await {
        Ok(session) => fleet_ok("session", serde_json::to_value(&session).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}

/// DELETE /api/_version_/fleet/sessions/{id}
pub async fn delete_session(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<u64>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.delete_session(&viewer, id as u32).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/login
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginBody>,
) -> FleetResponse {
    match state.service.login(&body.email, &body.password).await {
        Ok((user, session)) => {
            let mut user_json = serde_json::to_value(&user).unwrap_or_default();
            if let Some(obj) = user_json.as_object_mut() {
                obj.insert("token".to_string(), serde_json::Value::String(session.key));
            }
            fleet_ok("user", user_json)
        }
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/sessions (MFA-aware login)
pub async fn session_create(
    State(state): State<AppState>,
    Json(body): Json<SessionCreateBody>,
) -> FleetResponse {
    match state.service.login(&body.email, &body.password).await {
        Ok((user, session)) => {
            let mut user_json = serde_json::to_value(&user).unwrap_or_default();
            if let Some(obj) = user_json.as_object_mut() {
                obj.insert("token".to_string(), serde_json::Value::String(session.key));
            }
            fleet_ok("user", user_json)
        }
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/logout
pub async fn logout(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    match state.service.logout(&viewer).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/forgot_password
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(body): Json<ForgotPasswordBody>,
) -> FleetResponse {
    // Always return success to avoid leaking whether the email exists.
    match state.service.request_password_reset(&body.email).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => {
            tracing::warn!("password reset request failed: {}", e);
            fleet_ok("", serde_json::json!({}))
        }
    }
}

/// POST /api/_version_/fleet/reset_password
pub async fn reset_password(
    State(state): State<AppState>,
    Json(body): Json<ResetPasswordBody>,
) -> FleetResponse {
    match state.service.reset_password(&body.password_reset_token, &body.new_password).await {
        Ok(()) => fleet_ok("", serde_json::json!({})),
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/_version_/fleet/perform_required_password_reset
pub async fn perform_required_password_reset(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<PerformRequiredPasswordResetBody>,
) -> FleetResponse {
    let viewer = match auth.viewer(&state).await {
        Ok(v) => v,
        Err(e) => return fleet_error(e.0, e.1),
    };
    // Use empty string for old_password since this is a forced reset.
    match state.service.change_password(&viewer, "", &body.new_password).await {
        Ok(()) => {
            match state.service.get_user(&viewer, viewer.user_id()).await {
                Ok(user) => fleet_ok("user", serde_json::to_value(&user).unwrap_or_default()),
                Err(e) => encode_service_error(&e),
            }
        }
        Err(e) => encode_service_error(&e),
    }
}

/// POST /api/v1/fleet/sso
pub async fn initiate_sso(
    State(_state): State<AppState>,
    Json(_body): Json<InitiateSSOBody>,
) -> FleetResponse {
    // TODO: add initiate_sso to FleetService
    fleet_ok("url", serde_json::json!(""))
}

/// POST /api/v1/fleet/sso/callback
pub async fn callback_sso(
    State(_state): State<AppState>,
    Json(_body): Json<CallbackSSOBody>,
) -> FleetResponse {
    // TODO: add callback_sso to FleetService
    fleet_ok("user", serde_json::json!({"token": ""}))
}

/// GET /api/v1/fleet/sso
pub async fn settings_sso(
    State(state): State<AppState>,
) -> FleetResponse {
    match state.service.sso_settings().await {
        Ok(settings) => fleet_ok("settings", serde_json::to_value(&settings).unwrap_or_default()),
        Err(e) => encode_service_error(&e),
    }
}
