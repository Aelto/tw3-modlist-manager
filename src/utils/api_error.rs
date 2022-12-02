use std::borrow::Cow;
use std::fmt::Display;

use actix_web::HttpResponse;

pub fn api_error(message: impl Into<Cow<'static, str>>) -> ApiError {
  ApiError(message.into())
}

impl Display for ApiError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug)]
pub struct ApiError(Cow<'static, str>);

impl actix_web::ResponseError for ApiError {
  fn status_code(&self) -> actix_web::http::StatusCode {
    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
  }

  fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
    actix_web::HttpResponse::build(self.status_code())
      .insert_header(actix_web::http::header::ContentType::html())
      .body(self.0.to_string())
  }
}
