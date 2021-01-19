use actix_web::HttpResponse;

pub fn api_error(message: String) -> HttpResponse {
  HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(message)
}