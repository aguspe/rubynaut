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
    /// Install a Ruby version (supports CRuby, JRuby, TruffleRuby)
    Install {
        /// Ruby version to install (e.g. 4.0.2, jruby-9.4.9.0, truffleruby-24.1.1)
        version: String,
        /// Install from a local .tar.gz archive instead of downloading
        #[arg(long)]
        from_file: Option<String>,
    },
    /// Uninstall a Ruby version
    Uninstall {
        /// Ruby version to remove
        version: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
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
    /// Run a command with a specific Ruby version's environment
    Exec {
        /// Ruby version to use
        version: String,
        /// Command and arguments to run
        #[arg(trailing_var_arg = true, required = true)]
        command: Vec<String>,
    },
    /// Show the path to a command's executable for the active Ruby
    Which {
        /// Command name (e.g. ruby, gem, bundle)
        #[arg(default_value = "ruby")]
        command: String,
    },
    /// Configure Rubynaut settings
    #[command(subcommand)]
    Config(ConfigCommands),
    /// Manage version aliases (e.g. "4.0" → "4.0.2")
    #[command(subcommand)]
    Alias(AliasCommands),
    /// Check for and install Rubynaut updates
    Update,
    /// Set up Ruby for the first time (install, configure, and go)
    Init {
        /// Skip Rails installation
        #[arg(long)]
        no_rails: bool,
        /// Specific Ruby version to install (default: latest stable)
        #[arg(long)]
        version: Option<String>,
    },
    /// Reset and re-launch the Getting Started wizard in the GUI
    Wizard,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Set a custom mirror URL for ruby-builder downloads
    SetMirror {
        /// Mirror URL (e.g. https://my-mirror.example.com/ruby-builder)
        url: String,
    },
    /// Set an HTTP proxy for all network requests
    SetProxy {
        /// Proxy URL (e.g. http://proxy.corp:8080)
        url: String,
    },
    /// Clear the mirror URL (use default)
    ClearMirror,
    /// Clear the proxy setting
    ClearProxy,
    /// Show current configuration
    Show,
}

#[derive(Subcommand)]
enum AliasCommands {
    /// Set a version alias
    Set {
        /// Alias name (e.g. "4.0", "stable", "default")
        alias: String,
        /// Target version (e.g. "4.0.2")
        version: String,
    },
    /// Remove a version alias
    Remove {
        /// Alias name to remove
        alias: String,
    },
    /// List all aliases
    List,
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
        Commands::Install { version, from_file } => commands::install(version, from_file).await,
        Commands::Uninstall { version, yes } => commands::uninstall(version, yes),
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
        Commands::Exec { version, command } => commands::exec(version, command),
        Commands::Which { command } => commands::which_cmd(command),
        Commands::Config(sub) => match sub {
            ConfigCommands::SetMirror { url } => commands::config_set_mirror(url),
            ConfigCommands::SetProxy { url } => commands::config_set_proxy(url),
            ConfigCommands::ClearMirror => commands::config_clear_mirror(),
            ConfigCommands::ClearProxy => commands::config_clear_proxy(),
            ConfigCommands::Show => commands::config_show(),
        },
        Commands::Alias(sub) => match sub {
            AliasCommands::Set { alias, version } => commands::alias_set(alias, version),
            AliasCommands::Remove { alias } => commands::alias_remove(alias),
            AliasCommands::List => commands::alias_list(),
        },
        Commands::Update => commands::update().await,
        Commands::Init { no_rails, version } => commands::init(no_rails, version).await,
        Commands::Wizard => commands::wizard(),
    };

    if let Err(e) = result {
        eprintln!("{} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}
