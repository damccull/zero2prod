use actix_web::{HttpRequest, Responder, HttpResponse};

pub async fn health_check(req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}
