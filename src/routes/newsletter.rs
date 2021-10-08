use std::any;

use actix_http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use serde::Deserialize;
use sqlx::PgPool;

use crate::{domain::SubscriberEmail, email_client::EmailClient, routes::error_chain_fmt};

#[tracing::instrument(name = "Send a newsletter", skip(body))]
#[allow(clippy::async_yields_async)]
pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, PublishError> {
    let subscribers = get_confirmed_subscribers(&pool).await?;

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
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })?;
            }
            Err(error) => {
                tracing::warn!(
                    // Record the error chain as a structured field on the log record
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid."
                )
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    // This struct is only needed here, so it's a child of this function
    struct Row {
        email: String,
    }

    let rows = sqlx::query_as!(
        Row,
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed';
    "#
    )
    .fetch_all(pool)
    .await?;

    // Map into the domain type
    let confirmed_subscribers = rows
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        .collect();
    Ok(confirmed_subscribers)
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn status_code(&self) -> StatusCode {
        match self {
            PublishError::UnknownError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
