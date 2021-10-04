use actix_web::HttpResponse;

#[tracing::instrument(name = "Confirm a pending subscriber")]
#[allow(clippy::async_yields_async)]
pub async fn confirm() -> HttpResponse {
    HttpResponse::Ok().finish()
}
