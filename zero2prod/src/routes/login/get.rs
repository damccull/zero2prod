use axum::{extract::Query, response::IntoResponse};
use axum_extra::response::Html;
use axum_macros::debug_handler;
use http::StatusCode;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct QueryParams {
    error: Option<String>,
}

#[debug_handler]
#[tracing::instrument(name = "Login form", skip(query))]
pub async fn login_form(Query(query): Query<QueryParams>) -> impl IntoResponse {
    let error_html = match query.error {
        Some(error_message) => format!(
            "<p><i>{}</i></p>",
            htmlescape::encode_minimal(&error_message)
        ),
        None => "".into(),
    };
    Html((
        StatusCode::OK,
        format!(
            r#"
            <!DOCTYPE html>
            <html lang="en">
            
            <head>
                <meta http-equiv="content-type" content="text/html; charset=utf-8">
                <title>Login</title>
            </head>
            
            <body>
                {error_html}
                <form action="/login" method="post">
                    <label>Username
                        <input type="text" placeholder="Enter Username" name="username">
                    </label>
                    <label>Password
                        <input type="password" placeholder="Enter Password" name="password">
                    </label>
                    <button type="submit">Login</button>
                </form>
            </body>
            
            </html>
            "#
        ),
    ))
}
