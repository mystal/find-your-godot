use std::fs;

use anyhow::Result;
use octocrab::models::repos::Release;
use owo_colors::OwoColorize;

use crate::{
    dirs::FygDirs,
    version::GodotVersion,
};

#[must_use]
fn is_installed(version: &GodotVersion) -> bool {
    let fyg_dirs = FygDirs::get();
    fyg_dirs.get_binary_path(version).is_file()
}

pub async fn cmd(available: bool) -> Result<()> {
    if available {
        list_available().await
    } else {
        list_installed()
    }
}

fn list_installed() -> Result<()> {
    let fyg_dirs = FygDirs::get();

    if !fyg_dirs.engines_data().is_dir() {
        // Engines directory doesn't exist, so no engines installed.
        return Ok(());
    }

    // Start by finding the installed versions.
    let read_dir = fs::read_dir(fyg_dirs.engines_data())?;
    // By default, list just the installed versions.
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
            if is_installed(&version) {
                println!("{}", &version);
            }
        }
    }

    Ok(())
}

async fn list_available() -> Result<()> {
    // Query GitHub for list of Godot Releases.
    let octocrab = octocrab::instance();
    let mut page = octocrab.repos("godotengine", "godot")
        .releases()
        .list()
        .per_page(100)
        .send()
        .await?;

    // List release versions.
    // TODO: Filter out/mark ones that don't support this platform.
    // TODO: Sort by version number.
    loop {
        // List versions on this page.
        for release in &page.items {
            // TODO: List both normal and Mono versions. Or add flag to just show mono versions?
            let release_version = GodotVersion::new(&release.tag_name, false);
            if is_installed(&release_version) {
                let installed = format!("{} (installed)", release_version);
                println!("{}", installed.bold());
            } else {
                println!("{}", release_version);
            }
        }

        // Try to get the next page, if any.
        page = match octocrab
            .get_page::<Release>(&page.next)
            .await?
        {
            Some(next_page) => next_page,
            None => break,
        }
    }

    Ok(())
}
