use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use directories::BaseDirs;
use serde::Deserialize;

const FYG_DIR: &str = "find-your-godot";

#[derive(Parser)]
#[command(arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List Godot engine versions. By default shows only installed versions.
    List {
        /// Show all Godot engine versions installed and available.
        #[arg(short, long)]
        all: bool,
    },

    /// Install the given Godot engine version.
    Install {
        /// Which version to install. e.g. "3.5.1"
        version: String,

        /// Install the Mono version with C# support.
        #[arg(long)]
        mono: bool,

        /// Re-install if already installed.
        #[arg(short, long)]
        force: bool,
    },

    /// Uninstall the given Godot engine version.
    Uninstall {
        /// Which version to uninstall. e.g. "3.5.1"
        version: String,
    },

    /// Launch the given Godot engine version.
    Launch {
        /// Which version to launch. e.g. "3.5.1"
        version: String,
    },

    /// Open the Godot project in the current directory in its associated Godot engine.
    Open,
}

struct FygDirs {
    engines_data_dir: PathBuf,
    engines_cache_dir: PathBuf,
}

impl FygDirs {
    fn new() -> Option<Self> {
        let base_dirs = BaseDirs::new()?;

        let mut engines_cache_dir = base_dirs.cache_dir()
            .join(FYG_DIR);
        // Add an intermediate cache directory on Windows since it's placed in ~/AppData/Local
        // with other things by default.
        if cfg!(target_os = "windows") {
            engines_cache_dir.push("cache");
        }
        engines_cache_dir.push("engines");

        Some(Self {
            engines_data_dir: base_dirs.data_dir()
                .join(FYG_DIR)
                .join("engines"),
            engines_cache_dir,
        })
    }

    fn engines_data(&self) -> &Path {
        &self.engines_data_dir
    }

    fn engines_cache(&self) -> &Path {
        &self.engines_cache_dir
    }
}

#[derive(Debug, Deserialize)]
struct ProjectGodotVersionConfig {
    version: String,
}

#[derive(Clone, Copy, Debug)]
enum Platform {
    Windows32,
    Windows64,
    MacOS,
    Linux32,
    Linux64,
    Unsupported,
}

impl Platform {
    fn to_package(&self) -> &'static str {
        match self {
            Platform::Windows32 => "win32.exe",
            Platform::Windows64 => "win64.exe",
            Platform::MacOS => "osx.universal",
            Platform::Linux32 => "x11.32",
            Platform::Linux64 => "x11.64",
            Platform::Unsupported => "unsupported",
        }
    }
}

fn get_full_version(version: &str) -> String {
    // TODO: Use a more thorough heuristic.
    if version.contains('-') {
        version.to_string()
    } else {
        format!("{}-stable", version)
    }
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Compile time detection of platform we're running on.
    let platform = if cfg!(target_os = "windows") {
        if cfg!(target_arch = "x86") {
            Platform::Windows32
        } else if cfg!(target_arch = "x86_64") {
            Platform::Windows64
        } else {
            Platform::Unsupported
        }
    } else if cfg!(target_os = "macos") {
        Platform::MacOS
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "x86") {
            Platform::Linux32
        } else if cfg!(target_arch = "x86_64") {
            Platform::Linux64
        } else {
            Platform::Unsupported
        }
    } else {
        Platform::Unsupported
    };
    let fyg_dirs = FygDirs::new()
        .ok_or(anyhow!("Could not initialize app directories."))?;

    match &cli.command {
        Some(Commands::List { all }) => {
            if !all {
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
            let releases = octocrab.repos("godotengine", "godot")
                .releases()
                .list()
                .send()
                .await?;
            // List release versions.
            // TODO: Filter out/mark ones that don't support this platform.
            // TODO: Add option for ones with mono versions.
            // TODO: Sort by version number.
            for release in &releases.items {
                let release_version = release.tag_name.strip_suffix("-stable")
                    .unwrap_or(&release.tag_name);
                println!("{}", release_version);
            }
        }
        Some(Commands::Install { version, mono, force }) => {
            let full_version = get_full_version(version);
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
                uninstall(fyg_dirs.engines_data(), version)?;
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
            if let Ok(release) = maybe_release {
                // If found, download package for this platform.
                let maybe_url = release.assets.iter()
                    .find(|asset| asset.name == zip_name)
                    .map(|asset| &asset.browser_download_url);
                if let Some(package_url) = maybe_url {
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
                } else {
                    bail!("Version {} does not support your platform.", version);
                }
            } else {
                bail!("Version {} not found.", version);
                // TODO: Get list of releases and print available releases.
            }
        }
        Some(Commands::Uninstall { version }) => {
            uninstall(fyg_dirs.engines_data(), version)?;
            println!("Uninstalled version {}.", version);
        }
        Some(Commands::Launch { version }) => {
            // Try to launch the specified version.
            let full_version = get_full_version(version);
            let bin_name = get_binary_name(&full_version, platform);
            let bin_path = fyg_dirs.engines_data()
                .join(&full_version)
                .join(bin_name);
            if bin_path.is_file() {
                println!("Running: {}", bin_path.to_string_lossy());
                Command::new(&bin_path)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()?;
            } else {
                bail!("Version {} is not installed.", version);
            }
        }
        Some(Commands::Open) => {
            // TODO: check for project.godot and godot_version.toml
            if let Ok(project_config_str) = fs::read_to_string("godot_version.toml") {
                if let Ok(project_config) = toml::from_str::<ProjectGodotVersionConfig>(&project_config_str) {
                    // TODO: check that the version in godot_version.toml is installed.
                    let full_version = get_full_version(&project_config.version);
                    let bin_name = get_binary_name(&full_version, platform);
                    let bin_path = fyg_dirs.engines_data()
                        .join(&full_version)
                        .join(bin_name);
                    if bin_path.is_file() {
                        // Run Godot with the given project!!
                        println!("Opening project in: {}", bin_path.to_string_lossy());
                        Command::new(&bin_path)
                            .arg("project.godot")
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn()?;
                    } else {
                        bail!("Godot version {} is not installed.", &project_config.version);
                    }
                } else {
                    bail!("Could not parse godot_version.toml as valid TOML.");
                }
            } else {
                bail!("No godot_version.toml found in this directory.");
            }
        }
        None => {},
    }
    
    Ok(())
}
