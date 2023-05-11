use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Form,
};

use axum_flash::Flash;
use axum_macros::debug_handler;
use axum_session::SessionRedisPool;
use http::StatusCode;
use secrecy::Secret;
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    authentication::{validate_credentials, AuthError, Credentials},
    error_chain_fmt,
    session_state::TypedSession,
};

#[debug_handler(state = crate::startup::AppState)]
#[tracing::instrument(
    name = "Login posted"
    skip(form, flash, session, pool),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    State(pool): State<PgPool>,
    flash: Flash,
    session: TypedSession<SessionRedisPool>,
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, LoginError> {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };

    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    let response = match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            // In actix_web, it would be necessary to handle serialization failure here. Somehow axum gets around that.
            session.renew();
            session.insert_user_id(user_id);
            Redirect::to("/admin/dashboard").into_response()
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };
            tracing::error!("{:?}", &e);

            let flash = flash.error(e.to_string());

            let response = Redirect::to("/login").into_response();

            (flash, response).into_response()
        }
    };

    Ok(response)
}

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
impl IntoResponse for LoginError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:?}", self);
        match self {
            LoginError::AuthError(_) => StatusCode::UNAUTHORIZED.into_response(),
            LoginError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
