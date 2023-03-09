use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use serde::Deserialize;

pub async fn subscribe(State(db): State<PgPool>, Form(form): Form<FormData>) -> impl IntoResponse {
}
#[derive(Deserialize)]
#[allow(dead_code)]
pub struct FormData {
    email: String,
    name: String,
}
