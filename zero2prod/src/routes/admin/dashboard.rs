use anyhow::Context;
use axum::{extract::State, response::IntoResponse};
use axum_macros::debug_handler;
use axum_session::{Session, SessionRedisPool};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ResponseInternalServerError;

fn e500<T>(e: T) -> ResponseInternalServerError<T>
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    ResponseInternalServerError::from(e)
}

#[debug_handler]
#[tracing::instrument(name = "Admin Dashboard", skip(pool, session))]
pub async fn admin_dashboard(
    State(pool): State<PgPool>,
    session: Session<SessionRedisPool>,
) -> Result<impl IntoResponse, ResponseInternalServerError<anyhow::Error>> {
    let username = if let Some(user_id) = session.get::<Uuid>("user_id") {
        get_username(user_id, &pool).await.map_err(e500)?
    } else {
        todo!()
    };

    let response_body = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Admin dashboard</title>
</head>
<body>
    <p>Welcome {username}!</p>
</body>
</html>
"#
    )
    .into_response();
    Ok(response_body)
}

#[tracing::instrument(name = "Get username", skip(pool))]
async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT username
        FROM users
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to perform a query to retrieve a username.")?;
    Ok(row.username)
}
