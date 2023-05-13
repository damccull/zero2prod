use axum::{response::IntoResponse, Form};
use http::StatusCode;
use secrecy::Secret;
use serde::Deserialize;

use crate::error::ResponseInternalServerError;

#[tracing::instrument(name = "Change password", skip(form))]
pub async fn change_password(
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, ResponseInternalServerError<anyhow::Error>> {
    todo!();
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}
