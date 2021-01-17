use actix_web::{web, HttpRequest, HttpResponse, Result, http};

pub fn api_error(message: String) -> HttpResponse {
  HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(message)
}