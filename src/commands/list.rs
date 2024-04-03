use std::fs;

use anyhow::Result;
use octocrab::models::repos::Release;
use owo_colors::OwoColorize;

use crate::{
    commands::get_binary_name,
    dirs::FygDirs,
    version::get_full_version,
};

#[must_use]
fn is_installed(version: &str, fyg_dirs: &FygDirs) -> bool {
    let full_version = get_full_version(version);
    let bin_name = get_binary_name(&full_version);
    let bin_path = fyg_dirs.engines_data()
        .join(&full_version)
        .join(&bin_name);
    bin_path.is_file()
}

pub async fn cmd(available: bool) -> Result<()> {
    let fyg_dirs = FygDirs::get();

    if !available {
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
                let bin_name = get_binary_name(&full_version);
                let bin_path = fyg_dirs.engines_data()
                    .join(full_version.as_ref())
                    .join(bin_name);
                // TODO: Also check that it's executable?
                if bin_path.is_file() {
                    let version = full_version.strip_suffix("-stable")
                        .unwrap_or(&full_version);
                    println!("{}", &version);
                }
            }
        }

        return Ok(());
    }

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
    // TODO: Add option for ones with mono versions.
    // TODO: Sort by version number.
    loop {
        // List versions on this page.
        for release in &page.items {
            let release_version = release.tag_name.strip_suffix("-stable")
                .unwrap_or(&release.tag_name);
            if is_installed(release_version, &fyg_dirs) {
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
