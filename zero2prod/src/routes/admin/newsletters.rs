mod get;
mod post;

pub use get::newsletters_publish_form;
pub use post::publish_newsletter;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct BodyData {
    pub title: String,
    pub html_content: String,
    pub text_content: String,
}
