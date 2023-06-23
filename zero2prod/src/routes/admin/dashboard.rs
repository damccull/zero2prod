use axum::{extract::State, response::IntoResponse, Extension};
use axum_extra::response::Html;
use axum_macros::debug_handler;
use http::StatusCode;
use sqlx::PgPool;

use crate::{
    authentication::{get_username, UserId},
    e500,
    error::ResponseError,
};

#[debug_handler]
#[tracing::instrument(name = "Admin Dashboard", skip(pool, user_id))]
pub async fn admin_dashboard(
    Extension(user_id): Extension<UserId>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ResponseError> {
    let username = get_username(*user_id, &pool).await.map_err(e500)?;

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
        <li><a href="/admin/newsletters">Send a newsletter issue</li>
        <li><a href="/admin/password">Change password</a></li>
        <li>
            <form name="logoutForm" action="/admin/logout" method="post">
                <input type="submit" value="Logout">
            </form>
        </li>
    </ol>
</body>
</html>
"#
        ),
    ));
    Ok(response.into_response())
}
