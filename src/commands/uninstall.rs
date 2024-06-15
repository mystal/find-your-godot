use anyhow::Result;

use crate::{
    commands::uninstall,
    dirs::FygDirs,
    version::GodotVersion,
};

pub fn cmd(version: &str, mono: bool) -> Result<()> {
    let version = GodotVersion::new(version, mono);
    let fyg_dirs = FygDirs::get();

    uninstall(fyg_dirs.engines_data(), &version)?;
    println!("Uninstalled version {}.", version);

    Ok(())
}
