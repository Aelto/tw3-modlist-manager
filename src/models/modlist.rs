
use std::fs;
use std::path::{Path, PathBuf};
use fs::{copy, remove_dir_all};
use serde::{Deserialize, Serialize};
use toml;
use fs_extra;

use crate::constants;
use crate::utils::symlinks::{
  symlink_children,
  remove_symlinks,
  make_symlink,
  remove_symlink
};




#[derive(Deserialize, Serialize)]
pub struct ModListConfig {
  imports: Vec<String>,
  visibility: Option<i64>
}

#[derive(Deserialize, Serialize)]
pub struct ImportedModlist {
  pub name: String,
  pub order: i64
}

#[derive(Clone, Debug)]
pub struct ModList {
  pub name: String,
  
  /// it's a list of unique imported modlists. It's a `Vec` and not a `HashSet`
  /// because the ordering is important as it is used as the load order from top
  /// to bottom.
  pub imported_modlists: Vec<String>,

  pub visibility: i64,
}

impl ModList {
  pub fn new(name: String) -> ModList {
    ModList {
      name,
      imported_modlists: Vec::new(),
      visibility: 0
    }
  }

  pub fn import_modlist(&mut self, modlist_name: &str) {
    if !self.imported_modlists.iter().any(|m| m == modlist_name) {
      self.imported_modlists.push(modlist_name.to_owned());
    }
  }

  pub fn remove_import(&mut self, modlist_name: &str) {
    let some_index = self.imported_modlists
      .iter()
      .position(|modlist| modlist == modlist_name);

    if let Some(index) = some_index {
      self.imported_modlists.remove(index);
    }
  }

  /// move the supplied modlist higher in the list, which means at a lower index
  /// as the load order is from top to bottom (0 -> max)
  pub fn move_import_up(&mut self, modlist_name: &str) {
    let some_index = self.imported_modlists
      .iter()
      .position(|modlist| modlist == modlist_name);

    if let Some(index) = some_index {
      if index > 0 {
        self.imported_modlists.swap(
          index,
          index - 1
        );
      }
    }
  }

  /// move the supplied modlist lower in the list, which means at a higher index
  /// as the load order is from top to bottom (0 -> max)
  pub fn move_import_down(&mut self, modlist_name: &str) {
    let some_index = self.imported_modlists
      .iter()
      .position(|modlist| modlist == modlist_name);

    if let Some(index) = some_index {
      if index < self.imported_modlists.len() - 1 {
        self.imported_modlists.swap(
          index,
          index + 1
        );
      }
    }
  }

  /// fill the `Self.imported_modlists` with the data it reads from the disk
  pub fn read_metadata_from_disk(&mut self) -> std::io::Result<()> {
    let config_path = self.config_path();

    if !config_path.exists() {
      return Ok(());
    }

    let text = fs::read_to_string(config_path)?;

    let toml_config: ModListConfig = toml::from_str(&text)?;

    for import in toml_config.imports {
      self.import_modlist(&import);
    }

    self.visibility = toml_config.visibility.unwrap_or(0);

    Ok(())
  }

  pub fn read_metadata_from_disk_copy(&self) -> std::io::Result<Self> {
    let mut copy = self.clone();

    copy.read_metadata_from_disk()?;

    Ok(copy)
  }

  /// update the import list of the disk with the new data in memory
  pub fn write_metadata_to_disk(&self) -> Result<(), String> {
    let config = ModListConfig {
      imports: (&self.imported_modlists)
        .into_iter()
        .map(String::from)
        .collect(),
      visibility: Some(self.visibility)
    };

    let content = toml::to_string_pretty(&config)
      .map_err(|_| format!("config serialization error"))?;

    fs::write(self.config_path(), content)
      .map_err(|err| format!("disk write error {}", err))?;

    Ok(())
  }

  /// remove all imported modlists from the current modlist directories
  pub fn unload_imported_modlists(&self) -> std::io::Result<()> {
    remove_symlinks(&self.mods_path())?;
    remove_symlinks(&self.dlcs_path())?;
    remove_symlinks(&self.menus_path())?;
    remove_symlinks(&self.saves_path())?;
    remove_symlinks(&self.content_path())?;
    remove_symlinks(&self.bundles_path())?;

    Ok(())
  }

  /// load all imported modlists in the current modlist directories in the form
  /// of symlinks pointing to the other modlists' directories.
  pub fn load_imported_modlists(&mut self) -> std::io::Result<()> {
    self.read_metadata_from_disk()?;

    let valid_imported_modlists = self.imported_modlists
      .iter()
      .map(|modlist_name| ModList::get_by_name(&modlist_name))
      .filter(|modlist| modlist.is_some())
      .map(|some_modlist| some_modlist.unwrap())
      .filter(|modlist| modlist.is_valid());

    println!("loading {:?}", self.imported_modlists);
    

    for modlist in valid_imported_modlists {
      println!("loading {}", modlist.name);

      // symlinks to DLCs
      symlink_children(modlist.dlcs_path(), self.dlcs_path())?;

      // symlinks to mods
      symlink_children(modlist.mods_path(), self.mods_path())?;

      // symlinks to menus
      symlink_children(modlist.menus_path(), self.menus_path())?;

      symlink_children(modlist.content_path(), self.content_path())?;
      
      symlink_children(modlist.bundles_path(), self.bundles_path())?;
    }

    Ok(())
  }

  pub fn path(&self) -> PathBuf {
    std::env::current_dir().unwrap()
      // .join(constants::MODLIST_DATABASE_PATH)
      .join(&self.name)
  }

  pub fn dlcs_path(&self) -> PathBuf {
      self.path()
      .join("dlcs")
  }

  pub fn mods_path(&self) -> PathBuf {
    self.path()
      .join("mods")
  }

  pub fn menus_path(&self) -> PathBuf {
    self.path()
      .join("menus")
  }

  pub fn saves_path(&self) -> PathBuf {
    self.path()
      .join("saves")
  }

  pub fn content_path(&self) -> PathBuf {
    self.path()
      .join("content")
  }

  pub fn bundles_path(&self) -> PathBuf {
    self.path()
      .join("bundles")
  }

  pub fn config_path(&self) -> PathBuf {
    self.path()
      .join(constants::MODLIST_CONFIG_NAME)
  }

  pub fn mergeinventory_path(&self) -> PathBuf {
    self.path()
      .join(constants::MODLIST_MERGEINVENTORY_PATH)
  }

  pub fn packbackup_path(&self) -> PathBuf {
    self.mods_path()
      .join(format!("~{}.pack-backup", self.name))
  }

  pub fn pack_path(&self) -> PathBuf {
    self.mods_path()
      .join(format!("mod0000_{}", self.name))
  }

  pub fn mergedfiles_path(&self) -> PathBuf {
    self.mods_path()
      .join(constants::SCRIPTMERGER_MERGEDFILES_FOLDERNAME)
  }

  pub fn is_valid(&self) -> bool {
    let dlcs_path = self.dlcs_path();
    let mods_path = self.mods_path();
    let menus_path = self.menus_path();
    let saves_path = self.saves_path();
    let content_path = self.content_path();
    let bundles_path = self.bundles_path();

    self.path().exists()
    && dlcs_path.exists()
    && mods_path.exists()
    && menus_path.exists()
    && saves_path.exists()
    && content_path.exists()
    && bundles_path.exists()
  }

  pub fn install(&self) -> std::io::Result<()> {
    let witcher_root = std::env::current_dir().unwrap()
      .join(constants::WITCHER_GAME_ROOT);
    
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
    let current_menu_path = witcher_root
      .join("bin")
      .join("config")
      .join("r4game")
      .join("user_config_matrix")
      .join("pc");

    let current_saves_path = dirs::document_dir()
        .ok_or(std::io::ErrorKind::NotFound)?
        .join("The Witcher 3");
    
    // first, we remove all existing symlinks if they exist
    // let them fail if the paths do not exist
    if let Err(error) = remove_symlink(&current_mods_path) {
      println!("could not remove current mod symlink: {}", error);
    }
    if let Err(error) = remove_symlink(&current_dlc_path) {
      println!("could not remove current dlc symlink: {}", error);
    }

    if let Err(error) = remove_symlink(&current_menu_path) {
      println!("could not remove current menu symlink: {}", error);
    }

    if let Err(error) = remove_symlink(&current_saves_path) {
      println!("could not remove current saves symlink: {}", error);
    }
    
    if let Err(error) = remove_symlink(&current_content_path) {
      println!("could not remove current content symlink: {}", error);
    }
    
    if let Err(error) = remove_symlink(&current_bundles_path) {
      println!("could not remove current bundles symlink: {}", error);
    }

    // then we create the symlinks to the current modlist directories
    make_symlink(&current_mods_path, &self.mods_path())?;
    make_symlink(&current_dlc_path, &self.dlcs_path())?;
    make_symlink(&current_menu_path, &self.menus_path())?;
    make_symlink(&current_saves_path, &self.saves_path())?;
    make_symlink(&current_content_path, &self.content_path())?;
    make_symlink(&current_bundles_path, &self.bundles_path())?;

    // scriptermerger mergeinventory case:
    // special case to handle the scriptmerger mergeinventory.xml file.
    // because the tool uses a global database for the current state of the merge.
    // So whenever we swap modlists, we have to also swap the mergeinventory,
    // and for that we say that if the scripter merger is installed in the
    // `The Witcher 3/scriptmerger` directory and there is no merge inventory
    // we place a symlink to the new merge inventory. And if there is already a
    // symlink, we replace it.
    // And if there is a mergeinventory.xml file and not a symlink, we don't do anything
    // and let it fail. Because we don't want to override the current mergeinventory
    // of the user without asking him.
    let scriptmerger_path = std::env::current_dir().unwrap()
      .join(constants::SCRIPTMERGER_PATH);

    let modlist_mergeinventory_path = self.mergeinventory_path();

    // we do this operation only if the scriptmerger is installed at the right place
    // AND the modlist we're installing has a mergeinventory file
    if scriptmerger_path.exists() && modlist_mergeinventory_path.exists() {
      let scriptmerger_mergeinventory_path = scriptmerger_path.join(constants::MODLIST_MERGEINVENTORY_PATH);

      if let Err(error) = remove_symlink(&scriptmerger_mergeinventory_path) {
        // let it fail on purpose, just add a log for debugging
        println!("could not remove scriptmerger mergeinventory: {}", error)
      }

      make_symlink(&scriptmerger_mergeinventory_path, &modlist_mergeinventory_path)?;
    }
    
    Ok(())
  }

  pub fn is_packed(&self) -> bool {
    return self.packbackup_path().is_dir()
        && self.pack_path().is_dir();
  }

  pub fn pack(&self) -> std::io::Result<()> {
    let packbackup = self.packbackup_path();

    // return an error when there is no mergedfiles folder
    if !self.mergedfiles_path().exists() {
      let error = std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "no mergedfiles folder, nothing to pack"
      );

      return Err(error);
    }

    // first we start by removing the old pack backup if it already exists.
    if self.is_packed() {
      fs::remove_dir_all(&packbackup)?;
    }

    // then we create the directory.
    fs::create_dir_all(&packbackup)?;

    let pack_scripts_path = self.pack_path()
      .join("content")
      .join("scripts");

    // preparing the pack folder
    fs::create_dir_all(&pack_scripts_path)?;

    let modspath = self.mods_path();
    for mod_result in fs::read_dir(&modspath)? {
      if let Ok(modname) = mod_result {
        let modpath = modname.path();

        if modpath.file_name().unwrap().to_str().unwrap().starts_with("~") {
          continue;
        }

        let mod_scripts_path = &modspath
          .join(&modpath)
          .join("content")
          .join("scripts");

        println!("modpath: {:?}", &modpath);

        // nothing to do when the mod has no scripts folder
        if !mod_scripts_path.is_dir() {
          continue;
        }

        // then copy it in the pack folder
        let mut options = fs_extra::dir::CopyOptions::new();
        options.skip_exist = true;
        options.content_only = true;
        
        fs_extra::dir::copy(
          &mod_scripts_path,
          &pack_scripts_path,
          &options
        ).map_err(|_| std::io::ErrorKind::NotFound)?;

        // then we rename the script in the mod so that the scriptmerger won't
        // consider it a script mod if the modlist is imported by another modlist
        // and the user wants to merge the modlist.
        
      }
    }

    // and the final step is to manually drop the mergedfiles in the pack folder
    // and overwrite anything that already exist so it's considered a valid mod.
    //
    // Normally the mergedfiles scripts are already in the pack and because we use
    // `skip_exist = true` it should keep the mergedfiles scripts. But we can't be
    // sure the mergedfiles were the first to be loaded so it's safer to copy the
    // mergedfiles scripts again just to be sure but this time with `overwrite = true`
    let mut options = fs_extra::dir::CopyOptions::new();
    options.overwrite = true;
    options.content_only = true;

    let mergedfiles_scripts_path = self.mergedfiles_path()
      .join("content")
      .join("scripts");

    fs_extra::dir::copy(
      mergedfiles_scripts_path,
      self.pack_path()
            .join("content")
            .join("scripts"),
      &options
    ).map_err(|_| std::io::ErrorKind::NotFound)?;

    // once it's done we move the original mergedfiles folder in the packbackup
    // folder. So when the modlist is unpacked the mergedfiles folder is restored
    fs::rename(
      self.mergedfiles_path(),
      self.packbackup_path()
        .join(constants::SCRIPTMERGER_MERGEDFILES_FOLDERNAME)
    )?;

    Ok(())
  }

  // if the current modlist is packed, use the packbackup folder to restore the
  // original state of the mods (mergedfiles included)
  pub fn unpack(&self) -> std::io::Result<()> {
    if !self.is_packed() {
      return Ok(())
    }

    let packbackup = self.packbackup_path();
    let modspath = self.mods_path();
    for mod_result in fs::read_dir(&packbackup)? {
      if let Ok(modname) = mod_result {
        let modpath = modname.path();
        
        let backedup_mod_path = &packbackup
          .join(&modpath);
        
        let restored_mod_path = &modspath
          .join(&modpath);

        fs::create_dir_all(&restored_mod_path)?;

        let mut options = fs_extra::dir::CopyOptions::new();
        options.overwrite = true;
        options.content_only = true;

        
        fs_extra::dir::copy(
          &backedup_mod_path,
          &restored_mod_path,
          &options
        ).map_err(|_| std::io::ErrorKind::NotFound)?;
      }
    }

    // and once it's all restored we can remove the packbackup folder
    fs::remove_dir_all(&packbackup)?;

    Ok(())
  }

  /// returns all the modlists it can find in the current modlist database directory
  pub fn get_all() -> Vec<ModList> {
    let children = fs::read_dir(constants::MODLIST_DATABASE_PATH);

    if children.is_err() {
      return Vec::new();
    }

    let directories = children.unwrap()
      .filter(|child| child.is_ok())
      .map(|child| child.unwrap())
      .map(|entry| entry.file_name())
      .map(|filename| filename.into_string())
      .filter(|result| result.is_ok())
      .map(|result| result.unwrap());

    directories
    .map(|directory| ModList::new(directory))
    .filter(|modlist| modlist.is_valid())
    .collect()
  }

  pub fn get_by_name(name: &str) -> Option<ModList> {
    let database_path = Path::new(constants::MODLIST_DATABASE_PATH);
    let modlist_path = database_path.join(name);

    if !modlist_path.exists() {
      None
    }
    else {
      Some(ModList::new(name.to_owned()))
    }
  }

  pub fn create(name: &str) -> std::io::Result<ModList> {
    if let Some(modlist) = ModList::get_by_name(name) {
      return Ok(modlist);
    }

    let modlist = ModList::new(name.to_owned());

    fs::create_dir_all(modlist.dlcs_path())?;
    fs::create_dir_all(modlist.mods_path())?;
    fs::create_dir_all(modlist.menus_path())?;
    fs::create_dir_all(modlist.saves_path())?;
    fs::create_dir_all(modlist.content_path())?;
    fs::create_dir_all(modlist.bundles_path())?;

    let mergeinventory_content = "
      <?xml version=\"1.0\" encoding=\"utf-8\"?>
      <MergeInventory xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\">
        
      </MergeInventory>
    ".trim();

    fs::write(modlist.mergeinventory_path(), &mergeinventory_content)?;

    Ok(modlist)
  }
}