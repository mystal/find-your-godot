use anyhow::Result;

use crate::cli::CliCommand;

mod cache;
mod edit;
mod install;
mod launch;
mod list;
mod uninstall;

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
