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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn fleet_created(key: &str, value: Value) -> FleetResponse {
    fleet_response(StatusCode::CREATED, key, value)
}

/// Create a 204 No Content response (no body).
#[allow(dead_code)]
pub fn fleet_no_content() -> FleetResponse {
    (StatusCode::NO_CONTENT, Json(Value::Null))
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_fleet_ok_with_key() {
        let (status, Json(body)) = fleet_ok("user", json!({"id": 1}));
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, json!({"user": {"id": 1}}));
    }

    #[test]
    fn test_fleet_ok_empty_key() {
        let (status, Json(body)) = fleet_ok("", json!({"foo": "bar"}));
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, json!({"foo": "bar"}));
    }

    #[test]
    fn test_fleet_error() {
        let (status, Json(body)) = fleet_error(StatusCode::NOT_FOUND, "not found");
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body, json!({"error": "not found"}));
    }

    #[test]
    fn test_fleet_validation_error() {
        let errors = vec![ValidationError {
            name: "email".to_string(),
            reason: "invalid format".to_string(),
        }];
        let (status, Json(body)) = fleet_validation_error("validation failed", errors);
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["error"], "validation failed");
        let errors_arr = body["errors"].as_array().expect("errors should be an array");
        assert_eq!(errors_arr.len(), 1);
        assert_eq!(errors_arr[0]["name"], "email");
        assert_eq!(errors_arr[0]["reason"], "invalid format");
    }

    #[test]
    fn test_fleet_no_content() {
        let (status, _) = fleet_no_content();
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[test]
    fn test_encode_service_error_not_found() {
        let err = ServiceError::NotFound("resource missing".to_string());
        let (status, Json(body)) = encode_service_error(&err);
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body, json!({"error": "resource missing"}));
    }

    #[test]
    fn test_encode_service_error_forbidden() {
        let err = ServiceError::Forbidden("access denied".to_string());
        let (status, Json(body)) = encode_service_error(&err);
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body, json!({"error": "access denied"}));
    }

    #[test]
    fn test_encode_service_error_missing_license() {
        let err = ServiceError::MissingLicense;
        let (status, Json(body)) = encode_service_error(&err);
        assert_eq!(status, StatusCode::PAYMENT_REQUIRED);
        assert_eq!(body, json!({"error": "missing license"}));
    }
}
