use axum::response::IntoResponse;
use axum_extra::response::Html;
use http::StatusCode;

use crate::error::ResponseInternalServerError;

pub async fn change_password_form(
) -> Result<impl IntoResponse, ResponseInternalServerError<anyhow::Error>> {
    let body = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Change Password</title>
</head>
<body>
    <form action="/admin/password" method="post">
        <label>Current password
            <input type="password" placeholder="Enter current password" name="current_password">
        </label>
        <br>
        <label>New password
            <input type="password" placeholder="New password" name="new_password">
        </label>
        <br>
        <label>
            <input type="password" placeholder="Type the new password again" name="new_password_check">
        </label>
        <br>
        <button type="submit">Change password</button>
    </form>
</body>
</html>"#;

    Ok(Html((StatusCode::OK, body)))
}
