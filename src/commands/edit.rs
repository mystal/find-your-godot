use std::{
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{bail, Result};

use crate::{
    config::ProjectGodotVersionConfig,
    dirs::FygDirs,
    version::GodotVersion,
};

const PROJECT_GODOT: &str = "project.godot";

pub fn cmd() -> Result<()> {
    let project_config = ProjectGodotVersionConfig::load()?;

    // Check for project.godot in this directory.
    if !Path::new(PROJECT_GODOT).is_file() {
        bail!("No project.godot file in this directory.");
    }

    let version = GodotVersion::new(&project_config.version, project_config.mono);
    let fyg_dirs = FygDirs::get();

    // Check that the project's Godot version is installed.
    let full_version = version.get_full_version();
    let bin_name = version.get_binary_name();
    let bin_path = fyg_dirs.engines_data()
        .join(&full_version)
        .join(bin_name);
    if !bin_path.is_file() {
        bail!("Can't edit project. Godot version {} is not installed.", &project_config.version);
    }

    // Run Godot with the given project!!
    println!("Editing project with: {}", bin_path.to_string_lossy());
    Command::new(&bin_path)
        .arg("--editor")
        .arg(PROJECT_GODOT)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}
