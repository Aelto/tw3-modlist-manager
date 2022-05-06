use std::path::Path;
use std::fs;

use serde::{Deserialize, Serialize};
use actix_web::{web, HttpRequest, HttpResponse, Result, http};
use crate::constants;
use dirs;

use crate::utils::api_error::api_error;
use crate::utils::copy_across_drives;
use crate::models::modlist::ModList;

#[derive(Serialize, Deserialize)]
pub struct InstallModListBody {
  pub name: String,
}

pub async fn install_modlist(_req: HttpRequest, form: web::Form<InstallModListBody>) -> Result<HttpResponse> {
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

pub async fn create_modlist(_req: HttpRequest, form: web::Form<CreateModListBody>) -> Result<HttpResponse> {
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

pub async fn import_modlist(_req: HttpRequest, form: web::Form<ImportModListBody>) -> Result<HttpResponse> {
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
  
  modlist.read_metadata_from_disk()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not read modlist metadata. {}", err))
  })?;

  modlist.import_modlist(&form.imported_name);

  modlist.write_metadata_to_disk()
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

pub async fn remove_imported_modlist(_req: HttpRequest, form: web::Form<RemoveImportModListBody>) -> Result<HttpResponse> {
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
  
  modlist.read_metadata_from_disk()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not read modlist metadata. {}", err))
  })?;

  modlist.remove_import(&form.imported_name);

  modlist.write_metadata_to_disk()
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

pub async fn load_imports_modlist(_req: HttpRequest, form: web::Form<ModListLoadImportsBody>) -> Result<HttpResponse> {
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

pub async fn unload_imports_modlist(_req: HttpRequest, form: web::Form<ModListUnloadImportsBody>) -> Result<HttpResponse> {
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

pub async fn initialize(_req: HttpRequest) -> Result<HttpResponse> {
  let witcher_root = Path::new(constants::WITCHER_GAME_ROOT);
    
  let current_mods_path = witcher_root.join("mods");
  let current_dlc_path = witcher_root.join("dlc");
  let current_content_path = witcher_root
    .join("content")
    .join("content0")
    .join("scripts");
  let current_bundles_path = witcher_root
    .join("content")
    .join("content0")
    .join("bundles");
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

  let modlist_database = Path::new(constants::MODLIST_DATABASE_PATH);
  
  let vanilla_modlist = modlist_database.join("vanilla");
  let vanilla_mods_path = vanilla_modlist.join("mods");
  let vanilla_dlc_path = vanilla_modlist.join("dlcs");
  let vanilla_menu_path = vanilla_modlist.join("menus");
  let vanilla_content_path = vanilla_modlist.join("content");
  let vanilla_bundles_path = vanilla_modlist.join("bundles");
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
    fs::rename(&current_bundles_path, &vanilla_bundles_path)
    .map_err(|err| api_error(format!("could not transfer the content bundles into the vanilla modlist: {}", err)))
  })
  .and_then(|_| {
    fs::rename(&current_saves_path, &vanilla_saves_path)
    .or_else(|_| {
      copy_across_drives(current_saves_path.clone(), vanilla_saves_path.clone())?;

      fs::remove_dir_all(&current_saves_path)?;

      Ok(())
    })
    .map_err(|err: std::io::Error| api_error(format!("could not transfer the save into the vanilla modlist: {}", err)))
  });

  // if it failed, revert everything to its original location
  if result.is_err() {
    if let Err(error) = fs::rename(&vanilla_mods_path, &current_mods_path) {
      println!("could not rename {:?} to {:?}, error: {}", &vanilla_mods_path, &current_mods_path, error);
    };

    if let Err(error) = fs::rename(&vanilla_dlc_path, &current_dlc_path) {
      println!("could not rename {:?} to {:?}, error: {}", &vanilla_mods_path, &current_mods_path, error);
    };

    if let Err(error) = fs::rename(&vanilla_menu_path, &current_menu_path) {
      println!("could not rename {:?} to {:?}, error: {}", &vanilla_mods_path, &current_mods_path, error);
    };

    if let Err(error) = fs::rename(&vanilla_content_path, &current_content_path) {
      println!("could not rename {:?} to {:?}, error: {}", &vanilla_mods_path, &current_mods_path, error);
    };

    if let Err(error) = fs::rename(&vanilla_bundles_path, &current_bundles_path) {
      println!("could not rename {:?} to {:?}, error: {}", &vanilla_mods_path, &current_mods_path, error);
    };

    if let Err(error) = fs::rename(&vanilla_saves_path, &current_saves_path) {
      println!("could not rename {:?} to {:?}, error: {}", &vanilla_mods_path, &current_mods_path, error);
    };
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

pub async fn move_imported_modlist_down(_req: HttpRequest, form: web::Form<MoveModListDownBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let mut modlist = modlist.unwrap();

  modlist.read_metadata_from_disk()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not read modlist metadata. {}", err))
  })?;

  modlist.move_import_down(&form.imported_modlist_name);

  modlist.write_metadata_to_disk()
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

pub async fn move_imported_modlist_up(_req: HttpRequest, form: web::Form<MoveModListUpBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let mut modlist = modlist.unwrap();

  modlist.read_metadata_from_disk()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not read modlist metadata. {}", err))
  })?;

  modlist.move_import_up(&form.imported_modlist_name);

  modlist.write_metadata_to_disk()
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
  pub modlist_name: String,
  pub folder_name: String
}

pub async fn view_modlist(_req: HttpRequest, form: web::Form<ViewModListBody>) -> Result<HttpResponse> {
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
    .arg(modlist.path().join(&form.folder_name))
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

#[derive(Serialize, Deserialize)]
pub struct MergeModListBody {
  pub modlist_name: String
}

pub async fn merge_modlist(_req: HttpRequest, form: web::Form<MergeModListBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let modlist = modlist.unwrap();

  // first we install the modlist as scriptmerger can only work on an installed
  // modlist
  modlist.install()
  .map_err(|err| {
    HttpResponse::InternalServerError()
      .content_type("text/plain")
        .body(format!("Internal server error: could not install modlist {}. {}", modlist.name, err))
  })?;

  let scriptmerger_path = std::env::current_dir()
    .unwrap()
    .join(constants::SCRIPTMERGER_PATH);

  if cfg!(target_os = "windows") {
    std::process::Command::new("cmd")
      .arg("/C")
      .arg("start")
      .arg("/D")
      .arg(scriptmerger_path)
      .arg(constants::SCRIPTMERGER_EXE_NAME)
      .output()
      .map_err(|err| {
        HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("Internal server error: could not merge modlist. {}.
            Make sure your scriptmerger is installed in the correct directory,
            please refer to the written documentation about merging modlists for
            more information", err))
      })?;
  }
  else if cfg!(target_os = "linux") {
    std::process::Command::new("sh")
      .arg("-c")
      .arg(scriptmerger_path.join(constants::SCRIPTMERGER_EXE_NAME))
      .output()
      .map_err(|err| {
        HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("Internal server error: could not merge modlist. {}.
            Make sure your scriptmerger is installed in the correct directory,
            please refer to the written documentation about merging modlists for
            more information", err))
      })?;
  }


  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
      .content_type("text/plain")
      .body("modlist merged")
  )
}

pub async fn merge_modlist_scripts(_req: HttpRequest, form: web::Form<MergeModListBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let modlist = modlist.unwrap();

  println!("dsf");

  crate::api::socket_merge::main(modlist.name).await;

  println!("dsf");


  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
      .content_type("text/plain")
      .body("modlist merged")
  )
}


/// this version of the merge action uses the tw3-script-merger tool instead of
/// scriptmerger
pub async fn merge_modlist_scripts_old(_req: HttpRequest, form: web::Form<MergeModListBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let modlist = modlist.unwrap();

  let scriptmerger_path = std::env::current_dir()
    .unwrap()
    .join(constants::TW3SCRIPTMERGER_PATH);

  let source_path = modlist.content_path();
  let input_path = modlist.mods_path();
  let output_path = input_path
    .join(constants::SCRIPTMERGER_MERGEDFILES_FOLDERNAME)
    .join("content")
    .join("scripts");

  use std::process::{Command, Stdio};
  use std::io::{BufRead, BufReader, Error, ErrorKind};

  let stdout = Command::new(scriptmerger_path)
    .arg("--clean")
    .arg("--texteditor")
    .arg("code")
    .arg("--source")
    .arg(&source_path)
    .arg("--input")
    .arg(&input_path)
    .arg("--output")
    .arg(&output_path)
    .stdout(Stdio::piped())
    .spawn()
    .map_err(|err| {
      HttpResponse::InternalServerError()
          .content_type("text/plain")
          .body(format!("Internal server error: could not run the tw3-script-merger tool. {}", err))
    })?
    .stdout
    .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output"))
    .map_err(|err| {
      HttpResponse::InternalServerError()
          .content_type("text/plain")
          .body(format!("Internal server error: an error occured when listening to the tw3-script-merger tool. {}", err))
    })?;

  let reader = BufReader::new(stdout);

  reader
      .lines()
      .filter_map(|line| line.ok())
      .for_each(|line| println!("{}", line));

  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
      .content_type("text/plain")
      .body("modlist merged")
  )
}

#[derive(Serialize, Deserialize)]
pub struct ModlistVisibilityUpBody {
  pub modlist_name: String,
}

pub async fn modlist_visibility_up(_req: HttpRequest, form: web::Form<ModlistVisibilityUpBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let mut modlist = modlist.unwrap();

  modlist.read_metadata_from_disk()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not read modlist metadata. {}", err))
  })?;

  modlist.visibility += 1;

  modlist.write_metadata_to_disk()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not write modlist metadata. {}", err))
  })?;

  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
      .content_type("text/plain")
      .body("visibility decreased")
  )
}

#[derive(Serialize, Deserialize)]
pub struct ModlistVisibilityDownBody {
  pub modlist_name: String,
}

pub async fn modlist_visibility_down(_req: HttpRequest, form: web::Form<ModlistVisibilityDownBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let mut modlist = modlist.unwrap();

  modlist.read_metadata_from_disk()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not read modlist metadata. {}", err))
  })?;

  modlist.visibility -= 1;

  modlist.write_metadata_to_disk()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not write modlist metadata. {}", err))
  })?;

  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
      .content_type("text/plain")
      .body("visibility increased")
  )
}

#[derive(Serialize, Deserialize)]
pub struct PackModlistBody {
  pub modlist_name: String,
}

pub async fn pack_modlist(_req: HttpRequest, form: web::Form<PackModlistBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let modlist = modlist.unwrap();

  modlist.pack()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not pack the modlist. {}", err))
  })?;

  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
      .content_type("text/plain")
      .body("modlist packed")
  )
}

#[derive(Serialize, Deserialize)]
pub struct UnpackModlistBody {
  pub modlist_name: String,
}

pub async fn unpack_modlist(_req: HttpRequest, form: web::Form<UnpackModlistBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let modlist = modlist.unwrap();

  modlist.unpack()
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not unpack the modlist. {}", err))
  })?;

  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
      .content_type("text/plain")
      .body("modlist unpacked")
  )
}

#[derive(Serialize, Deserialize)]
pub struct RenameModlistFolderBody {
  pub modlist_name: String,
  pub folder_type: String,
  pub folder_name: String,
  pub new_folder_name: String
}

pub async fn rename_modlist_folder(_req: HttpRequest, form: web::Form<RenameModlistFolderBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let modlist = modlist.unwrap();
  let origin = modlist.path()
    .join(&form.folder_type)
    .join(&form.folder_name);

  let destination = modlist.path()
    .join(&form.folder_type)
    .join(&form.new_folder_name);

  fs::rename(origin, destination)
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not rename the file. {}", err))
  })?;

  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
      .content_type("text/plain")
      .body("folder rename")
  )
}

#[derive(Serialize, Deserialize)]
pub struct MoveModlistFolderBody {
  pub modlist_name: String,
  pub folder_type: String,
  pub folder_name: String,
  pub new_modlist_name: String
}

pub async fn move_modlist_folder(_req: HttpRequest, form: web::Form<MoveModlistFolderBody>) -> Result<HttpResponse> {
  fn get_modlist(modlist_name: &str) -> Result<ModList, HttpResponse> {
    let modlist = ModList::get_by_name(modlist_name);

    if modlist.is_none() {
      return Err(
        HttpResponse::NotFound()
          .content_type("text/plain")
          .body("no such modlist")
      )
    }

    return Ok(modlist.unwrap());
  }

  let modlist = get_modlist(&form.modlist_name)?;
  let new_modlist = get_modlist(&form.new_modlist_name)?;

  let origin = modlist.path()
    .join(&form.folder_type)
    .join(&form.folder_name);

  let destination = new_modlist.path()
    .join(&form.folder_type)
    .join(&form.folder_name);

  fs::rename(origin, destination)
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not move the file. {}", err))
  })?;

  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
      .content_type("text/plain")
      .body("folder moved")
  )
}

#[derive(Serialize, Deserialize)]
pub struct DeleteModlistFolderBody {
  pub modlist_name: String,
  pub folder_type: String,
  pub folder_name: String,
  pub folder_name_confirmation: String
}

pub async fn delete_modlist_folder(_req: HttpRequest, form: web::Form<DeleteModlistFolderBody>) -> Result<HttpResponse> {
  // this endpoint requires a confirmation for the folder name. If they it doesn't
  // match
  if form.folder_name != form.folder_name_confirmation {
    return Ok(
      HttpResponse::Found()
        .header(http::header::LOCATION, format!("/modlist/{}/edit/{}/{}", form.modlist_name, form.folder_type, form.folder_name))
        .content_type("text/plain")
        .body("confirmation does not match")
    )
  }

  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let modlist = modlist.unwrap();

  let origin = modlist.path()
    .join(&form.folder_type)
    .join(&form.folder_name);

  if origin.is_dir() {
    fs::remove_dir_all(&origin)
    .map_err(|err| {
      HttpResponse::InternalServerError()
          .content_type("text/plain")
          .body(format!("Internal server error: could not remove the directory. {}", err))
    })?;
  }

  if origin.is_file() {
    fs::remove_file(&origin)
    .map_err(|err| {
      HttpResponse::InternalServerError()
          .content_type("text/plain")
          .body(format!("Internal server error: could not remove the file. {}", err))
    })?;
  }

  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/modlist/{}", form.modlist_name))
      .content_type("text/plain")
      .body("folder deleted")
  )
}

#[derive(Serialize, Deserialize)]
pub struct DeleteModlistBody {
  pub modlist_name: String,
  pub modlist_name_confirmation: String
}

pub async fn delete_modlist(_req: HttpRequest, form: web::Form<DeleteModlistBody>) -> Result<HttpResponse> {
  // this endpoint requires a confirmation for the folder name. If they it doesn't
  // match
  if form.modlist_name != form.modlist_name_confirmation {
    return Ok(
      HttpResponse::Found()
        .header(http::header::LOCATION, format!("/modlist/{}/edit", form.modlist_name))
        .content_type("text/plain")
        .body("confirmation does not match")
    )
  }

  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let modlist = modlist.unwrap();
  let origin = modlist.path();

  if origin.is_dir() {
    fs::remove_dir_all(&origin)
    .map_err(|err| {
      HttpResponse::InternalServerError()
          .content_type("text/plain")
          .body(format!("Internal server error: could not remove the directory. {}", err))
    })?;
  }

  if origin.is_file() {
    fs::remove_file(&origin)
    .map_err(|err| {
      HttpResponse::InternalServerError()
          .content_type("text/plain")
          .body(format!("Internal server error: could not remove the file. {}", err))
    })?;
  }

  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/", ))
      .content_type("text/plain")
      .body("modlist deleted")
  )
}

#[derive(Serialize, Deserialize)]
pub struct RenameModlistBody {
  pub modlist_name: String,
  pub new_modlist_name: String
}

pub async fn rename_modlist(_req: HttpRequest, form: web::Form<RenameModlistBody>) -> Result<HttpResponse> {
  let modlist = ModList::get_by_name(&form.modlist_name);

  if modlist.is_none() {
    return Ok(
      HttpResponse::NotFound()
        .content_type("text/plain")
        .body("no such modlist")
    )
  }

  let modlist = modlist.unwrap();
  let origin = modlist.path();

  let mut destination = modlist.path();
  destination.set_file_name(&form.new_modlist_name);

  fs::rename(origin, destination)
  .map_err(|err| {
    HttpResponse::InternalServerError()
        .content_type("text/plain")
        .body(format!("Internal server error: could not rename the modlist. {}", err))
  })?;

  Ok(
    HttpResponse::Found()
      .header(http::header::LOCATION, format!("/modlist/{}", form.new_modlist_name))
      .content_type("text/plain")
      .body("folder rename")
  )
}