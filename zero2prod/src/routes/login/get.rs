use axum::response::IntoResponse;
use axum_extra::response::Html;
use axum_flash::IncomingFlashes;
use axum_macros::debug_handler;
use http::StatusCode;

#[debug_handler(state = axum_flash::Config)]
#[tracing::instrument(name = "Login form")]
pub async fn login_form(flashes: IncomingFlashes) -> impl IntoResponse {
    let error_html = &flashes
        .iter()
        .fold(String::new(), |mut acc, (level, text)| {
            acc.push_str(&format!(
                "<p><strong>{:?}</strong><i>{}</i></p>\n",
                level, text
            ));
            acc
        });

    let body_response = Html((
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
    ));

    (flashes, body_response)
}
