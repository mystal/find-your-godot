use std::process::{Command, Stdio};

use anyhow::{bail, Result};

use crate::{
    commands::get_binary_name,
    dirs::FygDirs,
    version::get_full_version,
};

pub fn cmd(version: &str) -> Result<()> {
    let fyg_dirs = FygDirs::get();

    // Try to launch the specified version.
    let full_version = get_full_version(version);
    let bin_name = get_binary_name(&full_version);
    let bin_path = fyg_dirs.engines_data()
        .join(&full_version)
        .join(bin_name);

    if !bin_path.is_file() {
        bail!("Version {} is not installed.", version);
    }

    println!("Running: {}", bin_path.to_string_lossy());
    Command::new(&bin_path)
        .arg("--project-manager")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}
