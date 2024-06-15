use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use directories::BaseDirs;

use crate::version::GodotVersion;

const FYG_DIR: &str = "find-your-godot";

pub struct FygDirs {
    engines_data_dir: PathBuf,
    engines_cache_dir: PathBuf,
}

impl FygDirs {
    pub fn get() -> &'static Self {
        static DIRS: OnceLock<FygDirs> = OnceLock::new();
        DIRS.get_or_init(|| Self::new())
    }

    pub fn new() -> Self {
        let Some(base_dirs) = BaseDirs::new() else {
            return Self {
                engines_data_dir: PathBuf::new(),
                engines_cache_dir: PathBuf::new(),
            }
        };

        let mut engines_cache_dir = base_dirs.cache_dir()
            .join(FYG_DIR);
        // Add an intermediate cache directory on Windows since it's placed in ~/AppData/Local
        // with other things by default.
        if cfg!(target_os = "windows") {
            engines_cache_dir.push("cache");
        }
        engines_cache_dir.push("engines");

        Self {
            engines_data_dir: base_dirs.data_dir()
                .join(FYG_DIR)
                .join("engines"),
            engines_cache_dir,
        }
    }

    pub fn engines_data(&self) -> &Path {
        &self.engines_data_dir
    }

    pub fn engines_cache(&self) -> &Path {
        &self.engines_cache_dir
    }

    pub fn is_valid(&self) -> bool {
        !self.engines_cache_dir.as_os_str().is_empty() &&
            !self.engines_data_dir.as_os_str().is_empty()
    }

    pub fn get_cached_zip_path(&self, version: &GodotVersion) -> PathBuf {
        let zip_name = version.get_zip_name();
        self.engines_cache()
            .join(version.get_full_version_with_flavor())
            .join(&zip_name)
    }

    pub fn get_binary_path(&self, version: &GodotVersion) -> PathBuf {
        self.engines_data()
            .join(version.get_full_version_with_flavor())
            .join(version.get_binary_name())
    }
}
