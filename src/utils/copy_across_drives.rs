use std::path::PathBuf;

/// it's a function to copy a folder from drive A to drive B, or on the same drive too
/// it was created because the std::fs::copy function fails on different drives.
pub fn copy_across_drives(from: PathBuf, to: PathBuf) -> std::io::Result<()> {
  if cfg!(target_os = "linux") {
    std::fs::copy(from, to)?;
  } else if cfg!(target_os = "windows") {
    std::process::Command::new("cmd")
      .arg("/C")
      .arg("xcopy")
      .arg("/E")
      .arg("/I")
      .arg(from)
      .arg(to)
      .output()?;
  }

  Ok(())
}
