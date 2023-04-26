use axum::{
    response::{IntoResponse, Redirect},
    Form,
};
use axum_macros::debug_handler;
use http::StatusCode;
use secrecy::Secret;
use serde::Deserialize;

use crate::error_chain_fmt;

#[debug_handler]
#[tracing::instrument(
    name = "Login posted"
    skip(form_data)
)]
pub async fn login(Form(form_data): Form<FormData>) -> Result<impl IntoResponse, LoginErrorTemp> {
    Ok(Redirect::to("/").into_response())
}

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[derive(thiserror::Error)]
#[error(transparent)]
pub struct LoginErrorTemp(#[from] anyhow::Error);
impl IntoResponse for LoginErrorTemp {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "It broke").into_response()
    }
}

impl std::fmt::Debug for LoginErrorTemp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
