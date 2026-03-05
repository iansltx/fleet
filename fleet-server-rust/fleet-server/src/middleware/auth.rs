//! Authentication extraction middleware.
//!
//! Provides extractors for different authentication schemes used by Fleet:
//! - User authentication (Bearer token in Authorization header)
//! - Host authentication (osquery node_key in JSON body)
//! - Device authentication (device token in URL path)
//! - Orbit authentication (orbit_node_key in JSON body)

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

use crate::AppState;

/// Authenticated user extracted from the Authorization header.
///
/// The Go server uses `auth.SetRequestsContexts(svc)` to validate
/// the Bearer token and set the user in the request context.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: u32,
    pub email: String,
    pub name: String,
    pub global_role: Option<String>,
    pub session_id: u32,
}

impl AuthenticatedUser {
    /// Helper to create a Viewer by fetching user and session from the service.
    /// Call this in handlers that need a Viewer.
    pub async fn viewer(&self, state: &AppState) -> Result<fleet_service::Viewer, (StatusCode, &'static str)> {
        let user = state.service.get_user_unauthorized(self.id).await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "User not found"))?;
        let session = state.service.datastore().session_by_id(self.session_id).await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Session not found"))?;
        Ok(fleet_service::Viewer { user, session })
    }
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization header"))?;

        // Expect "Bearer <token>"
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid authorization format"))?;

        if token.is_empty() {
            return Err((StatusCode::UNAUTHORIZED, "Empty authorization token"));
        }

        // Look up session by token
        let session = state.service.get_session_by_key(token).await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired session"))?;

        // Look up user
        let user = state.service.get_user_unauthorized(session.user_id).await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "User not found"))?;

        Ok(AuthenticatedUser {
            id: user.id,
            email: user.email.clone(),
            name: user.name.clone(),
            global_role: user.global_role.clone(),
            session_id: session.id,
        })
    }
}

/// Authenticated host extracted from the osquery node_key.
///
/// Osquery endpoints pass the node_key in the JSON request body.
/// This extractor validates the node_key against the datastore.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuthenticatedHost {
    pub id: u32,
    pub uuid: String,
    pub node_key: String,
    pub team_id: Option<u32>,
    pub platform: String,
}

/// Authenticated device extracted from the device token in the URL path.
///
/// Fleet Desktop and the device API use a device token for authentication.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuthenticatedDevice {
    pub host_id: u32,
    pub token: String,
}

/// Authenticated Orbit agent extracted from the orbit_node_key.
///
/// Orbit endpoints pass the orbit_node_key in the JSON request body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuthenticatedOrbit {
    pub host_id: u32,
    pub orbit_node_key: String,
}
