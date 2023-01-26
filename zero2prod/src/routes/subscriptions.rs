use axum::{http::StatusCode, response::IntoResponse, Form};
use serde::Deserialize;

pub async fn subscribe(Form(form): Form<FormData>) -> impl IntoResponse {
    StatusCode::OK
}
#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}
