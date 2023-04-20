use axum::{extract::Query, response::IntoResponse};
use http::StatusCode;
use serde::Deserialize;

#[tracing::instrument(name = "Confirm a pending subscription", skip(_parameters))]
pub async fn confirm(_parameters: Query<ConfirmParameters>) -> impl IntoResponse {
    StatusCode::OK
}

#[derive(Debug, Deserialize)]
pub struct ConfirmParameters {
    subscription_token: String,
}
