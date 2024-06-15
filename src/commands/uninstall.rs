use std::fs;

use anyhow::{anyhow, Context, Result};

use crate::{
    dirs::FygDirs,
    version::GodotVersion,
};

pub fn cmd(version: &str, mono: bool) -> Result<()> {
    let version = GodotVersion::new(version, mono);

    uninstall(&version)?;
    println!("Uninstalled version {}.", version);

    Ok(())
}

#[must_use]
pub fn uninstall(version: &GodotVersion) -> Result<()> {
    let fyg_dirs = FygDirs::get();

    let binary_path = fyg_dirs.get_binary_path(version);
    let engine_path = binary_path.parent().unwrap();
    if engine_path.is_dir() {
        fs::remove_dir_all(engine_path)?;
        return Ok(());
    }

    // No need to error if the engine dir doesn't exist? We succeeded.
    Ok(())
    // Err(anyhow!("Engine install dir \"{}\" does not exist.", engine_path.to_string_lossy()))
    //     .context(format!("Could not uninstall version {}.", version))
}
