use anyhow::Context;
use axum::{
    extract::{rejection::FormRejection, State},
    response::{IntoResponse, Redirect},
    Extension, Form,
};
use axum_flash::Flash;
use axum_macros::debug_handler;
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    authentication::{get_username, UserId},
    domain::SubscriberEmail,
    e400, e500,
    email_client::EmailClient,
    error::ResponseError,
    idempotency::{save_response, try_processing, IdempotencyKey, NextAction},
};

use newsletter_types::*;

static PUBLISH_SUCCESS_INFO_MESSAGE: &str = "The newsletter issue has been published";

#[cfg_attr(any(test, debug_assertions), debug_handler(state = crate::startup::AppState ))]
#[tracing::instrument(
    name = "Publish a newsletter",
    skip(flash, db_pool, email_client,  body),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    flash: Flash,
    Extension(user_id): Extension<UserId>,
    State(db_pool): State<PgPool>,
    State(email_client): State<Arc<EmailClient>>,
    body: Result<Form<FormData>, FormRejection>,
) -> Result<impl IntoResponse, ResponseError> {
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
    let username = get_username(*user_id, &db_pool).await;
    if let Ok(username) = username {
        tracing::Span::current().record("username", &tracing::field::display(username));
    }

    let body = if let Ok(body) = body {
        tracing::trace!("Successfully extracted form body");
        body
    } else {
        tracing::trace!("Unable to extract form body: {:?}", body);
        let flash = flash.error("Part of the form is not filled out");

        return Ok((flash, Redirect::to("/admin/newsletters")).into_response());
    };

    let FormData {
        title,
        text_content,
        html_content,
        idempotency_key,
    } = body.0;

    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(e400)?;
    // Concurrent idempotency requests wait for first to finish and then
    // Return early if we have a cached response
    let transaction = match try_processing(&db_pool, &idempotency_key, *user_id)
        .await
        .map_err(e500)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            let flash = flash.info(PUBLISH_SUCCESS_INFO_MESSAGE);
            return Ok((flash, saved_response).into_response());
        }
    };

    // Continue to make full email request if we did not have a cached response
    let subscribers = get_confirmed_subscribers(&db_pool).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(&subscriber.email, &title, &text_content, &html_content)
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
    let flash = flash.info(PUBLISH_SUCCESS_INFO_MESSAGE);
    let response = (flash, Redirect::to("/admin/newsletters")).into_response();
    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(e500)?;
    Ok(response)
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

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct FormData {
        pub title: String,
        pub html_content: String,
        pub text_content: String,
        pub idempotency_key: String,
    }
}

mod newsletter_errors {
    use axum::{extract::rejection::FormRejection, response::IntoResponse};
    use http::StatusCode;

    use crate::error_chain_fmt;

    #[allow(clippy::enum_variant_names)]
    #[derive(thiserror::Error)]
    pub enum PublishError {
        // #[error("Authentication failed")]
        // AuthError(#[source] anyhow::Error),
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
                // PublishError::AuthError(_) => {
                //     let mut response = StatusCode::UNAUTHORIZED.into_response();
                //     let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
                //     response
                //         .headers_mut()
                //         .insert(header::WWW_AUTHENTICATE, header_value);
                //     response
                // }
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
