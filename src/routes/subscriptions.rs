use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct FormData {
    _email: String,
    _name: String,
}

pub async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
