use actix_web::web::{self, HttpResponse};
use serde::Deserialize;

pub async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(Clone, Debug, Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}
