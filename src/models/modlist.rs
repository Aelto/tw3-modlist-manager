
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use toml;

use crate::constants;
use crate::utils::symlinks::{
  symlink_children,
  remove_symlinks,
  make_symlink,
  remove_symlink
};



#[derive(Deserialize, Serialize)]
pub struct ModListConfig {
  imports: Vec<String>
}

#[derive(Deserialize, Serialize)]
pub struct ImportedModlist {
  pub name: String,
  pub order: i64
}

pub struct ModList {
  pub name: String,
  
  /// it's a list of unique imported modlists. It's a `Vec` and not a `HashSet`
  /// because the ordering is important as it is used as the load order from top
  /// to bottom.
  pub imported_modlists: Vec<String>,
}

impl ModList {
  pub fn new(name: String) -> ModList {
    ModList {
      name,
      imported_modlists: Vec::new()
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
  pub fn read_imports_from_disk(&mut self) -> std::io::Result<()> {
    let config_path = self.config_path();

    if !config_path.exists() {
      return Ok(());
    }

    let text = fs::read_to_string(config_path)?;

    let toml_config: ModListConfig = toml::from_str(&text)?;

    for import in toml_config.imports {
      self.import_modlist(&import);
    }

    Ok(())
  }

  /// update the import list of the disk with the new data in memory
  pub fn write_imports_to_disk(&self) -> Result<(), String> {
    let config = ModListConfig {
      imports: (&self.imported_modlists)
        .into_iter()
        .map(String::from)
        .collect()
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

    Ok(())
  }

  /// load all imported modlists in the current modlist directories in the form
  /// of symlinks pointing to the other modlists' directories.
  pub fn load_imported_modlists(&mut self) -> std::io::Result<()> {
    self.read_imports_from_disk()?;

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

  pub fn config_path(&self) -> PathBuf {
    self.path()
      .join(constants::MODLIST_CONFIG_NAME)
  }

  pub fn mergeinventory_path(&self) -> PathBuf {
    self.path()
      .join(constants::MODLIST_MERGEINVENTORY_PATH)
  }

  pub fn is_valid(&self) -> bool {
    let dlcs_path = self.dlcs_path();
    let mods_path = self.mods_path();
    let menus_path = self.menus_path();
    let saves_path = self.saves_path();
    let content_path = self.content_path();

    self.path().exists()
    && dlcs_path.exists()
    && mods_path.exists()
    && menus_path.exists()
    && saves_path.exists()
    && content_path.exists()
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
    remove_symlink(&current_mods_path);
    remove_symlink(&current_dlc_path);
    remove_symlink(&current_menu_path);
    remove_symlink(&current_saves_path);
    remove_symlink(&current_content_path);

    // then we create the symlinks to the current modlist directories
    make_symlink(&current_mods_path, &self.mods_path())?;
    make_symlink(&current_dlc_path, &self.dlcs_path())?;
    make_symlink(&current_menu_path, &self.menus_path())?;
    make_symlink(&current_saves_path, &self.saves_path())?;
    make_symlink(&current_content_path, &self.content_path())?;

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
      let scriptmerger_mergeinventory_path = scriptmerger_path.join("MergeInventory.xml");

      remove_symlink(&scriptmerger_mergeinventory_path);
      make_symlink(&scriptmerger_mergeinventory_path, &modlist_mergeinventory_path)?;
    }
    
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

    Ok(modlist)
  }
}