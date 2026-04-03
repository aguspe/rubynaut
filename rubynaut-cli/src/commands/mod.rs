use console::style;
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use rubynaut_core::{self, ProgressCallback};
use std::sync::{Arc, Mutex};

pub async fn install(version: String, from_file: Option<String>) -> Result<(), String> {
    println!(
        "{} {}...",
        style("Installing").green().bold(),
        style(&version).cyan()
    );

    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {percent}% {msg}")
            .unwrap()
            .progress_chars("=> "),
    );

    let pb_clone = pb.clone();
    let progress: ProgressCallback =
        Box::new(move |_stage: &str, percent: u8, message: &str| {
            pb_clone.set_position(percent as u64);
            pb_clone.set_message(message.to_string());
        });

    if let Some(archive_path) = from_file {
        rubynaut_core::install_ruby_from_archive(version.clone(), archive_path, Some(progress)).await?;
    } else {
        rubynaut_core::install_ruby(version.clone(), Some(progress)).await?;
    }

    pb.finish_with_message(format!("{} installed!", version));
    Ok(())
}

pub fn uninstall(version: String, yes: bool) -> Result<(), String> {
    if !yes {
        let confirmed = Confirm::new()
            .with_prompt(format!("Remove Ruby {} and all its gems? This cannot be undone", version))
            .default(false)
            .interact()
            .map_err(|e| format!("Prompt failed: {e}"))?;

        if !confirmed {
            println!("{}", style("Cancelled").yellow());
            return Ok(());
        }
    }

    rubynaut_core::uninstall_ruby(version.clone())?;
    println!(
        "{} Ruby {} removed",
        style("done:").green().bold(),
        style(&version).cyan()
    );
    Ok(())
}

pub async fn list(available: bool) -> Result<(), String> {
    if available {
        let versions = rubynaut_core::get_available_rubies().await?;
        println!("{}", style("Available Ruby versions:").bold());
        for v in &versions {
            let marker = if v.installed {
                style("*").green().to_string()
            } else {
                " ".to_string()
            };
            let ver = if v.active {
                style(&v.version).green().bold().to_string()
            } else if v.installed {
                style(&v.version).white().to_string()
            } else {
                style(&v.version).dim().to_string()
            };
            println!("  {} {}", marker, ver);
        }
        println!(
            "\n  {} = installed",
            style("*").green()
        );
    } else {
        let versions = rubynaut_core::get_installed_rubies()?;
        if versions.is_empty() {
            println!("No Ruby versions installed. Run: rubynaut install <version>");
            return Ok(());
        }
        let active = rubynaut_core::get_active_version()?.unwrap_or_default();
        println!("{}", style("Installed Ruby versions:").bold());
        for v in &versions {
            let marker = if v.version == active {
                style("=>").green().bold().to_string()
            } else {
                "  ".to_string()
            };
            let ver = if v.version == active {
                style(&v.version).green().bold().to_string()
            } else {
                style(&v.version).white().to_string()
            };
            let label = if v.version == active { " (active)" } else { "" };
            println!("  {} {}{}", marker, ver, style(label).dim());
        }
    }
    Ok(())
}

pub fn use_version(version: String, local: bool) -> Result<(), String> {
    if local {
        let cwd = std::env::current_dir()
            .map_err(|e| format!("Cannot get current directory: {e}"))?;
        rubynaut_core::set_local_version(cwd.to_string_lossy().to_string(), version.clone())?;
        println!(
            "{} Ruby {} set for {}",
            style("done:").green().bold(),
            style(&version).cyan(),
            style(cwd.display()).dim()
        );
    } else {
        rubynaut_core::set_global_version(version.clone())?;
        println!(
            "{} Ruby {} set as global default",
            style("done:").green().bold(),
            style(&version).cyan()
        );
    }
    Ok(())
}

pub fn current() -> Result<(), String> {
    match rubynaut_core::get_active_version()? {
        Some(v) => println!("{}", style(v).green().bold()),
        None => println!("{}", style("No active Ruby version").dim()),
    }
    Ok(())
}

pub fn gems_list(version: Option<String>) -> Result<(), String> {
    let ver = version
        .or_else(|| rubynaut_core::get_active_version().ok().flatten())
        .ok_or("No active Ruby version. Specify one with --version")?;

    let gems = rubynaut_core::get_gems_for_version(ver.clone())?;
    println!(
        "{} (Ruby {})",
        style("Gems").bold(),
        style(&ver).cyan()
    );

    let default_count = gems.iter().filter(|g| g.is_default).count();
    let user_count = gems.iter().filter(|g| !g.is_default).count();
    println!(
        "  {} default, {} user-installed\n",
        default_count, user_count
    );

    for gem in &gems {
        let badge = if gem.is_default {
            style("default").dim().to_string()
        } else {
            style("user").magenta().to_string()
        };
        println!(
            "  {} {} [{}]",
            style(&gem.name).white().bold(),
            style(&gem.version).dim(),
            badge
        );
    }
    Ok(())
}

pub async fn gems_install(name: String, version: Option<String>) -> Result<(), String> {
    let ruby_ver = rubynaut_core::get_active_version()?
        .ok_or("No active Ruby version")?;
    let display = match &version {
        Some(v) => format!("{name} {v}"),
        None => name.clone(),
    };
    println!(
        "{} {} (Ruby {})...",
        style("Installing").green().bold(),
        style(&display).cyan(),
        style(&ruby_ver).dim()
    );
    let result = rubynaut_core::install_gem(ruby_ver, name, version).await?;
    println!("{}", result);
    Ok(())
}

pub async fn gems_uninstall(name: String) -> Result<(), String> {
    let ruby_ver = rubynaut_core::get_active_version()?
        .ok_or("No active Ruby version")?;
    let result = rubynaut_core::uninstall_gem(ruby_ver, name).await?;
    println!("{}", result);
    Ok(())
}

pub async fn doctor(fix: bool) -> Result<(), String> {
    let results = rubynaut_core::run_doctor()?;

    println!("{}\n", style("Rubynaut Doctor").bold().underlined());

    for d in &results {
        let icon = match d.status {
            rubynaut_core::DiagnosticStatus::Ok => style("  OK").green(),
            rubynaut_core::DiagnosticStatus::Warning => style("WARN").yellow(),
            rubynaut_core::DiagnosticStatus::Error => style(" ERR").red(),
        };
        println!("  [{}] {}", icon, style(&d.name).bold());
        println!("         {}", style(&d.message).dim());

        if let Some(hint) = &d.fix_hint {
            println!("         {}", style(hint).italic().dim());
        }

        if fix {
            if let Some(cmd) = &d.fix_command {
                print!("         {} {}... ", style("fixing:").yellow(), cmd);
                match rubynaut_core::run_doctor_fix(cmd.clone()).await {
                    Ok(_) => println!("{}", style("done").green()),
                    Err(e) => println!("{} {}", style("failed:").red(), e),
                }
            }
        }
    }

    let ok_count = results
        .iter()
        .filter(|d| matches!(d.status, rubynaut_core::DiagnosticStatus::Ok))
        .count();
    let total = results.len();
    println!(
        "\n  {}/{} checks passed",
        style(ok_count).green().bold(),
        total
    );

    Ok(())
}

pub fn shell_install() -> Result<(), String> {
    let shell_name = detect_current_shell();
    let result = rubynaut_core::install_shell_hook(shell_name)?;
    println!("{} {}", style("done:").green().bold(), result);
    println!("  Restart your terminal to activate.");
    Ok(())
}

pub fn shell_status() -> Result<(), String> {
    let statuses = rubynaut_core::check_shell_hook()?;
    println!("{}\n", style("Shell Hook Status").bold());
    for s in &statuses {
        let icon = if s.installed {
            style("installed").green()
        } else {
            style("not installed").red()
        };
        println!(
            "  {} {} ({})",
            style(&s.shell).bold(),
            icon,
            style(&s.rc_file).dim()
        );
    }
    Ok(())
}

pub fn shell_hook(shell: String) -> Result<(), String> {
    let hook = rubynaut_core::get_shell_hook(shell)?;
    println!("{}", hook);
    Ok(())
}

pub fn project_scan(path: String) -> Result<(), String> {
    let abs_path = resolve_path(&path)?;
    let result = rubynaut_core::scan_project(abs_path)?;
    println!("{}", style(&result.project_name).bold());
    println!("  Path: {}", style(&result.path).dim());
    if let Some(v) = &result.detected_version {
        let source = result.source.as_deref().unwrap_or("?");
        println!(
            "  Ruby: {} (via {})",
            style(v).cyan().bold(),
            style(source).dim()
        );
        if result.version_installed {
            println!("  Status: {}", style("ready").green());
        } else {
            println!("  Status: {}", style("not installed").red());
            println!(
                "  Run: rubynaut install {}",
                v
            );
        }
    } else {
        println!("  Ruby: {}", style("no version specified").yellow());
    }
    if result.has_gemfile {
        println!("  Gemfile: {}", style("found").green());
    }
    Ok(())
}

pub fn project_list() -> Result<(), String> {
    let projects = rubynaut_core::get_tracked_projects()?;
    if projects.is_empty() {
        println!("No tracked projects. Run: rubynaut project add <path>");
        return Ok(());
    }
    println!("{}\n", style("Tracked Projects").bold());
    for p in &projects {
        let status = if !p.folder_exists {
            style("missing").red().to_string()
        } else if let Some(v) = &p.detected_version {
            if p.version_installed {
                format!("{} {}", style(v).cyan(), style("ready").green())
            } else {
                format!("{} {}", style(v).cyan(), style("not installed").red())
            }
        } else {
            style("no version").yellow().to_string()
        };
        let source = p
            .source
            .as_deref()
            .map(|s| format!(" via {}", style(s).dim()))
            .unwrap_or_default();
        println!(
            "  {} {} [{}]{}",
            style(&p.project_name).bold(),
            style(&p.path).dim(),
            status,
            source
        );
    }
    Ok(())
}

pub fn project_add(path: String) -> Result<(), String> {
    let abs_path = resolve_path(&path)?;
    rubynaut_core::add_tracked_project(abs_path.clone())?;
    println!(
        "{} {} added to tracked projects",
        style("done:").green().bold(),
        style(&abs_path).cyan()
    );
    Ok(())
}

pub fn project_remove(path: String) -> Result<(), String> {
    let abs_path = resolve_path(&path)?;
    rubynaut_core::remove_tracked_project(abs_path)?;
    println!(
        "{} project removed from tracking",
        style("done:").green().bold()
    );
    Ok(())
}

pub async fn bundle(path: String) -> Result<(), String> {
    let abs_path = resolve_path(&path)?;
    let scan = rubynaut_core::scan_project(abs_path.clone())?;
    let version = scan
        .detected_version
        .ok_or("No Ruby version detected for this project")?;

    if !scan.has_gemfile {
        return Err("No Gemfile found in this project".to_string());
    }

    println!(
        "{} bundle install (Ruby {})...",
        style("Running").green().bold(),
        style(&version).cyan()
    );

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let output = Arc::new(Mutex::new(String::new()));
    let output_clone = output.clone();
    let pb_clone = pb.clone();

    let progress: ProgressCallback =
        Box::new(move |_stage: &str, _percent: u8, message: &str| {
            pb_clone.set_message(message.to_string());
            let mut o = output_clone.lock().unwrap();
            o.push_str(message);
            o.push('\n');
        });

    let result = rubynaut_core::bundle_install(version, abs_path, Some(progress)).await;

    pb.finish_and_clear();

    match result {
        Ok(out) => {
            println!("{}", out);
            println!("{}", style("Bundle install completed.").green().bold());
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn platform() -> Result<(), String> {
    let info = rubynaut_core::detect_platform()?;
    println!("{}", style("Platform Info").bold());
    println!("  OS:              {}", style(&info.os).cyan());
    println!("  Architecture:    {}", style(&info.arch).cyan());
    println!(
        "  Package Manager: {}",
        style(info.package_manager.as_deref().unwrap_or("none")).cyan()
    );
    println!("  Shell:           {}", style(&info.shell).cyan());
    println!(
        "  WSL:             {}",
        if info.is_wsl {
            style("yes").yellow()
        } else {
            style("no").green()
        }
    );
    println!(
        "  C Compiler:      {}",
        if info.has_c_compiler {
            style("available").green()
        } else {
            style("not found").red()
        }
    );
    if !info.existing_ruby_managers.is_empty() {
        println!(
            "  Other Managers:  {}",
            style(info.existing_ruby_managers.join(", ")).yellow()
        );
    }
    Ok(())
}

fn detect_current_shell() -> String {
    std::env::var("SHELL")
        .unwrap_or_default()
        .rsplit('/')
        .next()
        .unwrap_or("bash")
        .to_string()
}

fn resolve_path(path: &str) -> Result<String, String> {
    let p = std::path::Path::new(path);
    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Cannot get current directory: {e}"))?
            .join(p)
    };
    Ok(abs.to_string_lossy().to_string())
}

pub fn exec(version: String, command: Vec<String>) -> Result<(), String> {
    let ruby_dir = rubynaut_core::rubies_dir().join(&version);
    let ruby_bin = ruby_dir.join("bin").join("ruby");
    if !ruby_bin.exists() {
        return Err(format!("Ruby {version} is not installed"));
    }

    let env = rubynaut_core::gem_env(&ruby_dir, &version);

    let program = &command[0];
    let args = &command[1..];

    let status = std::process::Command::new(program)
        .args(args)
        .envs(env)
        .status()
        .map_err(|e| format!("Failed to execute {program}: {e}"))?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

pub fn config_set_mirror(url: String) -> Result<(), String> {
    let mut config = rubynaut_core::read_config();
    config.mirror_url = Some(url.clone());
    rubynaut_core::write_config(&config)?;
    println!("{} Mirror URL set to {}", style("done:").green().bold(), style(&url).cyan());
    Ok(())
}

pub fn config_set_proxy(url: String) -> Result<(), String> {
    let mut config = rubynaut_core::read_config();
    config.http_proxy = Some(url.clone());
    rubynaut_core::write_config(&config)?;
    println!("{} HTTP proxy set to {}", style("done:").green().bold(), style(&url).cyan());
    Ok(())
}

pub fn config_clear_mirror() -> Result<(), String> {
    let mut config = rubynaut_core::read_config();
    config.mirror_url = None;
    rubynaut_core::write_config(&config)?;
    println!("{} Mirror URL cleared (using default)", style("done:").green().bold());
    Ok(())
}

pub fn config_clear_proxy() -> Result<(), String> {
    let mut config = rubynaut_core::read_config();
    config.http_proxy = None;
    rubynaut_core::write_config(&config)?;
    println!("{} HTTP proxy cleared", style("done:").green().bold());
    Ok(())
}

pub fn config_show() -> Result<(), String> {
    let config = rubynaut_core::read_config();
    println!("{}", style("Rubynaut Configuration").bold());
    println!("  Config file:  {}", style(rubynaut_core::config_path().display()).dim());
    println!("  Global Ruby:  {}", style(config.global_version.as_deref().unwrap_or("(none)")).cyan());
    println!("  Mirror URL:   {}", style(config.mirror_url.as_deref().unwrap_or("(default: github.com/ruby/ruby-builder)")).cyan());
    println!("  HTTP Proxy:   {}", style(config.http_proxy.as_deref().unwrap_or("(none)")).cyan());
    println!("  Projects:     {}", style(format!("{} tracked", config.projects.len())).cyan());
    Ok(())
}

pub fn which_cmd(command: String) -> Result<(), String> {
    let active = rubynaut_core::get_active_version()?
        .ok_or("No active Ruby version. Set one with: rubynaut use <version>")?;

    let ruby_dir = rubynaut_core::rubies_dir().join(&active);
    if !ruby_dir.exists() {
        return Err(format!("Ruby {active} directory not found"));
    }

    // Check in the Ruby bin directory
    let bin_path = ruby_dir.join("bin").join(&command);
    if bin_path.exists() {
        println!("{}", bin_path.display());
        return Ok(());
    }

    // Check in the gems bin directory
    let gems_bin = rubynaut_core::rubies_dir().join("gems").join(&active).join("bin").join(&command);
    if gems_bin.exists() {
        println!("{}", gems_bin.display());
        return Ok(());
    }

    Err(format!("{command} not found for Ruby {active}"))
}

pub fn alias_set(alias: String, version: String) -> Result<(), String> {
    rubynaut_core::set_alias(alias.clone(), version.clone())?;
    println!(
        "{} Alias {} → {}",
        style("done:").green().bold(),
        style(&alias).cyan(),
        style(&version).cyan()
    );
    Ok(())
}

pub fn alias_remove(alias: String) -> Result<(), String> {
    rubynaut_core::remove_alias(alias.clone())?;
    println!(
        "{} Alias '{}' removed",
        style("done:").green().bold(),
        style(&alias).cyan()
    );
    Ok(())
}

pub fn alias_list() -> Result<(), String> {
    let aliases = rubynaut_core::list_aliases();
    if aliases.is_empty() {
        println!("No aliases configured. Set one with: rubynaut alias set <name> <version>");
        return Ok(());
    }
    println!("{}", style("Version Aliases:").bold());
    let mut sorted: Vec<_> = aliases.iter().collect();
    sorted.sort_by_key(|(k, _)| (*k).clone());
    for (alias, version) in sorted {
        println!(
            "  {} → {}",
            style(alias).cyan(),
            style(version).white()
        );
    }
    Ok(())
}

pub async fn init(no_rails: bool, version_opt: Option<String>) -> Result<(), String> {
    println!("{}", style("Rubynaut Init — Setting up your Ruby environment").green().bold());
    println!();

    // Step 1: Determine version
    let version = match version_opt {
        Some(v) => v,
        None => {
            println!("  {} Detecting latest stable Ruby version...", style("[1/6]").dim());
            let v = rubynaut_core::get_latest_stable_version().await?;
            println!("       Latest stable: {}", style(&v).cyan().bold());
            v
        }
    };

    // Step 2: Install Ruby
    println!("  {} Installing Ruby {}...", style("[2/6]").dim(), style(&version).cyan());
    let ruby_bin = rubynaut_core::rubies_dir().join(&version).join("bin").join("ruby");
    if ruby_bin.exists() {
        println!("       {} Already installed", style("skipped:").yellow());
    } else {
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("       {spinner:.green} [{bar:30.cyan/blue}] {percent}% {msg}")
                .unwrap()
                .progress_chars("=> "),
        );
        let pb_clone = pb.clone();
        let progress: ProgressCallback =
            Box::new(move |_stage: &str, percent: u8, message: &str| {
                pb_clone.set_position(percent as u64);
                pb_clone.set_message(message.to_string());
            });
        rubynaut_core::install_ruby(version.clone(), Some(progress)).await?;
        pb.finish_and_clear();
        println!("       {} Installed", style("done:").green());
    }

    // Step 3: Set global default
    println!("  {} Setting global default...", style("[3/6]").dim());
    let current = rubynaut_core::get_active_version()?.unwrap_or_default();
    if current == version {
        println!("       {} Already set as global", style("skipped:").yellow());
    } else {
        rubynaut_core::set_global_version(version.clone())?;
        println!("       {} Global version set to {}", style("done:").green(), style(&version).cyan());
    }

    // Step 4: Install shell hook
    println!("  {} Setting up shell integration...", style("[4/6]").dim());
    let shell = detect_current_shell();
    let hooks = rubynaut_core::check_shell_hook()?;
    let already_installed = hooks.iter().any(|h| h.shell == shell && h.installed);
    if already_installed {
        println!("       {} Shell hook already installed for {}", style("skipped:").yellow(), shell);
    } else {
        match rubynaut_core::install_shell_hook(shell.clone()) {
            Ok(msg) => println!("       {} {}", style("done:").green(), msg),
            Err(e) => println!("       {} {} (run: rubynaut shell install)", style("warn:").yellow(), e),
        }
    }

    // Step 5: Write default-gems file
    println!("  {} Configuring default gems...", style("[5/6]").dim());
    let mut default_gems: Vec<(String, Option<String>)> = vec![
        ("bundler".to_string(), None),
    ];
    if !no_rails {
        default_gems.push(("rails".to_string(), None));
    }
    rubynaut_core::write_default_gems_file(&default_gems)?;
    let gem_names: Vec<&str> = default_gems.iter().map(|(n, _)| n.as_str()).collect();
    println!("       {} default-gems: {}", style("done:").green(), gem_names.join(", "));

    // Step 6: Install default gems
    println!("  {} Installing default gems...", style("[6/6]").dim());
    for (gem_name, gem_version) in &default_gems {
        print!("       {} {}... ", style(">").dim(), gem_name);
        match rubynaut_core::install_gem(version.clone(), gem_name.clone(), gem_version.clone()).await {
            Ok(_) => println!("{}", style("installed").green()),
            Err(e) => println!("{} ({})", style("failed").red(), e),
        }
    }

    // Summary
    println!();
    println!("  {}", style("Setup complete!").green().bold());
    println!();
    println!("  Installed:  Ruby {}", style(&version).cyan());
    println!("  Global:     {}", style(&version).cyan());
    println!("  Shell hook: {} {}", style(&shell).cyan(),
        if already_installed { "(was already installed)" } else { "(installed)" });
    println!("  Gems:       {}", style(gem_names.join(", ")).cyan());
    println!();
    println!("  {}", style("Next steps:").bold());
    println!("    ruby -e \"puts 'Hello, Ruby!'\"");
    if !no_rails {
        println!("    rails new my-app");
    }
    println!("    rubynaut list --available");
    println!();
    println!("  {} Restart your terminal to activate the shell hook.", style("Note:").yellow().bold());

    Ok(())
}

pub async fn update() -> Result<(), String> {
    let current = env!("CARGO_PKG_VERSION");
    let repo = "aguspe/rubynaut";

    println!(
        "{} Checking for updates (current: v{})...",
        style("update:").green().bold(),
        current
    );

    match rubynaut_core::check_for_update(current, repo).await? {
        None => {
            println!("{} Already up to date (v{})", style("done:").green().bold(), current);
            Ok(())
        }
        Some((tag, download_url)) => {
            println!(
                "  New version available: {}",
                style(&tag).cyan().bold()
            );

            if download_url.is_empty() {
                println!("  No prebuilt binary for this platform. Please update manually.");
                return Ok(());
            }

            let confirmed = Confirm::new()
                .with_prompt(format!("Update to {tag}?"))
                .default(true)
                .interact()
                .map_err(|e| format!("Prompt failed: {e}"))?;

            if !confirmed {
                println!("{}", style("Cancelled").yellow());
                return Ok(());
            }

            println!("  Downloading...");
            let result = rubynaut_core::self_update(&download_url).await?;
            println!("{} {}", style("done:").green().bold(), result);
            Ok(())
        }
    }
}
