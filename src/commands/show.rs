use anyhow::Result;

use crate::{
    config::ProjectGodotVersionConfig,
    dirs::FygDirs,
};

pub fn cmd() -> Result<()> {
    let project_config = ProjectGodotVersionConfig::load()?;

    // Check for project.godot in this directory.
    // if !Path::new(PROJECT_GODOT).is_file() {
    //     bail!("No project.godot file in this directory.");
    // }

    // let fyg_dirs = FygDirs::get();

    // Check that the project's Godot version is installed.
    // let full_version = get_full_version(&project_config.version);
    // let bin_name = get_binary_name(&full_version);
    // let bin_path = fyg_dirs.engines_data()
    //     .join(&full_version)
    //     .join(bin_name);
    // if !bin_path.is_file() {
    //     bail!("Can't edit project. Godot version {} is not installed.", &project_config.version);
    // }

    // TODO: Search for godot_version.toml and project.godot in parent directories.
    // TODO: Print info about this Godot project:
    // * Godot version from godot_version.toml
    // * project.godot stuff
    //   * project name
    //   * Icon??
    //   * Rendering method
    //   * Main scene?
    //   * Whether Godot version is installed
    println!("Godot version: {}", &project_config.version);

    Ok(())
}
