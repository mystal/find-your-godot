use std::{
    fs,
    path::Path,
};

use anyhow::{anyhow, Context, Result};

use crate::{
    cli::CliCommand,
    version::GodotVersion,
};

mod cache;
mod edit;
mod install;
mod launch;
mod list;
mod uninstall;

#[must_use]
fn uninstall(engines_data_dir: &Path, version: &GodotVersion) -> Result<()> {
    let full_version = version.get_full_version();
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
        CliCommand::List { available } => list::cmd(*available).await,
        CliCommand::Install { version, mono, force } => install::cmd(version, *mono, *force).await,
        CliCommand::Uninstall { version, mono } => uninstall::cmd(version, *mono),
        CliCommand::Launch { version, mono } => launch::cmd(version, *mono),
        CliCommand::Edit => edit::cmd(),
        CliCommand::Cache { cache_command } => cache::cmd(cache_command),
    }
}
