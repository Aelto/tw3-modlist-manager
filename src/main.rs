#![feature(proc_macro_hygiene)]
#![feature(bool_to_option)]

// uncomment the line below when building a release.
// It allows the binary to start in background without a cli window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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

  std::process::Command::new("cmd")
    .arg("/C")
    .arg("taskkill")
    .arg("/f")
    .arg("/im")
    .arg(get_program_name().expect("cannot get program name"))
    .arg("/fi")
    .arg(format!("PID ne {}", std::process::id()))
    .output()?;

  // open a new browser tab

  #[cfg(not(debug_assertions))]
  std::process::Command::new("cmd")
      .arg("/C")
      .arg("start")
      .arg(format!("http://localhost:{}", port))
      .output()?;

  println!("starting server on port {}", port);

  HttpServer::new(|| {
    App::new()
    // home page
    .service(web::resource("/").route(web::get().to(pages::root::render)))
    .service(web::resource("/modlist/{modlist_name}").route(web::get().to(pages::modlist::render)))
    .service(web::resource("/modlist/{modlist_name}/edit/{folder_type}/{folder_name}").route(web::get().to(pages::modlist_folder_edit::render)))
    .service(web::resource("/modlist/{modlist_name}/edit").route(web::get().to(pages::modlist_edit::render)))
    .service(web::resource("/modlist/{modlist_name}/merge").route(web::get().to(pages::modlist_merge::render)))

    // static files
    // .service(fs::Files::new("/static", "./static"))

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
        .route("/modlist/move-import-down", web::post().to(api::modlist::move_imported_modlist_down))
        .route("/modlist/visibility-up", web::post().to(api::modlist::modlist_visibility_up))
        .route("/modlist/visibility-down", web::post().to(api::modlist::modlist_visibility_down))
        .route("/modlist/view", web::post().to(api::modlist::view_modlist))
        .route("/modlist/merge", web::post().to(api::modlist::merge_modlist))
        .route("/modlist/merge-scripts", web::post().to(api::modlist::merge_modlist_scripts))
        .route("/modlist/pack", web::post().to(api::modlist::pack_modlist))
        .route("/modlist/unpack", web::post().to(api::modlist::unpack_modlist))
        .route("/modlist/folder-rename", web::post().to(api::modlist::rename_modlist_folder))
        .route("/modlist/folder-move", web::post().to(api::modlist::move_modlist_folder))
        .route("/modlist/folder-delete", web::post().to(api::modlist::delete_modlist_folder))
        .route("/modlist/rename", web::post().to(api::modlist::rename_modlist))
        .route("/modlist/delete", web::post().to(api::modlist::delete_modlist))
    )

  })
  .bind(format!("127.0.0.1:{}", port))?
  .run()
  .await
}

fn get_program_name() -> Result<String, String> {
  let path = std::env::current_exe().map_err(|err| err.to_string())?;
  let filename = path.file_name().ok_or("could not get program name".to_owned())?;
  let str_filename = filename.to_str().ok_or("could not serialize program name".to_owned())?;

  Ok(
    String::from(str_filename)
  )
}