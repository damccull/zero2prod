use axum::{response::IntoResponse, Json};
use http::StatusCode;

pub async fn publish_newsletter(Json(body_data): Json<BodyData>) -> impl IntoResponse {
    StatusCode::OK
}

#[derive(Debug, serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(Debug, serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}
