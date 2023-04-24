use std::{fmt::Display, sync::Arc};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Result},
    Form,
};
use axum_macros::debug_handler;
use chrono::Utc;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Deserialize;
use sqlx::{types::Uuid, PgPool, Postgres, Transaction};

use crate::{
    domain::NewSubscriber,
    email_client::EmailClient,
    startup::{AppState, ApplicationBaseUrl},
    telemetry::MyErrorResponse,
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
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, MyErrorResponse> {
    tracing::info!(
        "Adding '{}' '{}' as a new subscriber.",
        form.email,
        form.name
    );

    let mut transaction = match db.begin().await {
        Ok(transaction) => transaction,
        Err(e) => {
            tracing::error!("Failed to get a database transaction: {:?}", e);
            return Err(e.into());
        }
    };

    let new_subscriber = match form.try_into() {
        Ok(subscriber) => subscriber,
        Err(e) => {
            let x = axum::Error::new(e);
            return Err(MyErrorResponse::from(x).status_code(StatusCode::UNPROCESSABLE_ENTITY));
        }
    };

    let subscriber_id = match insert_subscriber(&mut transaction, &new_subscriber).await {
        Ok(subscriber_id) => subscriber_id,
        Err(e) => {
            return Err(e.into());
        }
    };

    let subscription_token = generate_subscription_token();

    if let Err(e) = store_token(&mut transaction, subscriber_id, &subscription_token).await {
        return Err(StoreTokenError(e).into());
    }

    if transaction.commit().await.is_err() {
        tracing::error!("Failed to commit transaction");
        return Err(MyErrorResponse::new(StatusCode::INTERNAL_SERVER_ERROR));
    }

    if send_confirmation_email(
        email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await
    .is_err()
    {
        tracing::error!("Failed to send email");
        return Err(MyErrorResponse::new(StatusCode::INTERNAL_SERVER_ERROR));
    }
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
            new_subscriber.clone().email,
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
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
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
#[allow(dead_code)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

pub enum SubscribeError {
    StatusCode(StatusCode),
    StoreTokenError(sqlx::Error),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StatusCode(arg0) => f.debug_tuple("StatusCode").field(arg0).finish(),
            Self::StoreTokenError(arg0) => f.debug_tuple("StoreTokenError").field(arg0).finish(),
        }
    }
}

impl std::fmt::Display for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscribeError::StoreTokenError(e) => write!(
                f,
                "A database error was encountered while \
                trying to store a subscription token: {}",
                e
            ),
            SubscribeError::StatusCode(c) => write!(f, "{}", c),
        }
    }
}

impl IntoResponse for SubscribeError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{}", self.to_string());
        match self {
            SubscribeError::StatusCode(c) => c.into_response(),
            SubscribeError::StoreTokenError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        }
    }
}

impl std::error::Error for SubscribeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SubscribeError::StatusCode(_) => None,
            SubscribeError::StoreTokenError(e) => Some(e),
        }
    }
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
        write!(f, "{}\nCaused by:\n\t{}", self, self.0)
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}
