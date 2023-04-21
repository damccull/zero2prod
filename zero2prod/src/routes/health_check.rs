use axum::{http::StatusCode, response::IntoResponse};

/// Returns HTTP status code OK (200) to act as a health check
#[tracing::instrument(name = "[Health Check]")]
pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
