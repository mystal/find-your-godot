use std::path::{Path, PathBuf};

use directories::BaseDirs;

const FYG_DIR: &str = "find-your-godot";

pub struct FygDirs {
    engines_data_dir: PathBuf,
    engines_cache_dir: PathBuf,
}

impl FygDirs {
    pub fn new() -> Option<Self> {
        let base_dirs = BaseDirs::new()?;

        let mut engines_cache_dir = base_dirs.cache_dir()
            .join(FYG_DIR);
        // Add an intermediate cache directory on Windows since it's placed in ~/AppData/Local
        // with other things by default.
        if cfg!(target_os = "windows") {
            engines_cache_dir.push("cache");
        }
        engines_cache_dir.push("engines");

        Some(Self {
            engines_data_dir: base_dirs.data_dir()
                .join(FYG_DIR)
                .join("engines"),
            engines_cache_dir,
        })
    }

    pub fn engines_data(&self) -> &Path {
        &self.engines_data_dir
    }

    pub fn engines_cache(&self) -> &Path {
        &self.engines_cache_dir
    }
}
