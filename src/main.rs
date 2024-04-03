use anyhow::{bail, Result};

mod cli;
mod commands;
mod config;
mod dirs;
mod platform;
mod version;

#[tokio::main]
async fn main() -> Result<()> {
    use clap::Parser;

    let cli = cli::Cli::parse();

    // Initialize dirs and verify that it succeeded.
    if !dirs::FygDirs::get().is_valid() {
        bail!("Could not initialize app directories.");
    }

    commands::run_command(&cli.command).await?;

    Ok(())
}
