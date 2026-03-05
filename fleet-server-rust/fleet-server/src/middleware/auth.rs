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

/// Authenticated user extracted from the Authorization header.
///
/// The Go server uses `auth.SetRequestsContexts(svc)` to validate
/// the Bearer token and set the user in the request context.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: u64,
    pub email: String,
    pub name: String,
    pub global_role: Option<String>,
    pub session_id: u64,
}

impl<S: Send + Sync> FromRequestParts<S> for AuthenticatedUser {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
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

        // TODO: look up session by token in the datastore, validate it,
        // and return the associated user.
        Err((StatusCode::UNAUTHORIZED, "Authentication not yet implemented"))
    }
}

/// Authenticated host extracted from the osquery node_key.
///
/// Osquery endpoints pass the node_key in the JSON request body.
/// This extractor validates the node_key against the datastore.
#[derive(Debug, Clone)]
pub struct AuthenticatedHost {
    pub id: u64,
    pub uuid: String,
    pub node_key: String,
    pub team_id: Option<u64>,
    pub platform: String,
}

/// Authenticated device extracted from the device token in the URL path.
///
/// Fleet Desktop and the device API use a device token for authentication.
#[derive(Debug, Clone)]
pub struct AuthenticatedDevice {
    pub host_id: u64,
    pub token: String,
}

/// Authenticated Orbit agent extracted from the orbit_node_key.
///
/// Orbit endpoints pass the orbit_node_key in the JSON request body.
#[derive(Debug, Clone)]
pub struct AuthenticatedOrbit {
    pub host_id: u64,
    pub orbit_node_key: String,
}
