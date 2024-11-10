use std::path::PathBuf;
use std::sync::LazyLock;

use clap::{Parser, Subcommand};

static VERSION: LazyLock<String> = LazyLock::new(||
    format!("{} ({})", clap::crate_version!(), env!("VERGEN_GIT_SHA"))
);

#[derive(Parser)]
#[command(version = VERSION.as_str(), about, arg_required_else_help(true))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Subcommand)]
pub enum CliCommand {
    /// List Godot engine versions. Shows installed versions by default.
    List {
        /// Show all Godot engine versions available on GitHub.
        #[arg(short, long)]
        available: bool,
    },

    /// Install the given Godot engine version.
    Install {
        /// Which version to install. e.g. "3.5.1"
        version: String,

        // Install the Mono version with C# support.
        // #[arg(long)]
        // mono: bool,

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

    /// Edit a Godot project with its associated Godot engine.
    Edit {
        /// Path to a project directory to edit. If none specified, try the current directory.
        project_dir: Option<PathBuf>,
    },

    /// Show or remove files from fyg's cache. Shows downloaded engine versions by default.
    Cache {
        #[command(subcommand)]
        cache_command: Option<CacheCommand>,
    },
}

#[derive(Debug, Subcommand)]
pub enum CacheCommand {
    /// Show downloaded engine versions in the cache.
    Show,

    /// Remove downloaded engine versions from the cache.
    Rm {
        /// Remove all downloaded engine versions.
        #[arg(short, long)]
        all: bool,

        /// Which downloaded engine versions to remove. e.g. "3.5.1 4.0.3"
        versions: Vec<String>,
    },
}
