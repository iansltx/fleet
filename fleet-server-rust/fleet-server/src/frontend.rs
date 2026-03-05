//! Frontend asset serving.
//!
//! Uses `rust-embed` to embed the React frontend assets at compile time
//! and serves them via axum. This replaces the Go server's `ServeHTTP`
//! handler that serves the Fleet UI.
//!
//! The frontend is a single-page React application. All non-API routes
//! that don't match a static file should return `index.html` so that
//! client-side routing works correctly.

use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use rust_embed::Embed;

/// Embedded frontend assets.
///
/// At compile time, `rust-embed` includes all files from the frontend
/// build directory. In development this directory may be empty.
#[derive(Embed)]
#[folder = "../../frontend/build/"]
#[prefix = ""]
struct FrontendAssets;

/// Build the router for serving frontend assets.
///
/// This serves:
/// - Static files (JS, CSS, images) from the embedded assets
/// - `index.html` for all other paths (SPA fallback)
pub fn frontend_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        // Serve specific static asset paths
        .route("/assets/{*path}", get(serve_asset))
        .route("/favicon.ico", get(serve_favicon))
        // SPA fallback: all other non-API paths serve index.html
        .fallback(get(serve_index))
}

/// Serve a static asset file by path.
async fn serve_asset(Path(path): Path<String>) -> Response {
    let asset_path = format!("assets/{}", path);
    serve_embedded_file(&asset_path)
}

/// Serve the favicon.
async fn serve_favicon() -> Response {
    serve_embedded_file("favicon.ico")
}

/// Serve index.html for SPA routing.
///
/// Any path that doesn't match an API route or a static asset
/// returns the React app's index.html, allowing client-side
/// routing to handle the path.
async fn serve_index() -> Response {
    serve_embedded_file("index.html")
}

/// Look up and serve an embedded file.
fn serve_embedded_file(path: &str) -> Response {
    match FrontendAssets::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            let mut response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref());

            // Set cache headers for assets (long cache for hashed filenames)
            if path.starts_with("assets/") {
                response = response.header(
                    header::CACHE_CONTROL,
                    "public, max-age=31536000, immutable",
                );
            } else {
                // Short cache for index.html and other root files
                response = response.header(
                    header::CACHE_CONTROL,
                    "no-cache",
                );
            }

            response.body(axum::body::Body::from(file.data.to_vec()))
                .unwrap_or_else(|_| {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
                        .into_response()
                })
        }
        None => {
            // If the file doesn't exist and it looks like a frontend route,
            // serve index.html for SPA routing
            if !path.contains('.') || path == "index.html" {
                match FrontendAssets::get("index.html") {
                    Some(index) => {
                        Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                            .header(header::CACHE_CONTROL, "no-cache")
                            .body(axum::body::Body::from(index.data.to_vec()))
                            .unwrap_or_else(|_| {
                                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
                                    .into_response()
                            })
                    }
                    None => {
                        (StatusCode::NOT_FOUND, "Frontend not available").into_response()
                    }
                }
            } else {
                (StatusCode::NOT_FOUND, "Not Found").into_response()
            }
        }
    }
}
