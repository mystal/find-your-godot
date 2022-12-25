use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List available Godot engine versions.
    List,

    /// Install the given Godot engine version.
    Install,

    /// Uninstall the given Godot engine version.
    Uninstall,

    /// Launch the given Godot engine version.
    Launch,

    /// Open the Godot project in the current directory in its associated Godot engine.
    Open,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::List) => {
            println!("List");

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
        Some(Commands::Install) => {
            println!("Install");
        }
        Some(Commands::Uninstall) => {
            println!("Uninstall");
        }
        Some(Commands::Launch) => {
            println!("Launch");
        }
        Some(Commands::Open) => {
            println!("Open");
        }
        None => {},
    }
}
