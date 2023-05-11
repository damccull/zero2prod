use axum::response::IntoResponse;
use http::StatusCode;

pub async fn admin_dashboard() -> impl IntoResponse {
    StatusCode::OK
}
