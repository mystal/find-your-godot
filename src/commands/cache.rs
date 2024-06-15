use std::fs;

use anyhow::Result;

use crate::{
    cli::CacheCommand,
    dirs::FygDirs,
    version::GodotVersion,
};

pub fn cmd(cache_command: &Option<CacheCommand>) -> Result<()> {
    let fyg_dirs = FygDirs::get();

    match cache_command {
        Some(CacheCommand::Show) | None => {
            if !fyg_dirs.engines_cache().is_dir() {
                // Engines cache directory doesn't exist.
                return Ok(());
            }

            let mut total_size = 0;

            // List cached engine versions.
            let read_dir = fs::read_dir(fyg_dirs.engines_cache())?;
            for entry in read_dir {
                let entry = entry?;
                let version_path = entry.path();
                if version_path.is_dir() {
                    let file_name = entry.file_name();
                    let full_version = file_name.to_string_lossy();
                    let (full_version, mono) = if let Some((v, _)) = full_version.split_once("_mono") {
                        (v, true)
                    } else {
                        (full_version.as_ref(), false)
                    };
                    let version = GodotVersion::new(full_version, mono);
                    let zip_path = fyg_dirs.get_cached_zip_path(&version);
                    if zip_path.is_file() {
                        let metadata = zip_path.metadata()?;
                        let byte_size = metadata.len();
                        let formatted_size = humansize::format_size(byte_size, humansize::DECIMAL);
                        println!("{} ({}): {}", &version, formatted_size, zip_path.display());

                        total_size += byte_size;
                    }
                }
            }

            // Print full size of all files in cache.
            let formatted_size = humansize::format_size(total_size, humansize::DECIMAL);
            println!("Total: {}", formatted_size);
        }
        Some(CacheCommand::Rm { all, mono, versions }) => {
            if *all {
                // TODO: Collect all dirs to be removed, print them, and confirm removal.
                let read_dir = fs::read_dir(fyg_dirs.engines_cache())?;
                for entry in read_dir {
                    let entry = entry?;
                    let version_path = entry.path();
                    println!("Removing {}", version_path.display());
                    fs::remove_dir_all(version_path)?;
                }
                return Ok(());
            }

            for version in versions {
                let version = GodotVersion::new(version.trim(), *mono);
                let zip_path = fyg_dirs.get_cached_zip_path(&version);
                let version_path = zip_path.parent().unwrap();
                if version_path.is_dir() {
                    println!("Removing {}", version_path.display());
                    fs::remove_dir_all(version_path)?;
                } else {
                    println!("Cache for version \"{}\" not found", version);
                }
            }
        }
    }

    Ok(())
}
