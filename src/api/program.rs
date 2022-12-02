use actix_web::{HttpRequest, HttpResponse, Result};

pub async fn exit(_req: HttpRequest) -> Result<HttpResponse> {
  std::process::exit(0);
}

pub async fn ping(_req: HttpRequest) -> Result<HttpResponse> {
  Ok(HttpResponse::Found().content_type("text/plain").body(""))
}
