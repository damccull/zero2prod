use anyhow::Context;
use axum::{extract::State, response::IntoResponse, Extension, Form};
use axum_extra::extract::WithRejection;
use axum_macros::debug_handler;
use http::StatusCode;
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    authentication::{get_username, UserId},
    domain::SubscriberEmail,
    email_client::EmailClient,
    routes::newsletters::BodyData,
};

use newsletter_errors::*;
use newsletter_types::*;

#[cfg_attr(any(test, debug_assertions), debug_handler(state = crate::startup::AppState ))]
#[tracing::instrument(
    name = "Publish a newsletter",
    skip(db_pool, email_client,  body),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    Extension(user_id): Extension<UserId>,
    State(db_pool): State<PgPool>,
    State(email_client): State<Arc<EmailClient>>,
    WithRejection(Form(body), _): WithRejection<Form<BodyData>, PublishError>,
) -> Result<impl IntoResponse, PublishError> {
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
    let username = get_username(*user_id, &db_pool).await;
    if let Ok(username) = username {
        tracing::Span::current().record("username", &tracing::field::display(username));
    }

    let subscribers = get_confirmed_subscribers(&db_pool).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(&subscriber.email, &body.title, &body.html, &body.text)
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
    //TODO: Redirect back to admin dashboard and flash successful message
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

    use crate::domain::SubscriberEmail;

    pub(crate) struct ConfirmedSubscriber {
        pub(crate) email: SubscriberEmail,
    }
}

mod newsletter_errors {
    use axum::{extract::rejection::FormRejection, response::IntoResponse};
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
        FormExtractionError(#[from] FormRejection),
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
                PublishError::FormExtractionError(_) => {
                    StatusCode::UNPROCESSABLE_ENTITY.into_response()
                }
            }
        }
    }
}
