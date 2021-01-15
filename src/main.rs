#![feature(proc_macro_hygiene)]

// uncomment the line below when building a release.
// It allows the binary to start in background without a cli window.
// #![windows_subsystem = "windows"]

extern crate chrono;

use actix_web::{App, web, HttpServer};
use actix_files as fs;

mod pages;
mod components;
mod constants;
mod api;
mod models;
mod utils;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  let port: u16 = std::env::args()
    .nth(1)
    .and_then(|n| n.parse::<u16>().ok())
    .unwrap_or(5000);

  println!("checking if port is in use");
  if let Err(_) = std::net::TcpStream::connect(("0.0.0.0", port)) {
    println!("port is already in use");

    // port is already used.
    std::process::Command::new("cmd")
      .arg("/C")
      .arg("start")
      .arg(format!("http://localhost:{}", port))
      .output()?;

    Ok(())
  }
  else {
    println!("starting server on port {}", port);

    HttpServer::new(|| {
      App::new()
      // home page
      .service(web::resource("/").route(web::get().to(pages::root::render)))
      .service(web::resource("/modlist/{modlist_name}").route(web::get().to(pages::modlist::render)))

      // static files
      .service(fs::Files::new("/static", "./static"))

      // api endpoints
      .service(
        web::scope("/api")
          .route("/modlist/initialize", web::post().to(api::modlist::initialize))
          .route("/modlist/create", web::post().to(api::modlist::create_modlist))
          .route("/modlist/install", web::post().to(api::modlist::install_modlist))
          .route("/modlist/import", web::post().to(api::modlist::import_modlist))
          .route("/modlist/remove-import", web::post().to(api::modlist::remove_imported_modlist))
          .route("/modlist/load-imports", web::post().to(api::modlist::load_imports_modlist))
          .route("/modlist/unload-imports", web::post().to(api::modlist::unload_imports_modlist))
          .route("/modlist/move-import-up", web::post().to(api::modlist::move_imported_modlist_up))
          .route("/modlist/move-import-down", web::post().to(api::modlist::move_imported_modlist_down))
          .route("/modlist/view", web::post().to(api::modlist::view_modlist))
      )

    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
  }
}