use std::{fs, io::Write, process::Command};

use clap::{Parser, Subcommand};

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
    },

    /// Uninstall the given Godot engine version.
    Uninstall,

    /// Launch the given Godot engine version.
    Launch {
        /// Which version to launch. e.g. "3.5.1"
        version: String,
    },

    /// Open the Godot project in the current directory in its associated Godot engine.
    Open,
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

#[tokio::main]
async fn main() {
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
    let proj_dirs = directories::ProjectDirs::from("me.gabem", "Gabriel Martinez", "Too Many Godots").unwrap();

    match &cli.command {
        Some(Commands::List { all }) => {
            if !all {
                // Start by finding the installed versions.
                let engines_dir = proj_dirs.data_dir()
                    .join("engines");
                let read_dir = fs::read_dir(&engines_dir).unwrap();
                // By default, list just the installed versions.
                for entry in read_dir {
                    let entry = entry.unwrap();
                    let version_path = entry.path();
                    if version_path.is_dir() {
                        let file_name = entry.file_name();
                        let full_version = file_name.to_string_lossy();
                        let bin_name = format!("Godot_v{}_{}", &full_version, platform.to_package());
                        let bin_path = proj_dirs.data_dir()
                            .join("engines")
                            .join(full_version.as_ref())
                            .join(bin_name);
                        // TODO: Also check that it's executable?
                        if bin_path.is_file() {
                            println!("{}", &full_version);
                        }
                    }
                }
                return;
            }

            // Query GitHub for list of Godot Releases.
            let octocrab = octocrab::instance();
            let releases = octocrab.repos("godotengine", "godot")
                .releases()
                .list()
                .send()
                .await
                .unwrap();
            // TODO: List release versions (for this platform?)
            for release in &releases.items {
                // TODO: Decide to use either name or tag_name.
                let name = release.name.as_deref()
                    .unwrap_or("None");
                println!("Release: {} [{}]", name, &release.tag_name);
                for asset in &release.assets {
                    println!("  Asset: {}", &asset.name);
                }
            }
        }
        Some(Commands::Install { version, mono }) => {
            let full_version = format!("{}-stable", version);
            let package_name = format!("Godot_v{}_{}.zip", &full_version, platform.to_package());

            // TODO: Check if we already have this version installed.

            // Try to get the URL for this release.
            let octocrab = octocrab::instance();
            let maybe_release = octocrab.repos("godotengine", "godot")
                .releases()
                .get_by_tag(&full_version)
                .await;
            if let Ok(release) = maybe_release {
                // TODO: If found, download package for this OS.
                let maybe_url = release.assets.iter()
                    .find(|asset| asset.name == package_name)
                    .map(|asset| &asset.browser_download_url);
                if let Some(package_url) = maybe_url {
                    println!("Package URL: {}", package_url);

                    // Download the file.
                    let response = reqwest::get(package_url.as_str())
                        .await
                        .unwrap();
                    let content = response.bytes()
                        .await
                        .unwrap();

                    // Copy content to cache directory for versions.
                    let cache_dir = proj_dirs.cache_dir()
                        .join("engines")
                        .join(&full_version);
                    fs::create_dir_all(&cache_dir).unwrap();
                    let download_path = cache_dir.join(&package_name);
                    {
                        let mut file = fs::File::create(&download_path).unwrap();
                        // std::io::copy(&mut content.as_ref(), &mut file).unwrap();
                        file.write_all(&content).unwrap();
                    }
                    println!("Downloaded to: {}", download_path.to_string_lossy());

                    // TODO: Check SHA512 sum.

                    // Unzip downloaded file to data dir under its version.
                    let data_dir = proj_dirs.data_dir()
                        .join("engines")
                        .join(&full_version);
                    let seekable_content = std::io::Cursor::new(&content);
                    let mut archive = zip::ZipArchive::new(seekable_content).unwrap();
                    archive.extract(&data_dir).unwrap();

                    // By default, add an _sc_ file in the same directory to make Godot use Self-Contained Mode:
                    // https://docs.godotengine.org/en/latest/tutorials/io/data_paths.html#self-contained-mode
                    {
                        fs::File::create(data_dir.join("_sc_")).unwrap();
                    }

                    println!("Extracted to: {}", data_dir.to_string_lossy());
                } else {
                    println!("Sorry, version \"{}\" does not support your platform.", version);
                }
            } else {
                // TODO: Handle Err cases.
                println!("Sorry, version \"{}\" not found.", version);
                // TODO: Get list of releases and print available releases.
            }
        }
        Some(Commands::Uninstall) => {
            println!("Uninstall");
        }
        Some(Commands::Launch { version }) => {
            // Try to launch the specified version.
            let full_version = format!("{}-stable", version);
            let bin_name = format!("Godot_v{}_{}", &full_version, platform.to_package());
            let bin_path = proj_dirs.data_dir()
                .join("engines")
                .join(&full_version)
                .join(bin_name);
            if bin_path.is_file() {
                // TODO: Add option to disconnect from terminal.
                println!("Running: {}", bin_path.to_string_lossy());
                Command::new(&bin_path)
                    .spawn()
                    .unwrap();
            } else {
                println!("Version {} is not installed.", version);
            }
        }
        Some(Commands::Open) => {
            println!("Open");
        }
        None => {},
    }
}
