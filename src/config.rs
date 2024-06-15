use std::fs;

use anyhow::{Result, Context};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProjectGodotVersionConfig {
    pub version: String,
    #[serde(default)]
    pub mono: bool,
}

impl ProjectGodotVersionConfig {
    pub fn load() -> Result<ProjectGodotVersionConfig> {
        let project_config_str = fs::read_to_string("godot_version.toml")
            .context("No godot_version.toml found in this directory.")?;
        toml::from_str::<Self>(&project_config_str)
            .context("Could not parse godot_version.toml as valid TOML.")
    }
}
