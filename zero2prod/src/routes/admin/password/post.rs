use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Extension, Form,
};
use axum_flash::Flash;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    authentication::{get_username, validate_credentials, AuthError, Credentials, UserId},
    e500,
    error::ResponseError,
};

#[tracing::instrument(name = "Change password", skip(user_id, form))]
pub async fn change_password(
    flash: Flash,
    Extension(user_id): Extension<UserId>,
    State(pool): State<PgPool>,
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, ResponseError> {
    // Ensure the new password is the correct length
    if form.new_password.expose_secret().len() < 12 || form.new_password.expose_secret().len() > 128
    {
        let flash = flash.error("The new password should be between 12 and 128 characters long.");
        return Ok((flash, Redirect::to("/admin/password")).into_response());
    }

    // Ensure the new password and confirmation match
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        let flash =
            flash.error("You entered two different new passwords - the field values must match.");
        return Ok((flash, Redirect::to("/admin/password")).into_response());
    }

    // Ensure the old/current password is valid
    let username = get_username(*user_id, &pool).await.map_err(e500)?;

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
            AuthError::UnexpectedError(_) => Err(e500(e)),
        };
    }

    crate::authentication::change_password(*user_id, form.new_password, &pool)
        .await
        .map_err(e500)?;

    let flash = flash.error("Your password has been changed.");

    Ok((flash, Redirect::to("/admin/password")).into_response())
}

#[derive(Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}
