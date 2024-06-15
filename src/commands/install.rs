use std::{
    fs,
    io::Write,
};

use anyhow::{bail, Result};

use crate::{
    commands::uninstall,
    dirs::FygDirs,
    version::GodotVersion,
};

pub async fn cmd(version: &str, mono: bool, force: bool) -> Result<()> {
    let version = GodotVersion::new(version, mono);
    let fyg_dirs = FygDirs::get();

    // TODO: get_full_version should return "{version}-{pre_release/stable}", not including "mono"
    // TODO: And make a different method to return the directory/binary prefix.

    let full_version = version.get_full_version();
    let bin_name = version.get_binary_name();
    let bin_path = fyg_dirs.engines_data()
        .join(&full_version)
        .join(&bin_name);
    let zip_name = format!("{}.zip", &bin_name);
    let zip_path = fyg_dirs.engines_cache()
        .join(&full_version)
        .join(&zip_name);

    if force {
        // Uninstall any existing version before installing.
        uninstall(fyg_dirs.engines_data(), &version)?;
    } else {
        // Check if we already have this version installed.
        if bin_path.is_file() {
            bail!("Version {} is already installed. Pass --force to re-install.", version);
        }
    }

    // Skip download if engine zip is cached.
    if zip_path.is_file() {
        // TODO: Check SHA512 sum of zip.

        println!("Version {} is already downloaded. Extracting from cache.", version);

        let zip_file = fs::File::open(&zip_path)?;

        let data_dir = fyg_dirs.engines_data()
            .join(&full_version);
        let mut archive = zip::ZipArchive::new(zip_file)?;
        archive.extract(&data_dir)?;

        // By default, add an _sc_ file in the same directory to make Godot use Self-Contained Mode:
        // https://docs.godotengine.org/en/latest/tutorials/io/data_paths.html#self-contained-mode
        fs::File::create(data_dir.join("_sc_"))?;

        println!("Extracted to: {}", data_dir.to_string_lossy());

        return Ok(());
    }

    // Try to get the URL for this release.
    let octocrab = octocrab::instance();
    let maybe_release = octocrab.repos("godotengine", "godot")
        .releases()
        .get_by_tag(&full_version)
        .await;

    let Ok(release) = maybe_release else {
        bail!("Version {} not found.", version);
        // TODO: Get list of releases and print available releases.
    };

    // Download package for this platform.
    let maybe_url = release.assets.iter()
        .find(|asset| asset.name == zip_name)
        .map(|asset| &asset.browser_download_url);
    let Some(package_url) = maybe_url else {
        bail!(
            "Version {} does not support your platform.\nTuxFamily may have a build available: https://downloads.tuxfamily.org/godotengine/{}/",
            version,
            version,
        );
    };

    println!("Package URL: {}", package_url);

    // Download the file.
    let response = reqwest::get(package_url.as_str())
        .await?;
    let content = response.bytes()
        .await?;

    // Copy content to cache directory for versions.
    let cache_dir = fyg_dirs.engines_cache()
        .join(&full_version);
    fs::create_dir_all(&cache_dir)?;
    let download_path = cache_dir.join(&zip_name);
    {
        let mut file = fs::File::create(&download_path)?;
        file.write_all(&content)?;
    }

    // TODO: Check SHA512 sum of zip.

    println!("Downloaded to: {}", download_path.to_string_lossy());

    // Unzip downloaded file to data dir under its version.
    let data_dir = fyg_dirs.engines_data()
        .join(&full_version);
    let seekable_content = std::io::Cursor::new(content.as_ref());
    let mut archive = zip::ZipArchive::new(seekable_content)?;
    archive.extract(&data_dir)?;

    // By default, add an _sc_ file in the same directory to make Godot use Self-Contained Mode:
    // https://docs.godotengine.org/en/latest/tutorials/io/data_paths.html#self-contained-mode
    fs::File::create(data_dir.join("_sc_"))?;

    println!("Extracted to: {}", data_dir.to_string_lossy());

    Ok(())
}
