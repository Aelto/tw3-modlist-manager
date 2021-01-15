use actix_web::{HttpRequest, HttpResponse, Result};


pub async fn exit(req: HttpRequest) -> Result<HttpResponse> {
  std::process::exit(0);

  Ok(
    HttpResponse::Found()
      .content_type("text/plain")
      .body("closed")
  )
}

pub async fn ping(req: HttpRequest) -> Result<HttpResponse> {
  Ok(
    HttpResponse::Found()
      .content_type("text/plain")
      .body("")
  )
}