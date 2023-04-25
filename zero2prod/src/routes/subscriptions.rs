use std::{fmt::Display, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{rejection::FormRejection, State},
    http::StatusCode,
    response::{IntoResponse, Result},
    Form,
};
use axum_extra::extract::WithRejection;
use axum_macros::debug_handler;
use chrono::Utc;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Deserialize;
use sqlx::{types::Uuid, PgPool, Postgres, Transaction};

use crate::{
    domain::NewSubscriber,
    email_client::EmailClient,
    startup::{AppState, ApplicationBaseUrl},
};

#[tracing::instrument(
    name="[Adding a new subscriber]",
    skip(db, email_client, base_url, form),
    fields(
        subscriber_email=%form.email,
        subscriber_name=%form.name
    )
)]
#[cfg_attr(any(test, debug_assertions), debug_handler(state = AppState ))]
pub async fn subscribe(
    State(db): State<PgPool>,
    State(email_client): State<Arc<EmailClient>>,
    State(base_url): State<ApplicationBaseUrl>,
    WithRejection(Form(form), _): WithRejection<Form<FormData>, SubscribeError>,
) -> Result<impl IntoResponse, SubscribeError> {
    tracing::info!(
        "Adding '{}' '{}' as a new subscriber.",
        form.email,
        form.name
    );

    let mut transaction = db
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool.")?;

    let new_subscriber = form.try_into().map_err(SubscribeError::ValidationError)?;

    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber)
        .await
        .context("Failed to insert a new subscriber in the database.")?;

    let subscription_token = generate_subscription_token();

    store_token(&mut transaction, subscriber_id, &subscription_token)
        .await
        .context("Failed to store the confirmation token for a new subscriber.")?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber.")?;

    send_confirmation_email(
        email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await
    .context("Failed to send a confirmation email.")?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(transaction, subscription_token)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id) VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    )
    .execute(transaction)
    .await?;
    Ok(())
}

#[tracing::instrument(
    name="Send confirmation email"
    skip(email_client, new_subscriber, base_url)
)]
async fn send_confirmation_email(
    email_client: Arc<EmailClient>,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    // Build a confirmation link with a dynamic root
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
                Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    let plain_body = format!(
        "Welcome to our newsletter!<br />\
                Visit {} to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(
            &new_subscriber.clone().email,
            "Welcome!",
            &html_body,
            &plain_body,
        )
        .await
}

#[tracing::instrument(
    name = "[Saving new subscriber details in the database]",
    skip(transaction, new_subscriber)
)]
async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'pending_confirmation')
    "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(transaction)
    .await?;
    Ok(subscriber_id)
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[derive(Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

pub struct StoreTokenError(sqlx::Error);

impl Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while \
            trying to store a subscription token."
        )
    }
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::error_chain_fmt(self, f)
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    // Transparent delegates both `Display`'s and `source`'s implementation
    // to the type wrapped by `UnexpectedError`.
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error(transparent)]
    FormExtractionError(#[from] FormRejection),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::error_chain_fmt(self, f)
    }
}

impl IntoResponse for SubscribeError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:?}", self);
        match self {
            SubscribeError::ValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            SubscribeError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SubscribeError::FormExtractionError(_) => StatusCode::UNPROCESSABLE_ENTITY,
        }
        .into_response()
    }
}
