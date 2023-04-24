use anyhow::Context;
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
) -> Result<impl IntoResponse, ConfirmError> {
    let id = get_subscriber_id_from_token(&db_pool, &parameters.subscription_token)
        .await
        .context("Failed to get subscriber id from token.")?;

    match id {
        None => Ok(StatusCode::UNAUTHORIZED),
        Some(subscriber_id) => {
            confirm_subscriber(&db_pool, subscriber_id)
                .await
                .context("Failed to set subscriber to 'confirmed' status.")?;
            Ok(StatusCode::OK)
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
    .await?;
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
    .await?;
    Ok(result.map(|r| r.subscriber_id))
}

#[derive(Debug, Deserialize)]
pub struct ConfirmParameters {
    subscription_token: String,
}

#[derive(thiserror::Error)]
pub enum ConfirmError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for ConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::error_chain_fmt(self, f)
    }
}

impl IntoResponse for ConfirmError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:?}", self);
        match self {
            ConfirmError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}
