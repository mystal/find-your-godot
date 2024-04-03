use anyhow::Result;

use crate::{
    commands::uninstall,
    dirs::FygDirs,
};

pub fn cmd(version: &str) -> Result<()> {
    let fyg_dirs = FygDirs::get();

    uninstall(fyg_dirs.engines_data(), version)?;
    println!("Uninstalled version {}.", version);

    Ok(())
}
