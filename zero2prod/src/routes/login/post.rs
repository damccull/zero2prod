use axum::response::IntoResponse;
use http::StatusCode;

#[tracing::instrument(name = "Login posted")]
pub async fn login() -> impl IntoResponse {
    StatusCode::OK
}
