use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{types::Uuid, PgPool};

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

#[tracing::instrument(
    name="[Adding a new subscriber]",
    skip(db,form),
    fields(
        //request_id=%Uuid::new_v4(),
        subscriber_email=%form.email,
        subscriber_name=%form.name
    )
)]
pub async fn subscribe(State(db): State<PgPool>, Form(form): Form<FormData>) -> impl IntoResponse {
    tracing::info!(
        "Adding '{}' '{}' as a new subscriber.",
        form.email,
        form.name
    );

    let name = match SubscriberName::parse(form.name) {
        Ok(name) => name,
        Err(_) => {
            return StatusCode::UNPROCESSABLE_ENTITY;
        }
    };

    let email = match SubscriberEmail::parse(form.email) {
        Ok(email) => email,
        Err(_) => {
            return StatusCode::UNPROCESSABLE_ENTITY;
        }
    };

    let new_subscriber = NewSubscriber { email, name };

    match insert_subscriber(&db, &new_subscriber).await {
        Ok(_) => {
            tracing::info!("New subscriber details have been saved.");
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("failed to save subscriber details: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[tracing::instrument(
    name = "[Saving new subscriber details in the database]",
    skip(db, new_subscriber)
)]
async fn insert_subscriber(db: &PgPool, new_subscriber: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
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
    email: String,
    name: String,
}
