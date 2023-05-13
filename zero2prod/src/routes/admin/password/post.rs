use axum::{
    response::{IntoResponse, Redirect},
    Form,
};
use axum_flash::Flash;
use axum_session::SessionRedisPool;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

use crate::{error::ResponseInternalServerError, session_state::TypedSession};

#[tracing::instrument(name = "Change password", skip(session, form))]
pub async fn change_password(
    flash: Flash,
    session: TypedSession<SessionRedisPool>,
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, ResponseInternalServerError<anyhow::Error>> {
    if session.get_user_id().is_none() {
        return Ok(Redirect::to("/login").into_response());
    }
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        let flash =
            flash.error("You entered two different new passwords - the field values must match.");
        return Ok((flash, Redirect::to("/admin/password")).into_response());
    }
    todo!()
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}
