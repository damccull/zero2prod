use std::sync::Arc;

use anyhow::Context;
use axum::{extract::State, response::IntoResponse, Json};
use axum_macros::debug_handler;
use http::StatusCode;
use sqlx::PgPool;

use crate::{domain::SubscriberEmail, email_client::EmailClient, error_chain_fmt};

#[tracing::instrument(name = "Publish a newsletter", skip(db_pool, email_client, body))]
#[cfg_attr(any(test, debug_assertions), debug_handler(state = crate::startup::AppState ))]
pub async fn publish_newsletter(
    State(db_pool): State<PgPool>,
    State(email_client): State<Arc<EmailClient>>,
    Json(body): Json<BodyData>,
) -> Result<impl IntoResponse, PublishError> {
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

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[derive(Debug, serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(Debug, serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for PublishError {
    fn into_response(self) -> axum::response::Response {
        match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}
