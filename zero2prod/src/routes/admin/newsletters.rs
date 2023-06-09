mod get;
mod post;

pub use get::newsletters_publish_form;
pub use post::publish_newsletter;

#[derive(Debug, serde::Deserialize)]
pub struct BodyData {
    pub title: String,
    pub content: Content,
}

#[derive(Debug, serde::Deserialize)]
pub struct Content {
    pub html: String,
    pub text: String,
}
