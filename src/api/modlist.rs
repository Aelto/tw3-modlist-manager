use std::path::Path;
use std::fs;

use serde::{Deserialize, Serialize};
use actix_web::{web, HttpRequest, HttpResponse, Result, http};
use crate::constants;
use dirs;

use crate::utils::api_error::api_error;
use crate::models::modlist::ModList;

#[derive(Serialize, Deserialize)]
pub struct InstallModListBody {
  pub name: String,
}

pub async fn install_modlist(req: HttpRequest, form: web::Form<InstallModListBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let modlist = modlist.unwrap();

  modlist.install()
  .map_err(|err| {
    HttpResponse::InternalServerError()
      .content_type("text/plain")
        .body(format!("Internal server error: could not install modlist {}. {}", modlist.name, err))
  })?;

  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, "/")
      .content_type("text/plain")
      .body("installed")
  )
}

#[derive(Serialize, Deserialize)]
pub struct CreateModListBody {
  pub modlist_name: String,
}

pub async fn create_modlist(req: HttpRequest, form: web::Form<CreateModListBody>) -> Result<HttpResponse> {
  let _modlist = ModList::create(&form.modlist_name)
  .map_err(|err| {
    HttpResponse::InternalServerError()
    .content_type("text/plain")
    .body(format!("Internal server error: could not read modlist metadata. {}", err))
  })?;

  Ok(
    HttpResponse::Found()
    .header(http::header::LOCATION, "/")
    .content_type("text/plain")
    .body("created")
  )
}

#[derive(Serialize, Deserialize)]
pub struct ImportModListBody {
  pub modlist_name: String,
  pub imported_name: String,
}

pub async fn import_modlist(req: HttpRequest, form: web::Form<ImportModListBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let mut modlist = modlist.unwrap();
  
  modlist.read_imports_from_disk()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not read modlist metadata. {}", err))
  })?;

  modlist.import_modlist(&form.imported_name);

  modlist.write_imports_to_disk()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not write modlist metadata. {}", err))
  })?;

  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
      .content_type("text/plain")
      .body("imported")
  )
}

#[derive(Serialize, Deserialize)]
pub struct RemoveImportModListBody {
  pub modlist_name: String,
  pub imported_name: String,
}

pub async fn remove_imported_modlist(req: HttpRequest, form: web::Form<RemoveImportModListBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let mut modlist = modlist.unwrap();
  
  modlist.read_imports_from_disk()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not read modlist metadata. {}", err))
  })?;

  modlist.remove_import(&form.imported_name);

  modlist.write_imports_to_disk()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not write modlist metadata. {}", err))
  })?;

  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
      .content_type("text/plain")
      .body("removed")
  )
}

#[derive(Serialize, Deserialize)]
pub struct ModListLoadImportsBody {
  pub modlist_name: String,
}

pub async fn load_imports_modlist(req: HttpRequest, form: web::Form<ModListLoadImportsBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);
  println!("loading imports");

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
      .content_type("text/plain")
      .body(format!("modlist {} not found", form.modlist_name))
    );
  }

  let mut modlist = modlist.unwrap();

  modlist.load_imported_modlists()
  .map_err(|err| {
    HttpResponse::InternalServerError()
    .content_type("text/plain")
    .body(format!("Internal server error: could not load all imported modlists. {}", err))
  })?;

  Ok(
    HttpResponse::Found()
    .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
    .content_type("text/plain")
    .body("loaded")
  )
}

#[derive(Serialize, Deserialize)]
pub struct ModListUnloadImportsBody {
  pub modlist_name: String,
}

pub async fn unload_imports_modlist(req: HttpRequest, form: web::Form<ModListUnloadImportsBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
      .content_type("text/plain")
      .body(format!("modlist {} not found", form.modlist_name))
    );
  }

  let modlist = modlist.unwrap();

  modlist.unload_imported_modlists()
  .map_err(|err| {
    HttpResponse::InternalServerError()
    .content_type("text/plain")
    .body(format!("Internal server error: could not unload all imported modlists. {}", err))
  })?;

  Ok(
    HttpResponse::Found()
    .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
    .content_type("text/plain")
    .body("unloaded")
  )
}

pub async fn initialize(req: HttpRequest) -> Result<HttpResponse> {
  let witcher_root = Path::new(constants::WITCHER_GAME_ROOT);
    
  let current_mods_path = witcher_root.join("mods");
  let current_dlc_path = witcher_root.join("dlc");
  let current_content_path = witcher_root
    .join("content")
    .join("content0")
    .join("scripts");
  let current_saves_path = dirs::document_dir().ok_or(
    HttpResponse::InternalServerError()
    .content_type("text/plain")
    .body("Internal server error: could not find the Documents directory")
  )?.join("The Witcher 3");
  let current_menu_path = witcher_root
    .join("bin")
    .join("config")
    .join("r4game")
    .join("user_config_matrix")
    .join("pc");

  println!("
  current_mods_path = {:?}
  current_dlc_path = {:?}
  current_content_path = {:?}
  current_saves_path = {:?}
  current_menu_path = {:?}
  ",
  current_mods_path,
  current_dlc_path,
  current_content_path,
  current_saves_path,
  current_menu_path);

  let modlist_database = Path::new(constants::MODLIST_DATABASE_PATH);
  
  let vanilla_modlist = modlist_database.join("vanilla");
  let vanilla_mods_path = vanilla_modlist.join("mods");
  let vanilla_dlc_path = vanilla_modlist.join("dlcs");
  let vanilla_menu_path = vanilla_modlist.join("menus");
  let vanilla_content_path = vanilla_modlist.join("content");
  let vanilla_saves_path = vanilla_modlist.join("saves");

  let result = fs::create_dir_all(vanilla_modlist)
  .map_err(|err| api_error(format!("could not create the vanilla modlist directory: {}", err)))
  .and_then(|_| {
    fs::rename(&current_mods_path, &vanilla_mods_path)
    .or_else(|_| {
      // vanilla has no mods folder, so if it fails, just create an empty mods folder
      // in the modlist
      fs::create_dir(&vanilla_mods_path)?;

      Ok(())
    })
    .map_err(|err: std::io::Error| api_error(format!("could not transfer the mods into the vanilla modlist: {}", err)))
  })
  .and_then(|_| {
    fs::rename(&current_dlc_path, &vanilla_dlc_path)
    .map_err(|err| api_error(format!("could not transfer the dlcs into the vanilla modlist: {}", err)))
  })
  .and_then(|_| {
    fs::rename(&current_menu_path, &vanilla_menu_path)
    .map_err(|err| api_error(format!("could not transfer the menus into the vanilla modlist: {}", err)))
  })
  .and_then(|_| {
    fs::rename(&current_content_path, &vanilla_content_path)
    .map_err(|err| api_error(format!("could not transfer the content scripts into the vanilla modlist: {}", err)))
  })
  .and_then(|_| {
    fs::rename(&current_saves_path, &vanilla_saves_path)
    .or_else(|_| {
      fs::copy(&current_saves_path, &vanilla_saves_path)?;

      fs::remove_dir_all(&current_saves_path)?;

      Ok(())
    })
    .map_err(|err: std::io::Error| api_error(format!("could not transfer the save into the vanilla modlist: {}", err)))
  });

  // if it failed, revert everything to its original location
  if result.is_err() {
    fs::rename(&vanilla_mods_path, &current_mods_path);
    fs::rename(&vanilla_dlc_path, &current_dlc_path);
    fs::rename(&vanilla_menu_path, &current_menu_path);
    fs::rename(&vanilla_content_path, &current_content_path);
    fs::rename(&vanilla_saves_path, &current_saves_path);
  }

  result?;

  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, "/")
      .content_type("text/plain")
      .body("initialized")
  )

}

#[derive(Serialize, Deserialize)]
pub struct MoveModListDownBody {
  pub modlist_name: String,
  pub imported_modlist_name: String,
}

pub async fn move_imported_modlist_down(req: HttpRequest, form: web::Form<MoveModListDownBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let mut modlist = modlist.unwrap();

  modlist.read_imports_from_disk()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not read modlist metadata. {}", err))
  })?;

  modlist.move_import_down(&form.imported_modlist_name);

  modlist.write_imports_to_disk()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not write modlist metadata. {}", err))
  })?;

  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
      .content_type("text/plain")
      .body("import moved down")
  )
}

#[derive(Serialize, Deserialize)]
pub struct MoveModListUpBody {
  pub modlist_name: String,
  pub imported_modlist_name: String,
}

pub async fn move_imported_modlist_up(req: HttpRequest, form: web::Form<MoveModListUpBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let mut modlist = modlist.unwrap();

  modlist.read_imports_from_disk()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not read modlist metadata. {}", err))
  })?;

  modlist.move_import_up(&form.imported_modlist_name);

  modlist.write_imports_to_disk()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not write modlist metadata. {}", err))
  })?;

  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
      .content_type("text/plain")
      .body("import moved up")
  )
}

#[derive(Serialize, Deserialize)]
pub struct ViewModListBody {
  pub modlist_name: String
}

pub async fn view_modlist(req: HttpRequest, form: web::Form<ViewModListBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let modlist = modlist.unwrap();

  std::process::Command::new("explorer")
    .arg(modlist.path())
    .output()
    .map_err(|err| {
      HttpResponse::InternalServerError()
          .content_type("text/plain")
          .body(format!("Internal server error: could not view modlist. {}", err))
    })?;


  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
      .content_type("text/plain")
      .body("modlist viewed")
  )
}