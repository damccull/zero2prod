use axum::response::IntoResponse;
use axum_extra::response::Html;
use axum_flash::IncomingFlashes;
use axum_macros::debug_handler;
use std::fmt::Write;

use crate::error::ResponseInternalServerError;

#[debug_handler(state = axum_flash::Config)]
#[tracing::instrument(name = "Publish newsletter issue", skip(flashes))]
pub async fn newsletters_publish_form(
    flashes: IncomingFlashes,
) -> Result<impl IntoResponse, ResponseInternalServerError<anyhow::Error>> {
    let mut msg_html = String::new();
    for (level, text) in flashes.iter() {
        writeln!(
            msg_html,
            "<p><strong>{:?}</strong> - <i>{}</i></p>",
            level, text
        )
        .unwrap();
    }
    let body = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Publish newsletter issue</title>
</head>
<body>
    {msg_html}
    <form action="/admin/newsletters" method="post" enctype="application/x-www-form-urlencoded">
        
        <label>Newsletter Title
            <input type="text" placeholder="Enter newsletter title" name="title">
        </label>
        <br>
        <label>Plain text body
            <input type="textarea" placeholder="Enter plain text body" name="content.text">
        </label>
        <br>
        <label>Html body
            <input type="textarea" placeholder="Enter html body" name="content.html">
        </label>
        <br>
        <button type="submit">Send newsletter</button>
    </form>
</body>
</html>"#
    );
    Ok((flashes, Html(body)))
}
