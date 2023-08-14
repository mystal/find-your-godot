use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{anyhow, bail, Context, Result};

use crate::{
    config::ProjectGodotVersionConfig,
    dirs::FygDirs,
    platform::Platform,
    version::GodotVersion,
};

mod cli;
mod config;
mod dirs;
mod platform;
mod version;

const PROJECT_GODOT: &str = "project.godot";

fn get_full_version(version: &GodotVersion) -> String {
    version.to_full_version()
}

fn get_binary_name(full_version: &str, platform: Platform) -> String {
    // TODO: The naming convention for binary/zip names seems to change a lot. To support all
    // versions, might be best to use a static list that we generate.
    let platform_suffix = if full_version.starts_with('4') {
        match platform {
            Platform::Windows32 => "win32.exe",
            Platform::Windows64 => "win64.exe",
            Platform::MacOS => "macos.universal",
            Platform::Linux32 => "linux.x86_32",
            Platform::Linux64 => "linux.x86_64",
            Platform::Unsupported => "unsupported",
        }
    } else {
        match platform {
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
fn uninstall(engines_data_dir: &Path, version: &GodotVersion) -> Result<()> {
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

#[must_use]
fn is_installed(version: &GodotVersion, platform: Platform, fyg_dirs: &FygDirs) -> bool {
    let full_version = get_full_version(version);
    let bin_name = get_binary_name(&full_version, platform);
    let bin_path = fyg_dirs.engines_data()
        .join(&full_version)
        .join(&bin_name);
    bin_path.is_file()
}

#[tokio::main]
async fn main() -> Result<()> {
    use clap::Parser;
    use cli::{CacheCommands, Commands};

    let cli = cli::Cli::parse();

    let platform = Platform::get();

    let fyg_dirs = FygDirs::new()
        .ok_or(anyhow!("Could not initialize app directories."))?;

    // Unwrap should always work since clap will return early if no command is passed in.
    // See cli::Cli, which sets arg_required_else_help(true).
    let command = cli.command.unwrap();

    match &command {
        Commands::List { available } => {
            use octocrab::models::repos::Release;

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
                        let bin_name = get_binary_name(&full_version, platform);
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
                    let release_version = GodotVersion::parse(&release.tag_name)?;
                    if is_installed(&release_version, platform, &fyg_dirs) {
                        // TODO: Bold this output like how rustup does for targets?
                        println!("{} (installed)", release_version);
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
        }
        Commands::Install { version, force } => {
            let version = GodotVersion::parse(version)?;
            let full_version = get_full_version(&version);
            let bin_name = get_binary_name(&full_version, platform);
            let bin_path = fyg_dirs.engines_data()
                .join(&full_version)
                .join(&bin_name);
            let zip_name = format!("{}.zip", &bin_name);
            let zip_path = fyg_dirs.engines_cache()
                .join(&full_version)
                .join(&zip_name);

            if *force {
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
        }
        Commands::Uninstall { version } => {
            let version = GodotVersion::parse(version)?;
            uninstall(fyg_dirs.engines_data(), &version)?;
            println!("Uninstalled version {}.", version);
        }
        Commands::Launch { version } => {
            // Try to launch the specified version.
            let version = GodotVersion::parse(version)?;
            let full_version = get_full_version(&version);
            let bin_name = get_binary_name(&full_version, platform);
            let bin_path = fyg_dirs.engines_data()
                .join(&full_version)
                .join(bin_name);

            if !bin_path.is_file() {
                bail!("Version {} is not installed.", version);
            }

            println!("Running: {}", bin_path.to_string_lossy());
            Command::new(&bin_path)
                .arg("--project-manager")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
        }
        Commands::Edit => {
            let project_config = ProjectGodotVersionConfig::load()?;

            // Check for project.godot in this directory.
            if !Path::new(PROJECT_GODOT).is_file() {
                bail!("No project.godot file in this directory.");
            }

            // Check that the project's Godot version is installed.
            let full_version = get_full_version(&project_config.version);
            let bin_name = get_binary_name(&full_version, platform);
            let bin_path = fyg_dirs.engines_data()
                .join(&full_version)
                .join(bin_name);
            if !bin_path.is_file() {
                bail!("Can't edit project. Godot version {} is not installed.", &project_config.version);
            }

            // Run Godot with the given project!!
            println!("Editing project with: {}", bin_path.to_string_lossy());
            Command::new(&bin_path)
                .arg("--editor")
                .arg(PROJECT_GODOT)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
        }
        Commands::Cache { cache_command } => {
            match cache_command {
                Some(CacheCommands::Show) | None => {
                    if !fyg_dirs.engines_cache().is_dir() {
                        // Engines cache directory doesn't exist.
                        return Ok(());
                    }

                    let mut total_size = 0;

                    // List cached engine versions.
                    let read_dir = fs::read_dir(fyg_dirs.engines_cache())?;
                    for entry in read_dir {
                        let entry = entry?;
                        let version_path = entry.path();
                        if version_path.is_dir() {
                            let file_name = entry.file_name();
                            let full_version = file_name.to_string_lossy();
                            let bin_name = get_binary_name(&full_version, platform);
                            let zip_name = format!("{}.zip", &bin_name);
                            let zip_path = version_path
                                .join(&zip_name);
                            if zip_path.is_file() {
                                let version = full_version.strip_suffix("-stable")
                                    .unwrap_or(&full_version);
                                let metadata = zip_path.metadata()?;
                                let byte_size = metadata.len();
                                let formatted_size = humansize::format_size(byte_size, humansize::DECIMAL);
                                println!("{} ({}): {}", &version, formatted_size, zip_path.display());

                                total_size += byte_size;
                            }
                        }
                    }

                    // Print full size of all files in cache.
                    let formatted_size = humansize::format_size(total_size, humansize::DECIMAL);
                    println!("Total: {}", formatted_size);
                }
                Some(CacheCommands::Rm { all, versions }) => {
                    if *all {
                        // TODO: Collect all dirs to be removed, print them, and confirm removal.
                        let read_dir = fs::read_dir(fyg_dirs.engines_cache())?;
                        for entry in read_dir {
                            let entry = entry?;
                            let version_path = entry.path();
                            println!("Removing {}", version_path.display());
                            fs::remove_dir_all(version_path)?;
                        }
                        return Ok(());
                    }

                    for version in versions {
                        let version = version.trim();
                        let version_path = fyg_dirs.engines_cache()
                            .join(format!("{}-stable", version));
                        if version_path.is_dir() {
                            println!("Removing {}", version_path.display());
                            fs::remove_dir_all(version_path)?;
                        } else {
                            println!("Cache for version \"{}\" not found", version);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
