use std::{future::Future, net::SocketAddr};

use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};

pub fn run() -> impl Future<Output = Result<(), hyper::Error>> {
    // Create a router that will contain and match all routes for the application
    let app = Router::new().route("/health_check", get(health_check));

    // Listen on localhost on port 8000
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    tracing::debug!("listening on {}", addr);

    // Start the axum server
    axum::Server::bind(&addr).serve(app.into_make_service())
}

/// Returns HTTP status code OK (200) to act as a health check
pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
