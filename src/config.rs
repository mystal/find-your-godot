use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result, Context};
use serde::Deserialize;

static PROJECT_FYG_CONFIGS: &[&str] = &[
    "fyg.toml",
    "godot_version.toml",
];

#[derive(Debug, Deserialize)]
pub struct ProjectFygConfig {
    pub version: String,
    pub root: Option<PathBuf>,
}

impl ProjectFygConfig {
    /// Load the project's fyg TOML config file at `project_fyg_dir`, which is usually the root of
    /// the project's git directory.
    pub fn load(project_fyg_dir: &Path) -> Result<ProjectFygConfig> {
        for &config_name in PROJECT_FYG_CONFIGS {
            let project_fyg_config_path = project_fyg_dir.join(config_name);
            if !project_fyg_config_path.is_file() {
                continue;
            }
            let project_config_str = fs::read_to_string(&project_fyg_config_path)
                .with_context(|| format!("Could not read {}.", project_fyg_config_path.display()))?;
            return toml::from_str::<Self>(&project_config_str)
                .with_context(|| format!("Could not parse {} as valid TOML.", project_fyg_config_path.display()));
        }

        // TODO: Better error reporting. Collect errors from the for loop and report them here.
        bail!("No valid config file ({}) found in {}.", PROJECT_FYG_CONFIGS.join(", "), project_fyg_dir.display());
    }
}
