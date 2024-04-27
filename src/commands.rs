use std::{
    fs,
    path::Path,
};

use anyhow::{anyhow, Context, Result};

use crate::{
    cli::CliCommand,
    platform::{PLATFORM, Platform},
    version::get_full_version,
};

mod cache;
mod edit;
mod install;
mod launch;
mod list;
mod show;
mod uninstall;

pub fn get_binary_name(full_version: &str) -> String {
    // TODO: The naming convention for binary/zip names seems to change a lot. To support all
    // versions, might be best to use a static list that we generate.
    let platform_suffix = if full_version.starts_with('4') {
        match PLATFORM {
            Platform::Windows32 => "win32.exe",
            Platform::Windows64 => "win64.exe",
            Platform::MacOS => "macos.universal",
            Platform::Linux32 => "linux.x86_32",
            Platform::Linux64 => "linux.x86_64",
            Platform::Unsupported => "unsupported",
        }
    } else {
        match PLATFORM {
            Platform::Windows32 => "win32.exe",
            Platform::Windows64 => "win64.exe",
            Platform::MacOS => "osx.universal",
            Platform::Linux32 => "x11.32",
            Platform::Linux64 => "x11.64",
            Platform::Unsupported => "unsupported",
        }
    };
    format!("Godot_v{}_{}", &full_version, platform_suffix)
}

#[must_use]
fn uninstall(engines_data_dir: &Path, version: &str) -> Result<()> {
    let full_version = get_full_version(version);
    let engine_path = engines_data_dir
        .join(&full_version);
    if engine_path.is_dir() {
        fs::remove_dir_all(engine_path)?;
        return Ok(());
    }

    Err(anyhow!("Engine install dir \"{}\" does not exist.", engine_path.to_string_lossy()))
        .context(format!("Could not uninstall version {}.", version))
}

pub async fn run_command(command: &Option<CliCommand>) -> Result<()> {
    let Some(command) = command else {
        return Ok(());
    };

    match &command {
        CliCommand::Show => show::cmd(),
        CliCommand::List { available } => list::cmd(*available).await,
        CliCommand::Install { version, force } => install::cmd(version, *force).await,
        CliCommand::Uninstall { version } => uninstall::cmd(version),
        CliCommand::Launch { version } => launch::cmd(version),
        CliCommand::Edit => edit::cmd(),
        CliCommand::Cache { cache_command } => cache::cmd(cache_command),
    }
}
