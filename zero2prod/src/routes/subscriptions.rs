use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{types::Uuid, PgPool};

pub async fn subscribe(State(db): State<PgPool>, Form(form): Form<FormData>) -> impl IntoResponse {
    tracing::info!(
        "Adding '{}' '{}' as a new subscriber.",
        form.email,
        form.name
    );
    tracing::info!("Saving new subscriber details to the database.");
    match sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(&db)
    .await
    {
        Ok(_) => {
            tracing::info!("New subscriber details have been saved.");
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("failed to execute query: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
#[derive(Deserialize)]
#[allow(dead_code)]
pub struct FormData {
    email: String,
    name: String,
}
