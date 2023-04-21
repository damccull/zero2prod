use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use http::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument(name = "Confirm a pending subscription", skip(db_pool, parameters))]
pub async fn confirm(
    State(db_pool): State<PgPool>,
    parameters: Query<ConfirmParameters>,
) -> impl IntoResponse {
    let id = match get_subscriber_id_from_token(&db_pool, &parameters.subscription_token).await {
        Ok(id) => id,
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    match id {
        None => StatusCode::UNAUTHORIZED,
        Some(subscriber_id) => {
            if confirm_subscriber(&db_pool, subscriber_id).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        }
    }
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, db_pool))]
pub async fn confirm_subscriber(db_pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Get subscriber_id from token",
    skip(subscription_token, db_pool)
)]
pub async fn get_subscriber_id_from_token(
    db_pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens
           WHERE subscription_token = $1"#,
        subscription_token
    )
    .fetch_optional(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(result.map(|r| r.subscriber_id))
}

#[derive(Debug, Deserialize)]
pub struct ConfirmParameters {
    subscription_token: String,
}
