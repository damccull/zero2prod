use std::{future::Future, net::TcpListener};

use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};

pub fn run(listener: TcpListener) -> impl Future<Output = hyper::Result<()>> {
    // Create a router that will contain and match all routes for the application
    let app = Router::new().route("/health_check", get(health_check));

    // Start the axum server and set up to use supplied listener
    axum::Server::from_tcp(listener)
        .expect("failed to create server from listener")
        .serve(app.into_make_service())
}

/// Returns HTTP status code OK (200) to act as a health check
pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
