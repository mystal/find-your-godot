use std::{fs, path::Path};

use anyhow::{Result, Context};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProjectGodotVersionConfig {
    pub version: String,
}

impl ProjectGodotVersionConfig {
    pub fn load(project_dir: &Path) -> Result<ProjectGodotVersionConfig> {
        let godot_version_path = project_dir.join("godot_version.toml");
        let project_config_str = fs::read_to_string(godot_version_path)
            .with_context(|| format!("No godot_version.toml found in {}.", project_dir.display()))?;
        toml::from_str::<Self>(&project_config_str)
            .context("Could not parse godot_version.toml as valid TOML.")
    }
}
