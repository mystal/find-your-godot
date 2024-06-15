use std::{
    fs,
    io::Write,
};

use anyhow::{bail, Result};

use crate::{
    commands::uninstall::uninstall,
    dirs::FygDirs,
    version::GodotVersion,
};

pub async fn cmd(version: &str, mono: bool, force: bool) -> Result<()> {
    let version = GodotVersion::new(version, mono);
    let fyg_dirs = FygDirs::get();

    let bin_path = fyg_dirs.get_binary_path(&version);
    let zip_path = fyg_dirs.get_cached_zip_path(&version);

    if force {
        // Uninstall any existing version before installing.
        uninstall(&version)?;
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

        let data_dir = bin_path.parent().unwrap();
        let mut archive = zip::ZipArchive::new(zip_file)?;
        archive.extract(&data_dir)?;

        // By default, add an _sc_ file in the same directory to make Godot use Self-Contained Mode:
        // https://docs.godotengine.org/en/latest/tutorials/io/data_paths.html#self-contained-mode
        fs::File::create(data_dir.join("_sc_"))?;

        println!("Extracted to: {}", data_dir.to_string_lossy());

        return Ok(());
    }

    let full_version = version.get_full_version();

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

    let zip_name = version.get_zip_name();

    // Download package for this platform.
    let maybe_url = release.assets.iter()
        .find(|asset| asset.name == zip_name)
        .map(|asset| &asset.browser_download_url);
    dbg!(&zip_name);
    for asset in &release.assets {
        dbg!(&asset.name);
    }
    let Some(package_url) = maybe_url else {
        bail!(
            "Version {} does not support your platform.\nTuxFamily may have a build available: https://downloads.tuxfamily.org/godotengine/{}/",
            version,
            version,
        );
    };

    println!("Package URL: {}", package_url);

    // Download the file.
    let response = reqwest::get(package_url.as_str()).await?;
    let content = response.bytes().await?;

    // Copy content to cache directory for versions.
    let cache_dir = zip_path.parent().unwrap();
    fs::create_dir_all(&cache_dir)?;
    {
        let mut file = fs::File::create(&zip_path)?;
        file.write_all(&content)?;
    }

    // TODO: Check SHA512 sum of zip.

    println!("Downloaded to: {}", zip_path.to_string_lossy());

    // Unzip downloaded file to data dir under its version.
    // TODO: When unzipping a mono version, the contents contain a parent directory that needs to be
    // dealt wtih.
    let data_dir = bin_path.parent().unwrap();
    let seekable_content = std::io::Cursor::new(content.as_ref());
    let mut archive = zip::ZipArchive::new(seekable_content)?;
    archive.extract(&data_dir)?;

    // By default, add an _sc_ file in the same directory to make Godot use Self-Contained Mode:
    // https://docs.godotengine.org/en/latest/tutorials/io/data_paths.html#self-contained-mode
    fs::File::create(data_dir.join("_sc_"))?;

    println!("Extracted to: {}", data_dir.to_string_lossy());

    Ok(())
}
