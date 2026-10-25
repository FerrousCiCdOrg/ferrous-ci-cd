//! REST API implementation

use crate::application::Application;
use axum::{routing::get, Router};

/// Create the API server
pub async fn create_server(_app: Application) -> crate::Result<Router> {
    // Create router
    let router = Router::new().route("/health", get(health_check));

    Ok(router)
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}
