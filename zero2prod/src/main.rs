use std::net::SocketAddr;

use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() {
    // Set up tracing, see the method definition
    setup_tracing();

    // Create a router that will contain and match all routes for the application
    let app = Router::new().route("/health_check", get(health_check));

    // Listen on localhost on port 8000
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    tracing::debug!("listening on {}", addr);

    // Start the axum server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// Returns HTTP status code OK (200) to act as a health check
async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

/// Sets up a tracing subscriber.
fn setup_tracing() {
    let fmt_layer = fmt::layer().compact().with_target(true);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let subscriber = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer);

    let _ = tracing::subscriber::set_global_default(subscriber)
        .map_err(|_err| eprintln!("Unable to set global default subscriber"));
}
