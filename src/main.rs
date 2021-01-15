#![feature(proc_macro_hygiene)]

// uncomment the line below when building a release.
// It allows the binary to start in background without a cli window.
#![windows_subsystem = "windows"]

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

  
  // open a new browser tab
  std::process::Command::new("cmd")
      .arg("/C")
      .arg("start")
      .arg(format!("http://localhost:{}", port))
      .output()?;

  let is_already_running = is_already_running()
    .unwrap_or_else(|error| {
      println!("{}", error);

      true
    });

  if is_already_running {
    std::process::exit(0);
  }

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
        .route("/program/ping", web::post().to(api::program::ping))
        .route("/program/exit", web::post().to(api::program::exit))
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

fn is_already_running() -> Result<bool, String> {
  // open a new browser tab
  let output = std::process::Command::new("cmd")
      .arg("/C")
      .arg("tasklist")
      .arg("/NH")
      .arg("/FO")
      .arg("TABLE")
      .arg("/FI")
      .arg("IMAGENAME eq tw3-modlist-manager.exe")
      .output()
      .map_err(|err| format!("error with the tasklist command: {}", err))?;

  if !output.status.success() {
    return Ok(false);
  }

  let program_name = get_program_name()?;
  let program_count = String::from_utf8(output.stdout)
    .map_err(|_| "could not serialize tasklist command output")?
    
    .matches(&program_name)
    .count();


  Ok(program_count > 1)
}

fn get_program_name() -> Result<String, String> {
  let path = std::env::current_exe().map_err(|err| err.to_string())?;
  let filename = path.file_name().ok_or("could not get program name".to_owned())?;
  let str_filename = filename.to_str().ok_or("could not serialize program name".to_owned())?;

  Ok(
    String::from(str_filename)
  )
}