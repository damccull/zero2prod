use axum::{http::StatusCode, response::IntoResponse, Form};
use serde::Deserialize;

pub async fn subscribe(Form(_form): Form<FormData>) -> impl IntoResponse {
    StatusCode::OK
}
#[derive(Deserialize)]
#[allow(dead_code)]
pub struct FormData {
    email: String,
    name: String,
}
