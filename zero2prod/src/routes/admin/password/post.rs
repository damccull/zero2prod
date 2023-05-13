use axum::{
    response::{IntoResponse, Redirect},
    Form,
};
use axum_session::SessionRedisPool;
use http::StatusCode;
use secrecy::Secret;
use serde::Deserialize;

use crate::{error::ResponseInternalServerError, session_state::TypedSession};

#[tracing::instrument(name = "Change password", skip(session, form))]
pub async fn change_password(
    session: TypedSession<SessionRedisPool>,
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, ResponseInternalServerError<anyhow::Error>> {
    if session.get_user_id().is_none() {
        return Ok(Redirect::to("/login").into_response());
    }
    Ok(StatusCode::OK.into_response())
}

#[derive(Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}
