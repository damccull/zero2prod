use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use axum_extra::response::Html;
use axum_macros::debug_handler;
use hmac::{Hmac, Mac};
use http::StatusCode;
use secrecy::ExposeSecret;
use serde::Deserialize;

use crate::startup::HmacSecret;

#[derive(Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String,
}

impl QueryParams {
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag)?;
        let query_string = format!("error={}", urlencoding::Encoded::new(&self.error));
        let mut mac =
            Hmac::<sha3::Sha3_256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;
        Ok(self.error)
    }
}

#[debug_handler]
#[tracing::instrument(name = "Login form", skip(hmac_secret, query))]
pub async fn login_form(
    State(hmac_secret): State<HmacSecret>,
    query: Option<Query<QueryParams>>,
) -> impl IntoResponse {
    tracing::error!("Testing");

    let error_html = match query {
        Some(Query(query)) => match query.verify(&hmac_secret) {
            Ok(error) => {
                format!("<p><i>{}</i></p>", htmlescape::encode_minimal(&error))
            }
            Err(e) => {
                tracing::warn!(error.message = %e,
                    error.cause = ?e,
                    "Failed to verify query parameters using HMAC tag.");
                "".into()
            }
        },
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
