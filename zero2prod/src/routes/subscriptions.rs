use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{types::Uuid, PgPool};

use crate::{domain::NewSubscriber, email_client::EmailClient};

#[tracing::instrument(
    name="[Adding a new subscriber]",
    skip(db, email_client, form),
    fields(
        subscriber_email=%form.email,
        subscriber_name=%form.name
    )
)]
pub async fn subscribe(
    State(db): State<PgPool>,
    State(email_client): State<Arc<EmailClient>>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    tracing::info!(
        "Adding '{}' '{}' as a new subscriber.",
        form.email,
        form.name
    );

    let new_subscriber = match form.try_into() {
        Ok(subscriber) => subscriber,
        Err(e) => {
            tracing::error!("failed to parse subscriber: {:?}", e);
            return StatusCode::UNPROCESSABLE_ENTITY;
        }
    };

    if insert_subscriber(&db, &new_subscriber).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    if send_confirmation_email(email_client, new_subscriber)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

#[tracing::instrument(
    name="Send confirmation email"
    skip(email_client, new_subscriber)
)]
async fn send_confirmation_email(
    email_client: Arc<EmailClient>,
    new_subscriber: NewSubscriber,
) -> Result<(), reqwest::Error> {
    let confirmation_link = "https://there-is-no-such-domain.com/subscriptions/confirm";
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
    skip(db, new_subscriber)
)]
async fn insert_subscriber(db: &PgPool, new_subscriber: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'confirmed')
    "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct FormData {
    pub email: String,
    pub name: String,
}
