//! Standard JSON response helpers for the Fleet API.
//!
//! Matches the Go server's response format:
//! - `{"key": value}` for single items (e.g., `{"user": {...}}`)
//! - `{"key": [...]}` for lists (e.g., `{"users": [...]}`)
//! - `{"error": "message"}` for errors
//! - `{"error": "message", "errors": [...]}` for validation errors
//!
//! The Go server uses `encodeResponse` which wraps the response in a JSON
//! object with the endpoint-specific key.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use fleet_service::ServiceError;
use serde_json::Value;

/// The standard Fleet API response type.
///
/// All Fleet API handlers should return this type.
pub type FleetResponse = (StatusCode, Json<Value>);

/// Create a successful response with a named key wrapping the value.
///
/// If `key` is empty, the value is returned as-is (for endpoints that
/// return a flat JSON object).
///
/// # Examples
///
/// ```
/// // Returns: {"user": {"id": 1, "name": "admin"}}
/// fleet_ok("user", json!({"id": 1, "name": "admin"}));
///
/// // Returns: {"users": []}
/// fleet_ok("users", json!([]));
///
/// // Returns: {} (flat response)
/// fleet_ok("", json!({}));
/// ```
pub fn fleet_ok(key: &str, value: Value) -> FleetResponse {
    if key.is_empty() {
        (StatusCode::OK, Json(value))
    } else {
        let mut response = serde_json::Map::new();
        response.insert(key.to_string(), value);
        (StatusCode::OK, Json(Value::Object(response)))
    }
}

/// Create an error response.
///
/// Returns: `{"error": "message"}`
///
/// The status code determines the HTTP status. Common codes:
/// - 400 Bad Request
/// - 401 Unauthorized
/// - 403 Forbidden
/// - 404 Not Found
/// - 409 Conflict
/// - 422 Unprocessable Entity
/// - 500 Internal Server Error
pub fn fleet_error(status: StatusCode, message: &str) -> FleetResponse {
    let body = serde_json::json!({
        "error": message,
    });
    (status, Json(body))
}

/// Create an error response with additional validation errors.
///
/// Returns: `{"error": "message", "errors": [{"name": "field", "reason": "..."}]}`
pub fn fleet_validation_error(message: &str, errors: Vec<ValidationError>) -> FleetResponse {
    let body = serde_json::json!({
        "error": message,
        "errors": errors,
    });
    (StatusCode::UNPROCESSABLE_ENTITY, Json(body))
}

/// A single field validation error.
#[derive(Debug, serde::Serialize)]
pub struct ValidationError {
    pub name: String,
    pub reason: String,
}

/// Create a response with a specific status code and a named key.
pub fn fleet_response(status: StatusCode, key: &str, value: Value) -> FleetResponse {
    if key.is_empty() {
        (status, Json(value))
    } else {
        let mut response = serde_json::Map::new();
        response.insert(key.to_string(), value);
        (status, Json(Value::Object(response)))
    }
}

/// Create a 201 Created response.
pub fn fleet_created(key: &str, value: Value) -> FleetResponse {
    fleet_response(StatusCode::CREATED, key, value)
}

/// Create a 204 No Content response (no body).
pub fn fleet_no_content() -> FleetResponse {
    (StatusCode::NO_CONTENT, Json(Value::Null))
}

/// Convert a service-layer error into a Fleet API error response.
///
/// This maps internal error types to appropriate HTTP status codes,
/// matching the Go server's `fleetErrorEncoder` behavior.
pub fn encode_error(err: &dyn std::error::Error) -> FleetResponse {
    // TODO: match on specific error types from fleet-service:
    // - NotFoundError -> 404
    // - AuthRequiredError -> 401
    // - ForbiddenError -> 403
    // - ConflictError -> 409
    // - ValidationError -> 422
    // - etc.
    fleet_error(StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
}

/// Convert a `ServiceError` into a Fleet API error response.
///
/// This maps each variant to the appropriate HTTP status code,
/// matching the Go server's `fleetErrorEncoder` behavior.
pub fn encode_service_error(err: &ServiceError) -> FleetResponse {
    match err {
        ServiceError::NotFound(msg) => fleet_error(StatusCode::NOT_FOUND, msg),
        ServiceError::Unauthorized(msg) => fleet_error(StatusCode::UNAUTHORIZED, msg),
        ServiceError::Forbidden(msg) => fleet_error(StatusCode::FORBIDDEN, msg),
        ServiceError::BadRequest(msg) => fleet_error(StatusCode::BAD_REQUEST, msg),
        ServiceError::Conflict(msg) => fleet_error(StatusCode::CONFLICT, msg),
        ServiceError::MissingLicense => {
            fleet_error(StatusCode::PAYMENT_REQUIRED, "missing license")
        }
        ServiceError::AuthFailed(msg) => fleet_error(StatusCode::UNAUTHORIZED, msg),
        ServiceError::AuthRequired(msg) => fleet_error(StatusCode::UNAUTHORIZED, msg),
        ServiceError::PasswordResetRequired => {
            fleet_error(StatusCode::FORBIDDEN, "password reset required")
        }
        ServiceError::InvalidArgument { field, message } => fleet_validation_error(
            &format!("validation failed: {}: {}", field, message),
            vec![ValidationError {
                name: field.clone(),
                reason: message.clone(),
            }],
        ),
        ServiceError::Internal(msg) => fleet_error(StatusCode::INTERNAL_SERVER_ERROR, msg),
        ServiceError::Anyhow(err) => {
            fleet_error(StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
        }
    }
}
