use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, arg_required_else_help(true))]
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

        // Install the Mono flavor of Godot with C# support
        #[arg(short, long)]
        mono: bool,

        /// Re-install if already installed.
        #[arg(short, long)]
        force: bool,
    },

    /// Uninstall the given Godot engine version.
    Uninstall {
        /// Which version to uninstall. e.g. "3.5.1"
        version: String,

        /// Uninstall the mono flavor of Godot
        #[arg(short, long)]
        mono: bool,
    },

    /// Launch the given Godot engine version.
    Launch {
        /// Which version to launch. e.g. "3.5.1"
        version: String,

        /// Launch the mono flavor of Godot with C# support
        #[arg(short, long)]
        mono: bool,
    },

    /// Edit the Godot project in the current directory in its associated Godot engine.
    Edit,

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
        /// Remove all downloaded engine versions of all flavors.
        #[arg(short, long)]
        all: bool,

        /// Target Mono flavor of engine versions. Only used when not removing all
        #[arg(short, long)]
        mono: bool,

        /// Which downloaded engine versions to remove. e.g. "3.5.1 4.0.3"
        versions: Vec<String>,
    },
}
