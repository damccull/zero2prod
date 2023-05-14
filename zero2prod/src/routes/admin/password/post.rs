use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Form,
};
use axum_flash::Flash;
use axum_session::SessionRedisPool;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    authentication::{validate_credentials, AuthError, Credentials},
    e500,
    error::ResponseInternalServerError,
    routes::admin::dashboard::get_username,
    session_state::TypedSession,
};

#[tracing::instrument(name = "Change password", skip(session, form))]
pub async fn change_password(
    flash: Flash,
    session: TypedSession<SessionRedisPool>,
    State(pool): State<PgPool>,
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, ResponseInternalServerError<anyhow::Error>> {
    let user_id = session.get_user_id();
    // Ensure the user is logged in
    if user_id.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }

    let user_id = user_id.unwrap(); // Can't panic; we already know it's not `None`.

    // Ensure the new password and confirmation match
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        let flash =
            flash.error("You entered two different new passwords - the field values must match.");
        return Ok((flash, Redirect::to("/admin/password")).into_response());
    }

    // Ensure the old/current password is valid
    let username = get_username(user_id, &pool).await.map_err(e500)?;

    let credentials = Credentials {
        username,
        password: form.current_password,
    };

    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                let flash = flash.error("The current password is incorrect.");
                Ok((flash, Redirect::to("/admin/password")).into_response())
            }
            AuthError::UnexpectedError(_) => Err(e500(e.into())),
        };
    }

    todo!()
}

#[derive(Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}
