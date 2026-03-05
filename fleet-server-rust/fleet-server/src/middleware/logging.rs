//! Request logging middleware.
//!
//! Logs incoming requests and outgoing responses with timing information,
//! matching the Go server's `log.LogRequestEnd` behavior.

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

/// Middleware function that logs request start and completion.
///
/// Logs method, path, status code, and duration. In the Go server this
/// is implemented as a go-kit `ServerAfter` hook via `log.LogRequestEnd`.
pub async fn log_request(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    tracing::info!(
        method = %method,
        path = %uri.path(),
        "request started"
    );

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    if status.is_server_error() {
        tracing::error!(
            method = %method,
            path = %uri.path(),
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "request completed with server error"
        );
    } else if status.is_client_error() {
        tracing::warn!(
            method = %method,
            path = %uri.path(),
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "request completed with client error"
        );
    } else {
        tracing::info!(
            method = %method,
            path = %uri.path(),
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "request completed"
        );
    }

    response
}
