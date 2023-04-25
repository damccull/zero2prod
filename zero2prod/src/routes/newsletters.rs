use std::sync::Arc;

use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{extract::State, response::IntoResponse, Json};
use axum_extra::extract::WithRejection;
use axum_macros::debug_handler;
use base64::Engine;
use http::{HeaderMap, StatusCode};
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use crate::{domain::SubscriberEmail, email_client::EmailClient};

use newsletter_errors::*;
use newsletter_types::*;

#[cfg_attr(any(test, debug_assertions), debug_handler(state = crate::startup::AppState ))]
#[tracing::instrument(
    name = "Publish a newsletter",
    skip(db_pool, email_client, headers, body),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    State(db_pool): State<PgPool>,
    State(email_client): State<Arc<EmailClient>>,
    headers: HeaderMap,
    WithRejection(Json(body), _): WithRejection<Json<BodyData>, PublishError>,
) -> Result<impl IntoResponse, PublishError> {
    // Check if the user is authorized to use this endpoint
    let credentials = basic_authentication(headers).map_err(PublishError::AuthError)?;
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, &db_pool).await?;
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

    let subscribers = get_confirmed_subscribers(&db_pool).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &body.title,
                        &body.content.html,
                        &body.content.text,
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {},", subscriber.email)
                    })?;
            }
            Err(e) => {
                tracing::warn!(
                    error.cause_chain = ?e,
                    "Skipping a confirmed subscriber.\
                    Their stored contact details are invalid.");
            }
        }
    }
    Ok(StatusCode::OK)
}

async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool,
) -> Result<uuid::Uuid, PublishError> {
    let row: Option<_> = sqlx::query!(
        r#"
        SELECT user_id, password_hash
        FROM users
        WHERE username = $1
        "#,
        credentials.username,
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform a query to validate auth credentials.")
    .map_err(PublishError::UnexpectedError)?;

    let (expected_password_hash, user_id) = match row {
        Some(row) => (row.password_hash, row.user_id),
        None => {
            return Err(PublishError::AuthError(anyhow::anyhow!(
                "Unknown username."
            )))
        }
    };

    let expected_password_hash = PasswordHash::new(&expected_password_hash)
        .context("Failed to parse hash in PHC string format.")
        .map_err(PublishError::UnexpectedError)?;

    Argon2::default()
        .verify_password(
            credentials.password.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password.")
        .map_err(PublishError::AuthError)?;

    Ok(user_id)
}

fn basic_authentication(headers: HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' headers was missing.")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF-8 string.")?;

    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'.")?;

    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials.")?;

    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF-8.")?;

    // Split decoded string into two segments delimited by ':'.
    let mut credentials = decoded_credentials.splitn(2, ':');

    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided with 'Basic' auth."))?
        .to_string();

    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth."))?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(db_pool))]
async fn get_confirmed_subscribers(
    db_pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(db_pool)
    .await?;

    // Map the fetched rows to domain types
    let confirmed_subscribers = rows
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(e) => Err(anyhow::anyhow!(e)),
        })
        .collect();
    Ok(confirmed_subscribers)
}

mod newsletter_types {
    use secrecy::Secret;

    use crate::domain::SubscriberEmail;

    pub(crate) struct ConfirmedSubscriber {
        pub(crate) email: SubscriberEmail,
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct BodyData {
        pub title: String,
        pub content: Content,
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct Content {
        pub html: String,
        pub text: String,
    }

    #[derive(Debug)]
    pub(crate) struct Credentials {
        pub(crate) username: String,
        pub(crate) password: Secret<String>,
    }
}

mod newsletter_errors {
    use axum::{extract::rejection::JsonRejection, response::IntoResponse};
    use http::{header, HeaderValue, StatusCode};

    use crate::error_chain_fmt;

    #[allow(clippy::enum_variant_names)]
    #[derive(thiserror::Error)]
    pub enum PublishError {
        #[error("Authentication failed")]
        AuthError(#[source] anyhow::Error),
        #[error(transparent)]
        UnexpectedError(#[from] anyhow::Error),
        #[error(transparent)]
        JsonExtractionError(#[from] JsonRejection),
    }

    impl std::fmt::Debug for PublishError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            error_chain_fmt(self, f)
        }
    }

    impl IntoResponse for PublishError {
        fn into_response(self) -> axum::response::Response {
            tracing::error!("{:?}", self);
            match self {
                PublishError::AuthError(_) => {
                    let mut response = StatusCode::UNAUTHORIZED.into_response();
                    let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
                    response
                        .headers_mut()
                        .insert(header::WWW_AUTHENTICATE, header_value);
                    response
                }
                PublishError::UnexpectedError(_) => {
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
                PublishError::JsonExtractionError(_) => {
                    StatusCode::UNPROCESSABLE_ENTITY.into_response()
                }
            }
        }
    }
}
