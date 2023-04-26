use axum::response::IntoResponse;
use axum_extra::response::Html;
use http::StatusCode;

#[tracing::instrument(name = "Home page")]
pub async fn home() -> impl IntoResponse {
    let body = include_str!("home/home.html");
    Html((StatusCode::OK, body))
}
