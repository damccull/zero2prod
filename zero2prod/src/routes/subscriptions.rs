use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{types::Uuid, PgPool};

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
    match insert_subscriber(&db, &form).await {
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

#[tracing::instrument(name = "[Saving new subscriber details in the database]", skip(db, form))]
async fn insert_subscriber(db: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to exeucte query: {:?}", e);
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
