//! Authentication helpers.
//!
//! Provides password hashing, JWT token creation/validation,
//! session management, and random token generation.
//! Corresponds to Go's auth-related code in `server/service/sessions.go`
//! and `server/service/middleware/auth/auth.go`.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{ServiceError, ServiceResult, Viewer};

// ---------------------------------------------------------------------------
// Password hashing (bcrypt)
// ---------------------------------------------------------------------------

/// Hashes a password using bcrypt with the given cost.
pub fn hash_password(password: &str, cost: u32) -> ServiceResult<Vec<u8>> {
    let hash = bcrypt::hash(password, cost).map_err(|e| {
        ServiceError::internal(format!("failed to hash password: {}", e))
    })?;
    Ok(hash.into_bytes())
}

/// Verifies a password against a bcrypt hash.
pub fn verify_password(password: &str, hash: &[u8]) -> ServiceResult<bool> {
    let hash_str = std::str::from_utf8(hash).map_err(|e| {
        ServiceError::internal(format!("invalid password hash encoding: {}", e))
    })?;
    bcrypt::verify(password, hash_str).map_err(|e| {
        ServiceError::internal(format!("failed to verify password: {}", e))
    })
}

// ---------------------------------------------------------------------------
// JWT tokens
// ---------------------------------------------------------------------------

/// Claims embedded in a Fleet JWT.
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user ID as string).
    pub sub: String,
    /// Issued at (Unix timestamp).
    pub iat: i64,
    /// Expiration (Unix timestamp).
    pub exp: i64,
    /// Session ID.
    pub session_id: u32,
}

/// Creates a new JWT token for the given user and session.
pub fn create_jwt_token(
    user_id: u32,
    session_id: u32,
    jwt_key: &str,
    duration: std::time::Duration,
) -> ServiceResult<String> {
    let now = Utc::now();
    let exp = now + Duration::from_std(duration).unwrap_or(Duration::hours(4));

    let claims = JwtClaims {
        sub: user_id.to_string(),
        iat: now.timestamp(),
        exp: exp.timestamp(),
        session_id,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_key.as_bytes()),
    )
    .map_err(|e| ServiceError::internal(format!("failed to create JWT: {}", e)))?;

    Ok(token)
}

/// Validates a JWT token and returns the claims.
pub fn validate_jwt_token(token: &str, jwt_key: &str) -> ServiceResult<JwtClaims> {
    let validation = Validation::default();

    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(jwt_key.as_bytes()),
        &validation,
    )
    .map_err(|e| ServiceError::auth_failed(format!("invalid JWT: {}", e)))?;

    Ok(token_data.claims)
}

// ---------------------------------------------------------------------------
// Random token generation
// ---------------------------------------------------------------------------

/// Generates a cryptographically random text string of the given byte length,
/// encoded as URL-safe base64.
pub fn generate_random_text(size: usize) -> ServiceResult<String> {
    use base64::Engine;
    use rand::RngCore;

    let mut bytes = vec![0u8; size];
    rand::thread_rng().fill_bytes(&mut bytes);
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes))
}

// ---------------------------------------------------------------------------
// Viewer authentication (from session key)
// ---------------------------------------------------------------------------

/// Creates an authenticated Viewer by validating a session key.
///
/// Corresponds to Go's `auth.AuthViewer`.
/// This is used in middleware to establish the viewer context.
pub async fn auth_viewer(
    svc: &crate::FleetService,
    session_key: &str,
) -> ServiceResult<Viewer> {
    let session = svc.get_session_by_key(session_key).await?;
    let user = svc.get_user_unauthorized(session.user_id).await?;

    // Check if the user needs a forced password reset.
    if user.admin_forced_password_reset {
        return Err(ServiceError::PasswordResetRequired);
    }

    Ok(Viewer { user, session })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "test_password_123!";
        let hash = hash_password(password, bcrypt::DEFAULT_COST).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_jwt_create_and_validate() {
        let key = "test-secret-key";
        let token = create_jwt_token(42, 100, key, std::time::Duration::from_secs(3600)).unwrap();
        let claims = validate_jwt_token(&token, key).unwrap();

        assert_eq!(claims.sub, "42");
        assert_eq!(claims.session_id, 100);
    }

    #[test]
    fn test_jwt_invalid_key() {
        let token = create_jwt_token(1, 1, "key1", std::time::Duration::from_secs(3600)).unwrap();
        assert!(validate_jwt_token(&token, "key2").is_err());
    }

    #[test]
    fn test_generate_random_text() {
        let t1 = generate_random_text(24).unwrap();
        let t2 = generate_random_text(24).unwrap();
        assert_ne!(t1, t2);
        assert!(!t1.is_empty());
    }
}
