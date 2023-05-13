use anyhow::Context;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};
use axum_extra::response::Html;
use axum_macros::debug_handler;
use axum_session::SessionRedisPool;
use http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{e500, error::ResponseInternalServerError, session_state::TypedSession};

#[debug_handler]
#[tracing::instrument(name = "Admin Dashboard", skip(pool, session))]
pub async fn admin_dashboard(
    State(pool): State<PgPool>,
    session: TypedSession<SessionRedisPool>,
) -> Result<impl IntoResponse, ResponseInternalServerError<anyhow::Error>> {
    let username = if let Some(user_id) = session.get_user_id() {
        get_username(user_id, &pool).await.map_err(e500)?
    } else {
        return Ok(Redirect::to("/login").into_response());
    };

    let response = Html((
        StatusCode::OK,
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Admin dashboard</title>
</head>
<body>
    <p>Welcome {username}!</p>
    <p>Available actions:</p>
    <ol>
        <li><a href="/admin/password">Change password</a></li>
    </ol>
</body>
</html>
"#
        ),
    ));
    Ok(response.into_response())
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
