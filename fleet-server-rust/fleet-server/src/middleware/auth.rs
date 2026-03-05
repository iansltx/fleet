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
    pub id: u32,
    pub email: String,
    pub name: String,
    pub global_role: Option<String>,
    pub session_id: u32,
}

// To convert an AuthenticatedUser to a `fleet_service::Viewer` for use with
// the service layer, we need the full `User` and `Session` objects, which will
// be available once AppState is wired into the router and we perform the
// session/user lookups during extraction. At that point, add:
//
//   impl AuthenticatedUser {
//       pub fn to_viewer(user: fleet_types::User, session: fleet_types::Session) -> fleet_service::Viewer {
//           fleet_service::Viewer { user, session }
//       }
//   }

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
        let _token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid authorization format"))?;

        if _token.is_empty() {
            return Err((StatusCode::UNAUTHORIZED, "Empty authorization token"));
        }

        // To implement this properly, we need AppState in the router.
        // The flow will be:
        // 1. Extract Bearer token (done above)
        // 2. Call state.service.get_session_by_key(token) to get Session
        // 3. Call state.service.get_user_unauthorized(session.user_id) to get User
        // 4. Return AuthenticatedUser from user + session data
        //
        // For now, return an error until AppState is wired in.
        Err((StatusCode::UNAUTHORIZED, "Authentication not yet implemented"))
    }
}

/// Authenticated host extracted from the osquery node_key.
///
/// Osquery endpoints pass the node_key in the JSON request body.
/// This extractor validates the node_key against the datastore.
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
#[derive(Debug, Clone)]
pub struct AuthenticatedDevice {
    pub host_id: u32,
    pub token: String,
}

/// Authenticated Orbit agent extracted from the orbit_node_key.
///
/// Orbit endpoints pass the orbit_node_key in the JSON request body.
#[derive(Debug, Clone)]
pub struct AuthenticatedOrbit {
    pub host_id: u32,
    pub orbit_node_key: String,
}
