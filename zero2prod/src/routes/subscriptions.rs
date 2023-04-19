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

    match insert_subscriber(&db, &new_subscriber).await {
        Ok(_) => {
            tracing::info!("New subscriber details have been saved.");
        }
        Err(e) => {
            tracing::error!("failed to save subscriber details: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    match email_client
        .send_email(
            new_subscriber.clone().email,
            "Welcome!",
            "Welcome to our newsletter!",
            "Welcome to our newsletter!",
        )
        .await
    {
        Ok(_) => {
            tracing::info!("Confirmation email sent to {:?}", new_subscriber.email);
        }
        Err(e) => {
            tracing::error!("failed to send confirmation email: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }
    StatusCode::OK
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
