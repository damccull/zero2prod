use axum::response::IntoResponse;
use axum_extra::{
    extract::{cookie::Cookie, CookieJar},
    response::Html,
};
use axum_macros::debug_handler;
use http::StatusCode;

#[debug_handler]
#[tracing::instrument(name = "Login form")]
pub async fn login_form(jar: CookieJar) -> impl IntoResponse {
    let error_html = match jar.get("_flash") {
        None => "".into(),
        Some(cookie) => {
            format!("<p><i>{}</i></p>", cookie.value())
        }
    };

    let mut eat_flash_cookie = Cookie::new("_flash", "");
    eat_flash_cookie.make_removal();

    let jar = jar.remove(Cookie::named("_flash")).add(eat_flash_cookie);

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

    (jar, body_response)
}
