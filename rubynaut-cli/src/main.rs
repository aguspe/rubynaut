use clap::{Parser, Subcommand};
use console::style;

mod commands;

#[derive(Parser)]
#[command(
    name = "rubynaut",
    about = "Cross-platform Ruby version manager",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a Ruby version
    Install {
        /// Ruby version to install (e.g. 4.0.2, 3.3.6)
        version: String,
    },
    /// Uninstall a Ruby version
    Uninstall {
        /// Ruby version to remove
        version: String,
    },
    /// List Ruby versions
    List {
        /// Show all available versions (not just installed)
        #[arg(short, long)]
        available: bool,
    },
    /// Set the active Ruby version
    Use {
        /// Ruby version to activate
        version: String,
        /// Set for current directory only (writes .ruby-version)
        #[arg(short, long)]
        local: bool,
    },
    /// Show the currently active Ruby version
    Current,
    /// Manage gems
    #[command(subcommand)]
    Gems(GemsCommands),
    /// Run environment diagnostics
    Doctor {
        /// Auto-fix all fixable issues
        #[arg(short, long)]
        fix: bool,
    },
    /// Manage shell integration
    #[command(subcommand)]
    Shell(ShellCommands),
    /// Manage project tracking
    #[command(subcommand)]
    Project(ProjectCommands),
    /// Run bundle install for a project
    Bundle {
        /// Path to the project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },
    /// Show platform information
    Platform,
}

#[derive(Subcommand)]
enum GemsCommands {
    /// List gems for the active Ruby version
    List {
        /// Ruby version (defaults to active)
        #[arg(short, long)]
        version: Option<String>,
    },
    /// Install a gem
    Install {
        /// Gem name
        name: String,
        /// Gem version (optional, defaults to latest)
        #[arg(short, long)]
        version: Option<String>,
    },
    /// Uninstall a gem
    Uninstall {
        /// Gem name
        name: String,
    },
}

#[derive(Subcommand)]
enum ShellCommands {
    /// Install the shell hook
    Install,
    /// Show shell hook status
    Status,
    /// Print the hook script for a shell
    Hook {
        /// Shell name: bash, zsh, fish, powershell
        shell: String,
    },
}

#[derive(Subcommand)]
enum ProjectCommands {
    /// Scan a project for Ruby version requirements
    Scan {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },
    /// List tracked projects
    List,
    /// Add a project to tracking
    Add {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },
    /// Remove a project from tracking
    Remove {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Install { version } => commands::install(version).await,
        Commands::Uninstall { version } => commands::uninstall(version),
        Commands::List { available } => commands::list(available).await,
        Commands::Use { version, local } => commands::use_version(version, local),
        Commands::Current => commands::current(),
        Commands::Gems(sub) => match sub {
            GemsCommands::List { version } => commands::gems_list(version),
            GemsCommands::Install { name, version } => commands::gems_install(name, version).await,
            GemsCommands::Uninstall { name } => commands::gems_uninstall(name).await,
        },
        Commands::Doctor { fix } => commands::doctor(fix).await,
        Commands::Shell(sub) => match sub {
            ShellCommands::Install => commands::shell_install(),
            ShellCommands::Status => commands::shell_status(),
            ShellCommands::Hook { shell } => commands::shell_hook(shell),
        },
        Commands::Project(sub) => match sub {
            ProjectCommands::Scan { path } => commands::project_scan(path),
            ProjectCommands::List => commands::project_list(),
            ProjectCommands::Add { path } => commands::project_add(path),
            ProjectCommands::Remove { path } => commands::project_remove(path),
        },
        Commands::Bundle { path } => commands::bundle(path).await,
        Commands::Platform => commands::platform(),
    };

    if let Err(e) = result {
        eprintln!("{} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}
