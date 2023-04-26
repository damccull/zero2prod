use axum::response::IntoResponse;
use axum_extra::response::Html;
use http::StatusCode;

pub async fn login_form() -> impl IntoResponse {
    Html((StatusCode::OK, include_str!("login.html")))
}
