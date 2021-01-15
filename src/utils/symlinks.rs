use std::fs;
use std::path::PathBuf;

/// loops through all children in the `source` directory and creates a smylink for
/// every child in the `destination` directory.
pub fn symlink_children(source: PathBuf, destination: PathBuf) -> std::io::Result<()> {
  let source_children = fs::read_dir(&source)?;

  for source_child_err in source_children {
    if let Ok(source_child_name) = source_child_err {
      // the path to the child in the other destination directory,
      // where the symlink will link to.
      let child_path = source_child_name.path();

      // the path where the symlink will be created.
      let imported_child_path = destination.join(source_child_name.file_name());

      let current_dir = std::env::current_dir()?;
      let absolute_from = current_dir.join(&imported_child_path);
      let absolute_to = current_dir.join(&child_path);

      make_symlink(&absolute_from, &absolute_to);
    }
  }

  println!("symlinks made");

  Ok(())
}

/// removes all symlinks in the directory
pub fn remove_symlinks(directory: &PathBuf) -> std::io::Result<()> {
  let children = fs::read_dir(&directory)?;

  for child_err in children {
    if let Ok(child) = child_err {
      remove_symlink(&child.path())?;
    }
  }

  Ok(())
}

/// `from` is where the symlink is placed.
///
/// `to` is where the symlink links to.
pub fn make_symlink(from: &PathBuf, to: &PathBuf) -> std::io::Result<()> {
  symlink::symlink_auto(to, from)?;

  Ok(())
}

pub fn remove_symlink(path: &PathBuf) -> std::io::Result<()> {
  // symlink::remove_symlink_auto(path)?;
  let symlink_path = fs::read_link(&path)?;

  // println!("removing symlink at {:?}", path);
  
  if symlink_path.exists() {
    println!("removing symlink");

    if symlink_path.is_dir() {
      fs::remove_dir(path)?;
    }
    else {
      fs::remove_file(path)?;
    }
  }

  Ok(())
}