use std::{
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{bail, Result};

use crate::{
    commands::get_binary_name,
    config::ProjectGodotVersionConfig,
    dirs::FygDirs,
    version::get_full_version,
};

const PROJECT_GODOT_NAME: &str = "project.godot";

pub fn cmd(project_dir: &Path) -> Result<()> {
    let project_config = ProjectGodotVersionConfig::load(project_dir)?;

    // Check for project.godot in this directory.
    let project_file_path = project_dir.join(PROJECT_GODOT_NAME);
    if !project_file_path.is_file() {
        bail!("No project.godot file in {}.", project_dir.display());
    }

    let fyg_dirs = FygDirs::get();

    // Check that the project's Godot version is installed.
    let full_version = get_full_version(&project_config.version);
    let bin_name = get_binary_name(&full_version);
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
        .arg(project_file_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}
