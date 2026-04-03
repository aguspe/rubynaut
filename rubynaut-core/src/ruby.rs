use crate::types::*;
use serde::Deserialize;
use sha2::{Sha256, Digest};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn rubies_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".rubies")
}

pub fn config_path() -> PathBuf {
    rubies_dir().join("config.json")
}

pub fn read_config() -> RubynautConfig {
    config_path()
        .exists()
        .then(|| {
            fs::read_to_string(config_path())
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
        })
        .flatten()
        .unwrap_or_default()
}

pub fn write_config(config: &RubynautConfig) -> Result<(), String> {
    let dir = rubies_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create rubies dir: {e}"))?;
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    fs::write(config_path(), json).map_err(|e| format!("Failed to write config: {e}"))?;
    Ok(())
}

pub fn get_installed_rubies() -> Result<Vec<RubyVersion>, String> {
    let dir = rubies_dir();
    let config = read_config();
    let active_version = get_active_version_inner(&config);

    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut versions = Vec::new();
    let entries = fs::read_dir(&dir).map_err(|e| format!("Failed to read rubies dir: {e}"))?;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        // Skip non-version directories
        if !name.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            continue;
        }
        let ruby_bin = entry.path().join("bin").join("ruby");
        if ruby_bin.exists() {
            let is_active = active_version.as_deref() == Some(name.as_str());
            versions.push(RubyVersion {
                version: name.clone(),
                installed: true,
                active: is_active,
                path: Some(entry.path().to_string_lossy().to_string()),
                prebuilt_available: false,
            });
        }
    }

    versions.sort_by(|a, b| b.version.cmp(&a.version));
    Ok(versions)
}

fn get_active_version_inner(config: &RubynautConfig) -> Option<String> {
    // Check for .ruby-version in current dir, walking up
    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = Some(cwd.as_path());
        while let Some(d) = dir {
            let rv_file = d.join(".ruby-version");
            if rv_file.exists() {
                if let Ok(v) = fs::read_to_string(&rv_file) {
                    let v = v.trim().to_string();
                    if !v.is_empty() {
                        return Some(v);
                    }
                }
            }
            dir = d.parent();
        }
    }
    config.global_version.clone()
}

pub fn get_active_version() -> Result<Option<String>, String> {
    let config = read_config();
    Ok(get_active_version_inner(&config))
}

pub fn set_global_version(version: String) -> Result<(), String> {
    if !is_valid_version(&version) {
        return Err(format!("Invalid version format: {version}"));
    }
    let ruby_path = rubies_dir().join(&version).join("bin").join("ruby");
    if !ruby_path.exists() {
        return Err(format!("Ruby {version} is not installed"));
    }
    let mut config = read_config();
    config.global_version = Some(version);
    write_config(&config)
}

pub fn set_local_version(path: String, version: String) -> Result<(), String> {
    if !is_valid_version(&version) {
        return Err(format!("Invalid version format: {version}"));
    }
    let ruby_path = rubies_dir().join(&version).join("bin").join("ruby");
    if !ruby_path.exists() {
        return Err(format!("Ruby {version} is not installed"));
    }
    let rv_file = PathBuf::from(&path).join(".ruby-version");
    fs::write(&rv_file, format!("{version}\n"))
        .map_err(|e| format!("Failed to write .ruby-version: {e}"))?;
    Ok(())
}

pub fn uninstall_ruby(version: String) -> Result<(), String> {
    let version_dir = rubies_dir().join(&version);
    if !version_dir.exists() {
        return Err(format!("Ruby {version} is not installed"));
    }
    // Also remove gems
    let gems_dir = rubies_dir().join("gems").join(&version);

    fs::remove_dir_all(&version_dir)
        .map_err(|e| format!("Failed to remove Ruby {version}: {e}"))?;

    if gems_dir.exists() {
        let _ = fs::remove_dir_all(&gems_dir);
    }

    // If this was the global version, unset it
    let mut config = read_config();
    if config.global_version.as_deref() == Some(version.as_str()) {
        config.global_version = None;
        write_config(&config)?;
    }

    Ok(())
}

pub fn scan_project(path: String) -> Result<ProjectScanResult, String> {
    let project_dir = PathBuf::from(&path);
    if !project_dir.is_dir() {
        return Err(format!("{path} is not a directory"));
    }

    let project_name = project_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());

    let mut detected_version: Option<String> = None;
    let mut source: Option<String> = None;

    // Priority 1: .ruby-version file
    let rv_file = project_dir.join(".ruby-version");
    if rv_file.exists() {
        if let Ok(content) = fs::read_to_string(&rv_file) {
            let v = content.trim().to_string();
            if !v.is_empty() {
                detected_version = Some(v);
                source = Some(".ruby-version".to_string());
            }
        }
    }

    // Priority 2: .tool-versions file (asdf/mise format)
    if detected_version.is_none() {
        let tv_file = project_dir.join(".tool-versions");
        if tv_file.exists() {
            if let Ok(content) = fs::read_to_string(&tv_file) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("ruby ") {
                        let v = line.strip_prefix("ruby ").unwrap_or("").trim().to_string();
                        if !v.is_empty() {
                            detected_version = Some(v);
                            source = Some(".tool-versions".to_string());
                            break;
                        }
                    }
                }
            }
        }
    }

    // Priority 3: Gemfile ruby version constraint
    if detected_version.is_none() {
        let gemfile = project_dir.join("Gemfile");
        if gemfile.exists() {
            if let Ok(content) = fs::read_to_string(&gemfile) {
                for line in content.lines() {
                    let line = line.trim();
                    // Match: ruby "3.3.6" or ruby '3.3.6'
                    if line.starts_with("ruby ") && !line.starts_with("ruby_") {
                        let version_str = line
                            .strip_prefix("ruby ")
                            .unwrap_or("")
                            .trim()
                            .trim_matches(|c| c == '"' || c == '\'')
                            .to_string();
                        // Handle: ruby "~> 3.3.0" — extract the base version
                        let clean = version_str
                            .trim_start_matches("~>")
                            .trim_start_matches(">=")
                            .trim_start_matches("=")
                            .trim()
                            .to_string();
                        if !clean.is_empty()
                            && clean.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
                        {
                            detected_version = Some(clean);
                            source = Some("Gemfile".to_string());
                            break;
                        }
                    }
                }
            }
        }
    }

    // Check if the detected version is installed
    let version_installed = detected_version
        .as_ref()
        .map(|v| rubies_dir().join(v).join("bin").join("ruby").exists())
        .unwrap_or(false);

    let has_gemfile = project_dir.join("Gemfile").exists();
    let has_gemfile_lock = project_dir.join("Gemfile.lock").exists();

    Ok(ProjectScanResult {
        path,
        project_name,
        detected_version,
        source,
        version_installed,
        has_gemfile,
        has_gemfile_lock,
        folder_exists: true,
    })
}

/// Scan a project path, returning a result even if the folder doesn't exist.
/// Used by the tracked projects list to handle moved/deleted folders gracefully.
pub fn scan_project_safe(path: &str, name: &str) -> ProjectScanResult {
    let project_dir = PathBuf::from(path);
    if !project_dir.is_dir() {
        return ProjectScanResult {
            path: path.to_string(),
            project_name: name.to_string(),
            detected_version: None,
            source: None,
            version_installed: false,
            has_gemfile: false,
            has_gemfile_lock: false,
            folder_exists: false,
        };
    }
    scan_project(path.to_string()).unwrap_or(ProjectScanResult {
        path: path.to_string(),
        project_name: name.to_string(),
        detected_version: None,
        source: None,
        version_installed: false,
        has_gemfile: false,
        has_gemfile_lock: false,
        folder_exists: true,
    })
}

pub fn get_tracked_projects() -> Result<Vec<ProjectScanResult>, String> {
    let config = read_config();
    let results = config
        .projects
        .iter()
        .map(|p| scan_project_safe(&p.path, &p.name))
        .collect();
    Ok(results)
}

pub fn add_tracked_project(path: String) -> Result<Vec<ProjectScanResult>, String> {
    let project_dir = PathBuf::from(&path);
    let name = project_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());

    let mut config = read_config();

    // Normalize: trim trailing slash for dedup
    let normalized = path.trim_end_matches('/').to_string();

    // Don't add duplicates
    let already_tracked = config.projects.iter().any(|p| {
        p.path.trim_end_matches('/') == normalized
    });

    if !already_tracked {
        config.projects.push(TrackedProject {
            path: normalized,
            name,
        });
        write_config(&config)?;
    }

    get_tracked_projects()
}

pub fn remove_tracked_project(path: String) -> Result<Vec<ProjectScanResult>, String> {
    let mut config = read_config();
    let normalized = path.trim_end_matches('/');
    config.projects.retain(|p| p.path.trim_end_matches('/') != normalized);
    write_config(&config)?;
    get_tracked_projects()
}

pub fn get_project_gems(project_path: String) -> Result<Vec<ProjectGem>, String> {
    let lockfile = PathBuf::from(&project_path).join("Gemfile.lock");
    if !lockfile.exists() {
        return Err("No Gemfile.lock found in this project".to_string());
    }

    let content = fs::read_to_string(&lockfile)
        .map_err(|e| format!("Failed to read Gemfile.lock: {e}"))?;

    let mut gems = Vec::new();
    let mut in_specs = false;

    for line in content.lines() {
        // Look for the "  specs:" line after "GEM"
        if line.trim() == "specs:" {
            in_specs = true;
            continue;
        }
        // End of specs section: a line that doesn't start with spaces or is empty after specs
        if in_specs {
            if !line.starts_with(' ') && !line.is_empty() {
                in_specs = false;
                continue;
            }
            // Gem entries are indented with 4 spaces: "    gem_name (version)"
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Only top-level gems (4 spaces indent), skip sub-dependencies (6+ spaces)
            let indent = line.len() - line.trim_start().len();
            if indent == 4 {
                if let Some(paren_start) = trimmed.find(" (") {
                    let name = &trimmed[..paren_start];
                    let version = trimmed[paren_start + 2..].trim_end_matches(')');
                    gems.push(ProjectGem {
                        name: name.to_string(),
                        version: version.to_string(),
                    });
                }
            }
        }
    }

    gems.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(gems)
}

pub async fn bundle_install(
    ruby_version: String,
    project_path: String,
    on_progress: Option<ProgressCallback>,
) -> Result<String, String> {
    let ruby_dir = rubies_dir().join(&ruby_version);
    let ruby_bin = ruby_dir.join("bin").join("ruby");
    if !ruby_bin.exists() {
        return Err(format!("Ruby {ruby_version} is not installed"));
    }

    let project_dir = PathBuf::from(&project_path);
    if !project_dir.join("Gemfile").exists() {
        return Err("No Gemfile found in this project".to_string());
    }

    if let Some(cb) = &on_progress {
        cb("running", 0, "Running bundle install...");
    }

    let env = gem_env(&ruby_dir, &ruby_version);
    let output = Command::new(&ruby_bin)
        .args(["-S", "bundle", "install"])
        .current_dir(&project_dir)
        .envs(env)
        .output()
        .map_err(|e| format!("Failed to run bundle install: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        if let Some(cb) = &on_progress {
            cb("done", 100, stdout.trim());
        }
        Ok(stdout)
    } else {
        let msg = format!("{}\n{}", stderr.trim(), stdout.trim()).trim().to_string();
        if let Some(cb) = &on_progress {
            cb("error", 100, &msg);
        }
        Err(msg)
    }
}

pub fn get_gems_for_version(version: String) -> Result<Vec<GemInfo>, String> {
    let ruby_bin = rubies_dir().join(&version).join("bin").join("ruby");
    if !ruby_bin.exists() {
        return Err(format!("Ruby {version} is not installed"));
    }

    let mut gems = Vec::new();

    // Default gems bundled with this Ruby version
    let default_gem_dir = find_gem_spec_dir(&rubies_dir().join(&version), true);
    if let Some(dir) = &default_gem_dir {
        gems.extend(read_gemspecs(dir, true));
    }

    // User-installed gems
    let user_gem_dir = rubies_dir().join("gems").join(&version);
    let user_spec_dir = user_gem_dir.join("specifications");
    if user_spec_dir.exists() {
        gems.extend(read_gemspecs(&user_spec_dir, false));
    }

    gems.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(gems)
}

/// Build the environment variables needed to run gem commands for a specific Ruby version.
/// Sets RUBYLIB to fix the hardcoded load path from ruby-builder binaries.
pub fn gem_env(ruby_dir: &std::path::Path, ruby_version: &str) -> Vec<(String, String)> {
    let gems_dir = rubies_dir().join("gems").join(ruby_version);
    let bin_dir = ruby_dir.join("bin");
    // Build RUBYLIB: the standard library paths that Ruby would normally find via its compiled prefix
    let rubylib = build_rubylib(ruby_dir);

    let mut env = vec![
        ("GEM_HOME".to_string(), gems_dir.to_string_lossy().to_string()),
        ("GEM_PATH".to_string(), format!("{}:{}", gems_dir.display(), ruby_dir.join("lib/ruby/gems").display())),
        ("PATH".to_string(), format!("{}:{}:{}", bin_dir.display(), gems_dir.join("bin").display(), std::env::var("PATH").unwrap_or_default())),
    ];

    if !rubylib.is_empty() {
        env.push(("RUBYLIB".to_string(), rubylib));
    }

    // On macOS, ruby-builder binaries link against Homebrew libraries (libyaml, openssl, etc.)
    // Set DYLD_FALLBACK_LIBRARY_PATH so they can be found
    if cfg!(target_os = "macos") {
        let mut dyld_paths = Vec::new();
        // Homebrew lib paths
        for path in ["/opt/homebrew/lib", "/usr/local/lib"] {
            if std::path::Path::new(path).exists() {
                dyld_paths.push(path.to_string());
            }
        }
        // Ruby's own lib directory
        dyld_paths.push(ruby_dir.join("lib").to_string_lossy().to_string());
        // Existing fallback paths
        if let Ok(existing) = std::env::var("DYLD_FALLBACK_LIBRARY_PATH") {
            dyld_paths.push(existing);
        }
        env.push(("DYLD_FALLBACK_LIBRARY_PATH".to_string(), dyld_paths.join(":")));
    }

    // On Linux, ensure the Ruby lib dir is on LD_LIBRARY_PATH
    if cfg!(target_os = "linux") {
        let mut ld_paths = vec![ruby_dir.join("lib").to_string_lossy().to_string()];
        // Common system lib paths
        for path in ["/usr/lib/x86_64-linux-gnu", "/usr/lib64", "/usr/local/lib"] {
            if std::path::Path::new(path).exists() {
                ld_paths.push(path.to_string());
            }
        }
        if let Ok(existing) = std::env::var("LD_LIBRARY_PATH") {
            ld_paths.push(existing);
        }
        env.push(("LD_LIBRARY_PATH".to_string(), ld_paths.join(":")));
    }

    env
}

/// Build the RUBYLIB value by finding the actual lib/ruby/X.Y.Z directories.
pub fn build_rubylib(ruby_dir: &std::path::Path) -> String {
    let lib_ruby = ruby_dir.join("lib").join("ruby");
    let mut paths = Vec::new();

    if let Ok(entries) = fs::read_dir(&lib_ruby) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Version directories like "4.0.0", "3.3.0"
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                && name.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
            {
                let version_dir = entry.path();
                paths.push(version_dir.to_string_lossy().to_string());

                // Also add architecture-specific subdirectory
                if let Ok(sub_entries) = fs::read_dir(&version_dir) {
                    for sub in sub_entries.flatten() {
                        let sub_name = sub.file_name().to_string_lossy().to_string();
                        if sub.file_type().map(|t| t.is_dir()).unwrap_or(false)
                            && (sub_name.contains("darwin") || sub_name.contains("linux") || sub_name.contains("x86") || sub_name.contains("arm"))
                        {
                            paths.push(sub.path().to_string_lossy().to_string());
                        }
                    }
                }
            }
            // site_ruby, vendor_ruby
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                && (name == "site_ruby" || name == "vendor_ruby")
            {
                paths.push(entry.path().to_string_lossy().to_string());
            }
        }
    }

    paths.join(":")
}

pub async fn install_gem(ruby_version: String, gem_name: String, gem_version: Option<String>) -> Result<String, String> {
    let ruby_dir = rubies_dir().join(&ruby_version);
    let ruby_bin = ruby_dir.join("bin").join("ruby");
    if !ruby_bin.exists() {
        return Err(format!("Ruby {ruby_version} is not installed"));
    }

    let gems_dir = rubies_dir().join("gems").join(&ruby_version);
    let _ = fs::create_dir_all(&gems_dir);

    // Gem names are case-sensitive on rubygems.org — lowercase is the convention
    let gem_name_normalized = gem_name.trim().to_string();

    let mut args = vec![
        "-S".to_string(),
        "gem".to_string(),
        "install".to_string(),
        gem_name_normalized.clone(),
        "--no-document".to_string(),
    ];

    // Only pass --version if a non-empty version string was provided
    if let Some(ver) = gem_version.as_deref() {
        let ver = ver.trim();
        if !ver.is_empty() {
            args.push("--version".to_string());
            args.push(ver.to_string());
        }
    }

    let env = gem_env(&ruby_dir, &ruby_version);
    let output = Command::new(&ruby_bin)
        .args(&args)
        .envs(env)
        .output()
        .map_err(|e| format!("Failed to run gem install: {e}"))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Err(format!("{stderr}\n{stdout}").trim().to_string())
    }
}

pub async fn uninstall_gem(ruby_version: String, gem_name: String) -> Result<String, String> {
    let ruby_dir = rubies_dir().join(&ruby_version);
    let ruby_bin = ruby_dir.join("bin").join("ruby");
    if !ruby_bin.exists() {
        return Err(format!("Ruby {ruby_version} is not installed"));
    }

    let env = gem_env(&ruby_dir, &ruby_version);
    let output = Command::new(&ruby_bin)
        .args(["-S", "gem", "uninstall", &gem_name, "--executables", "--force"])
        .envs(env)
        .output()
        .map_err(|e| format!("Failed to run gem uninstall: {e}"))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(stderr)
    }
}

/// Find the specifications directory inside a Ruby installation.
/// default_gems: look in lib/ruby/gems/X.Y.0/specifications/default/
/// user gems: look in lib/ruby/gems/X.Y.0/specifications/
pub fn find_gem_spec_dir(ruby_root: &std::path::Path, default: bool) -> Option<PathBuf> {
    let gems_dir = ruby_root.join("lib").join("ruby").join("gems");
    if !gems_dir.exists() {
        return None;
    }
    // Find the X.Y.0 directory (e.g., 4.0.0, 3.3.0)
    let entries = fs::read_dir(&gems_dir).ok()?;
    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            let spec_dir = if default {
                entry.path().join("specifications").join("default")
            } else {
                entry.path().join("specifications")
            };
            if spec_dir.exists() {
                return Some(spec_dir);
            }
        }
    }
    None
}

pub fn read_gemspecs(dir: &std::path::Path, is_default: bool) -> Vec<GemInfo> {
    let mut gems = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".gemspec") {
                // Filename format: name-version.gemspec
                let base = name.trim_end_matches(".gemspec");
                if let Some(last_dash) = base.rfind('-') {
                    let gem_name = &base[..last_dash];
                    let gem_version = &base[last_dash + 1..];
                    gems.push(GemInfo {
                        name: gem_name.to_string(),
                        version: gem_version.to_string(),
                        is_default,
                    });
                }
            }
        }
    }
    gems
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

fn cache_path() -> PathBuf {
    rubies_dir().join("versions_cache.json")
}

pub fn read_cache() -> Option<VersionCache> {
    let path = cache_path();
    if !path.exists() {
        return None;
    }
    let data = fs::read_to_string(&path).ok()?;
    let cache: VersionCache = serde_json::from_str(&data).ok()?;
    // Cache valid for 1 hour
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now - cache.fetched_at < 3600 {
        Some(cache)
    } else {
        None
    }
}

pub fn write_cache(versions: &[String]) {
    let _ = fs::create_dir_all(rubies_dir());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let cache = VersionCache {
        versions: versions.to_vec(),
        fetched_at: now,
    };
    if let Ok(json) = serde_json::to_string(&cache) {
        let _ = fs::write(cache_path(), json);
    }
}

/// Fallback list when GitHub API is unavailable (rate-limited, offline, etc.)
pub fn fallback_versions() -> Vec<String> {
    [
        "4.0.2", "4.0.1", "4.0.0",
        "3.4.9", "3.4.8", "3.4.7", "3.4.6", "3.4.5",
        "3.4.4", "3.4.3", "3.4.2", "3.4.1", "3.4.0",
        "3.3.11", "3.3.10", "3.3.9", "3.3.8", "3.3.7",
        "3.3.6", "3.3.5", "3.3.4", "3.3.3", "3.3.2", "3.3.1", "3.3.0",
        "3.2.11", "3.2.10", "3.2.9", "3.2.8", "3.2.7", "3.2.6",
        "3.2.5", "3.2.4", "3.2.3", "3.2.2", "3.2.1", "3.2.0",
        "3.1.7", "3.1.6", "3.1.5", "3.1.4", "3.1.3", "3.1.2", "3.1.1", "3.1.0",
        "3.0.7", "3.0.6", "3.0.5", "3.0.4", "3.0.3", "3.0.2", "3.0.1", "3.0.0",
        "2.7.8", "2.7.7", "2.7.6", "2.7.5", "2.7.4", "2.7.3", "2.7.2", "2.7.1", "2.7.0",
        "2.6.10", "2.6.9", "2.6.8", "2.6.7", "2.6.6", "2.6.5", "2.6.4", "2.6.3", "2.6.2", "2.6.1", "2.6.0",
        "2.5.9", "2.5.8", "2.5.7", "2.5.6", "2.5.5", "2.5.4", "2.5.3", "2.5.2", "2.5.1", "2.5.0",
        "2.4.10", "2.4.9", "2.4.7", "2.4.6", "2.4.5", "2.4.4", "2.4.3", "2.4.2", "2.4.1", "2.4.0",
        "2.3.8", "2.3.7", "2.3.6", "2.3.5", "2.3.4", "2.3.3", "2.3.2", "2.3.1", "2.3.0",
        "2.2.10", "2.1.9", "2.0.0-p648", "1.9.3-p551",
    ].iter().map(|s| s.to_string()).collect()
}

pub async fn fetch_versions_from_github() -> Result<Vec<String>, String> {
    let client = build_http_client()?;
    let mut all_versions: Vec<String> = Vec::new();
    let mut page = 1u32;

    // Engine prefixes to look for in release tags
    let engine_prefixes: &[(&str, &str)] = &[
        ("ruby-", ""),                       // CRuby: "ruby-4.0.2" → "4.0.2"
        ("jruby-", "jruby-"),                // JRuby: "jruby-9.4.9.0" → "jruby-9.4.9.0"
        ("truffleruby+graalvm-", "truffleruby+graalvm-"), // TruffleRuby+GraalVM
        ("truffleruby-", "truffleruby-"),    // TruffleRuby
    ];

    loop {
        let url = format!(
            "https://api.github.com/repos/ruby/ruby-builder/releases?per_page=100&page={page}"
        );
        let response = client
            .get(&url)
            .header("User-Agent", "Rubynaut")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch releases: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API returned {}", response.status()));
        }

        let releases: Vec<GitHubRelease> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse releases: {e}"))?;

        if releases.is_empty() {
            break;
        }

        for release in &releases {
            let tag = &release.tag_name;
            for &(prefix, output_prefix) in engine_prefixes {
                if let Some(base_version) = tag.strip_prefix(prefix) {
                    let is_stable = base_version
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                        && !base_version.contains("preview")
                        && !base_version.contains("-rc");
                    if is_stable {
                        let full_version = format!("{output_prefix}{base_version}");
                        all_versions.push(full_version);
                    }
                    break; // Only match the first prefix
                }
            }
        }

        page += 1;
    }

    all_versions.sort_by(|a, b| {
        let (ea, va, _) = parse_engine(a);
        let (eb, vb, _) = parse_engine(b);
        ea.cmp(eb).then_with(|| version_cmp(vb, va))
    });
    Ok(all_versions)
}

pub async fn get_available_rubies() -> Result<Vec<RubyVersion>, String> {
    let installed = get_installed_rubies().unwrap_or_default();
    let installed_versions: Vec<String> = installed.iter().map(|r| r.version.clone()).collect();

    // Try: cached → GitHub API → hardcoded fallback
    let all_versions = if let Some(cache) = read_cache() {
        cache.versions
    } else {
        match fetch_versions_from_github().await {
            Ok(versions) => {
                write_cache(&versions);
                versions
            }
            Err(_) => {
                // API unavailable (rate-limited, offline) — use fallback
                fallback_versions()
            }
        }
    };

    let versions = all_versions
        .into_iter()
        .map(|v| {
            let is_installed = installed_versions.iter().any(|i| i == &v);
            RubyVersion {
                version: v.clone(),
                installed: is_installed,
                active: installed.iter().any(|i| i.version == v && i.active),
                path: if is_installed {
                    Some(rubies_dir().join(&v).to_string_lossy().to_string())
                } else {
                    None
                },
                prebuilt_available: true,
            }
        })
        .collect();

    Ok(versions)
}

/// Compare version strings like "4.0.2" > "3.4.9" > "3.3.11"
pub fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |s: &str| -> Vec<u64> {
        // Handle versions like "2.0.0-p648" — strip the -pXXX suffix
        let base = s.split('-').next().unwrap_or(s);
        base.split('.').filter_map(|p| p.parse().ok()).collect()
    };
    let va = parse(a);
    let vb = parse(b);
    va.cmp(&vb)
}

/// Validate that a version string is safe for use in URLs and filesystem paths.
/// Accepts formats like "4.0.2", "3.3.6", "2.0.0-p648", "1.9.3-p551",
/// "jruby-9.4.9.0", "truffleruby-24.1.1", "truffleruby+graalvm-24.1.1".
pub fn is_valid_version(version: &str) -> bool {
    if version.is_empty() || version.len() > 40 {
        return false;
    }
    // Strip known engine prefixes for validation
    let numeric_part = version
        .strip_prefix("jruby-")
        .or_else(|| version.strip_prefix("truffleruby+graalvm-"))
        .or_else(|| version.strip_prefix("truffleruby-"))
        .unwrap_or(version);

    if numeric_part.is_empty() {
        return false;
    }
    // Numeric part must start with a digit
    if !numeric_part.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return false;
    }
    // Only allow digits, dots, and -p suffix in the numeric part
    numeric_part.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == 'p')
        && !numeric_part.contains("..")
        && !numeric_part.contains("--")
}

/// Determine the engine type and base version from a version string.
/// Returns (engine_prefix, version, tag_prefix) for URL construction.
pub fn parse_engine(version: &str) -> (&str, &str, &str) {
    if let Some(v) = version.strip_prefix("jruby-") {
        ("jruby", v, "jruby")
    } else if let Some(v) = version.strip_prefix("truffleruby+graalvm-") {
        ("truffleruby+graalvm", v, "truffleruby+graalvm")
    } else if let Some(v) = version.strip_prefix("truffleruby-") {
        ("truffleruby", v, "truffleruby")
    } else {
        ("ruby", version, "ruby")
    }
}

/// Build a reqwest::Client respecting proxy configuration from config.
pub fn build_http_client() -> Result<reqwest::Client, String> {
    let config = read_config();
    let mut builder = reqwest::Client::builder();
    if let Some(proxy_url) = &config.http_proxy {
        if !proxy_url.is_empty() {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| format!("Invalid proxy URL: {e}"))?;
            builder = builder.proxy(proxy);
        }
    }
    builder.build().map_err(|e| format!("Failed to build HTTP client: {e}"))
}

/// Get the base download URL, respecting mirror_url from config.
/// Default: "https://github.com/ruby/ruby-builder"
pub fn download_base_url() -> String {
    let config = read_config();
    config.mirror_url
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| "https://github.com/ruby/ruby-builder".to_string())
}

/// Install a Ruby version from a local .tar.gz archive file.
/// Skips download and checksum — goes straight to extract, fix paths, verify.
pub async fn install_ruby_from_archive(
    version: String,
    archive_path: String,
    on_progress: Option<ProgressCallback>,
) -> Result<(), String> {
    if !is_valid_version(&version) {
        return Err(format!("Invalid version format: {version}"));
    }

    let archive = PathBuf::from(&archive_path);
    if !archive.exists() {
        return Err(format!("Archive not found: {archive_path}"));
    }
    if !archive_path.ends_with(".tar.gz") && !archive_path.ends_with(".tgz") {
        return Err("Archive must be a .tar.gz or .tgz file".to_string());
    }

    let target_dir = rubies_dir().join(&version);
    if target_dir.join("bin").join("ruby").exists() {
        return Err(format!("Ruby {version} is already installed"));
    }

    fs::create_dir_all(&rubies_dir())
        .map_err(|e| format!("Failed to create rubies directory: {e}"))?;

    if let Some(cb) = &on_progress {
        cb("extract", 20, "Extracting archive...");
    }

    let tmp_dir = rubies_dir().join(".tmp-install");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir)
        .map_err(|e| format!("Failed to create temp dir: {e}"))?;

    let extract_output = Command::new("tar")
        .args(["xzf", &archive_path, "-C", &tmp_dir.to_string_lossy()])
        .output()
        .map_err(|e| format!("Extraction failed: {e}"))?;

    if !extract_output.status.success() {
        let stderr = String::from_utf8_lossy(&extract_output.stderr);
        let _ = fs::remove_dir_all(&tmp_dir);
        return Err(format!("Extraction failed: {stderr}"));
    }

    if let Some(cb) = &on_progress {
        cb("extract", 50, "Organizing files...");
    }

    let bin_dir = find_ruby_bin_dir(&tmp_dir)
        .ok_or("Could not find ruby binary in extracted archive")?;
    let ruby_root = bin_dir.parent().ok_or("Unexpected archive structure")?;

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)
            .map_err(|e| format!("Failed to clean target: {e}"))?;
    }

    fs::rename(ruby_root, &target_dir).or_else(|_| {
        copy_dir_recursive(ruby_root, &target_dir)
    }).map_err(|e| format!("Failed to move Ruby to final location: {e}"))?;

    let _ = fs::remove_dir_all(&tmp_dir);
    let gems_dir = rubies_dir().join("gems").join(&version);
    let _ = fs::create_dir_all(&gems_dir);

    if cfg!(target_os = "macos") {
        if let Some(cb) = &on_progress {
            cb("verify", 70, "Fixing library paths...");
        }
        fix_macos_dylib_paths(&target_dir, &version)?;
    }
    if cfg!(target_os = "linux") {
        if let Some(cb) = &on_progress {
            cb("verify", 70, "Fixing library paths...");
        }
        fix_linux_rpath(&target_dir)?;
    }
    fix_shebangs(&target_dir)?;

    if let Some(cb) = &on_progress {
        cb("verify", 85, "Verifying installation...");
    }

    let ruby_bin = target_dir.join("bin").join("ruby");
    let verify = Command::new(&ruby_bin)
        .arg("--version")
        .output()
        .map_err(|e| format!("Verification failed: {e}"))?;

    if !verify.status.success() {
        let stderr = String::from_utf8_lossy(&verify.stderr);
        let _ = fs::remove_dir_all(&target_dir);
        return Err(format!("Ruby binary failed verification: {stderr}"));
    }

    let ruby_version_output = String::from_utf8_lossy(&verify.stdout).trim().to_string();

    let default_gems = read_default_gems();
    if !default_gems.is_empty() {
        if let Some(cb) = &on_progress {
            cb("gems", 95, &format!("Installing {} default gem(s)...", default_gems.len()));
        }
        for (gem_name, gem_version) in &default_gems {
            let _ = install_gem(version.clone(), gem_name.clone(), gem_version.clone()).await;
        }
    }

    if let Some(cb) = &on_progress {
        cb("done", 100, &format!("Installed: {ruby_version_output}"));
    }

    Ok(())
}

/// Compute the SHA256 hex digest of a byte slice.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Fetch the expected SHA256 checksum for a ruby-builder archive.
/// ruby-builder publishes a `.sha256` file alongside each archive.
pub async fn fetch_checksum(client: &reqwest::Client, archive_url: &str) -> Result<Option<String>, String> {
    let checksum_url = format!("{archive_url}.sha256");
    let response = client
        .get(&checksum_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch checksum: {e}"))?;

    if !response.status().is_success() {
        // Checksum file not available — skip verification with a warning
        return Ok(None);
    }

    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read checksum: {e}"))?;

    // Format is either just the hex hash, or "hash  filename"
    let hash = body.trim().split_whitespace().next().unwrap_or("").to_lowercase();
    if hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(Some(hash))
    } else {
        Ok(None)
    }
}

/// Read the default-gems file (~/.rubies/default-gems) and return gem names.
/// Each line is a gem name, optionally followed by a version.
/// Lines starting with # are comments.
pub fn read_default_gems() -> Vec<(String, Option<String>)> {
    let path = rubies_dir().join("default-gems");
    if !path.exists() {
        return vec![];
    }
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let mut parts = l.split_whitespace();
            let name = parts.next().unwrap_or("").to_string();
            let version = parts.next().map(|v| v.to_string());
            (name, version)
        })
        .filter(|(name, _)| !name.is_empty())
        .collect()
}

pub async fn install_ruby(version: String, on_progress: Option<ProgressCallback>) -> Result<(), String> {
    if !is_valid_version(&version) {
        return Err(format!("Invalid version format: {version}. Expected format like 4.0.2, jruby-9.4.9.0, or truffleruby-24.1.1"));
    }

    let target_dir = rubies_dir().join(&version);
    if target_dir.join("bin").join("ruby").exists() {
        return Err(format!("Ruby {version} is already installed"));
    }

    fs::create_dir_all(&rubies_dir())
        .map_err(|e| format!("Failed to create rubies directory: {e}"))?;

    // Determine download URL based on platform and engine
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let platform_tag = match (os, arch) {
        ("macos", "aarch64") => "darwin-arm64",
        ("macos", "x86_64") => "darwin-x64",
        ("linux", "x86_64") => "ubuntu-22.04-x64",
        ("linux", "aarch64") => "ubuntu-22.04-arm64",
        _ => return Err(format!("Unsupported platform: {os}-{arch}. Windows requires RubyInstaller.")),
    };

    let (_, base_version, tag_prefix) = parse_engine(&version);
    let base_url = download_base_url();
    let url = format!(
        "{base_url}/releases/download/{tag_prefix}-{base_version}/{tag_prefix}-{base_version}-{platform_tag}.tar.gz"
    );

    // Stage 1: Download
    if let Some(cb) = &on_progress {
        cb("download", 10, &format!("Downloading {version}..."));
    }

    let client = build_http_client()?;
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed: HTTP {} — prebuilt binary may not exist for this platform",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download: {e}"))?;

    if let Some(cb) = &on_progress {
        cb("download", 50, &format!("Downloaded {} MB", bytes.len() / 1_048_576));
    }

    // Verify SHA256 checksum
    if let Some(cb) = &on_progress {
        cb("verify", 55, "Verifying checksum...");
    }

    match fetch_checksum(&client, &url).await {
        Ok(Some(expected)) => {
            let actual = sha256_hex(&bytes);
            if actual != expected {
                return Err(format!(
                    "Checksum mismatch! Expected {expected}, got {actual}. The download may be corrupted."
                ));
            }
            if let Some(cb) = &on_progress {
                cb("verify", 58, "Checksum verified");
            }
        }
        Ok(None) => {
            // No checksum available — proceed with warning
            if let Some(cb) = &on_progress {
                cb("verify", 58, "No checksum available — skipping verification");
            }
        }
        Err(_) => {
            // Checksum fetch failed — proceed with warning
            if let Some(cb) = &on_progress {
                cb("verify", 58, "Could not fetch checksum — skipping verification");
            }
        }
    }

    // Stage 2: Extract
    if let Some(cb) = &on_progress {
        cb("extract", 60, "Extracting archive...");
    }

    let tmp_dir = rubies_dir().join(".tmp-install");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir)
        .map_err(|e| format!("Failed to create temp dir: {e}"))?;

    let archive_path = tmp_dir.join(format!("ruby-{version}.tar.gz"));
    fs::write(&archive_path, &bytes)
        .map_err(|e| format!("Failed to write archive: {e}"))?;

    // Extract using system tar
    let extract_output = Command::new("tar")
        .args(["xzf", &archive_path.to_string_lossy(), "-C", &tmp_dir.to_string_lossy()])
        .output()
        .map_err(|e| format!("Extraction failed: {e}"))?;

    if !extract_output.status.success() {
        let stderr = String::from_utf8_lossy(&extract_output.stderr);
        return Err(format!("Extraction failed: {stderr}"));
    }

    if let Some(cb) = &on_progress {
        cb("extract", 80, "Organizing files...");
    }

    // Find the extracted Ruby directory (may be nested, e.g. arm64/bin/ruby)
    let bin_dir = find_ruby_bin_dir(&tmp_dir)
        .ok_or("Could not find ruby binary in extracted archive")?;

    // ruby_root is the directory containing bin/ (e.g. {tmp}/arm64/)
    let ruby_root = bin_dir
        .parent()
        .ok_or("Unexpected archive structure")?;

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)
            .map_err(|e| format!("Failed to clean target: {e}"))?;
    }

    fs::rename(ruby_root, &target_dir).or_else(|_| {
        // Cross-device move fallback: copy then delete
        copy_dir_recursive(ruby_root, &target_dir)
    }).map_err(|e| format!("Failed to move Ruby to final location: {e}"))?;

    // Cleanup
    let _ = fs::remove_dir_all(&tmp_dir);

    // Create gems directory
    let gems_dir = rubies_dir().join("gems").join(&version);
    let _ = fs::create_dir_all(&gems_dir);

    // Fix hardcoded dylib paths (macOS only)
    // ruby-builder binaries reference /Users/runner/hostedtoolcache/... which doesn't exist locally
    if cfg!(target_os = "macos") {
        if let Some(cb) = &on_progress {
            cb("verify", 85, "Fixing library paths...");
        }
        fix_macos_dylib_paths(&target_dir, &version)?;
    }

    // Fix hardcoded paths on Linux (rpath / interpreter)
    if cfg!(target_os = "linux") {
        if let Some(cb) = &on_progress {
            cb("verify", 85, "Fixing library paths...");
        }
        fix_linux_rpath(&target_dir)?;
    }

    // Fix shebangs in bin/ scripts (gem, bundle, irb, etc.)
    // They reference /Users/runner/... which doesn't exist locally
    fix_shebangs(&target_dir)?;

    // Stage 3: Verify
    if let Some(cb) = &on_progress {
        cb("verify", 90, "Verifying installation...");
    }

    let ruby_bin = target_dir.join("bin").join("ruby");
    let verify = Command::new(&ruby_bin)
        .arg("--version")
        .output()
        .map_err(|e| format!("Verification failed: {e}"))?;

    if !verify.status.success() {
        let stderr = String::from_utf8_lossy(&verify.stderr);
        let _ = fs::remove_dir_all(&target_dir);
        return Err(format!("Ruby binary failed verification: {stderr}"));
    }

    let ruby_version_output = String::from_utf8_lossy(&verify.stdout).trim().to_string();

    // Install default gems if ~/.rubies/default-gems exists
    let default_gems = read_default_gems();
    if !default_gems.is_empty() {
        if let Some(cb) = &on_progress {
            cb("gems", 95, &format!("Installing {} default gem(s)...", default_gems.len()));
        }
        for (gem_name, gem_version) in &default_gems {
            let _ = install_gem(version.clone(), gem_name.clone(), gem_version.clone()).await;
        }
    }

    if let Some(cb) = &on_progress {
        cb("done", 100, &format!("Installed: {ruby_version_output}"));
    }

    Ok(())
}

/// On macOS, ruby-builder binaries have hardcoded dylib paths from the GitHub Actions runner.
/// We use install_name_tool to rewrite them to the actual install location.
pub fn fix_macos_dylib_paths(target_dir: &std::path::Path, _version: &str) -> Result<(), String> {
    let target_lib_dir = target_dir.join("lib");

    // Collect all Mach-O files that may reference the runner's libruby:
    // bin/ruby, all .bundle files (native extensions), and .dylib files
    let mut files_to_fix: Vec<PathBuf> = Vec::new();

    let ruby_bin = target_dir.join("bin").join("ruby");
    if ruby_bin.exists() {
        files_to_fix.push(ruby_bin);
    }

    // Recursively find all .bundle and .dylib files
    collect_files_recursive(target_dir, &["bundle", "dylib"], &mut files_to_fix);

    for file in &files_to_fix {
        let otool_output = match Command::new("otool")
            .args(["-L", &file.to_string_lossy()])
            .output()
        {
            Ok(o) => o,
            Err(_) => continue,
        };

        let otool_str = String::from_utf8_lossy(&otool_output.stdout);

        for line in otool_str.lines() {
            let line = line.trim();
            if line.contains("libruby") && line.contains("/runner/") {
                let old_path = match line.split_whitespace().next() {
                    Some(p) if !p.is_empty() => p,
                    _ => continue,
                };
                let lib_filename = match std::path::Path::new(old_path).file_name().and_then(|f| f.to_str()) {
                    Some(f) if !f.is_empty() => f.to_string(),
                    _ => continue,
                };

                let new_path = target_lib_dir.join(&lib_filename);
                if new_path.exists() {
                    let _ = Command::new("install_name_tool")
                        .args([
                            "-change",
                            old_path,
                            &new_path.to_string_lossy(),
                            &file.to_string_lossy(),
                        ])
                        .output();
                }
            }
        }
    }

    // Fix the dylib's own install name
    if let Ok(entries) = fs::read_dir(&target_lib_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("dylib") {
                let _ = Command::new("install_name_tool")
                    .args(["-id", &path.to_string_lossy(), &path.to_string_lossy()])
                    .output();
            }
        }
    }

    Ok(())
}

pub fn collect_files_recursive(dir: &std::path::Path, extensions: &[&str], out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files_recursive(&path, extensions, out);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext) {
                    out.push(path);
                }
            }
        }
    }
}

/// On Linux, fix the rpath in the ruby binary to point to the actual lib directory.
pub fn fix_linux_rpath(target_dir: &std::path::Path) -> Result<(), String> {
    let ruby_bin = target_dir.join("bin").join("ruby");
    if !ruby_bin.exists() {
        return Ok(());
    }

    let lib_dir = target_dir.join("lib");

    // Use patchelf if available to set the rpath
    if which::which("patchelf").is_ok() {
        let _ = Command::new("patchelf")
            .args([
                "--set-rpath",
                &lib_dir.to_string_lossy(),
                &ruby_bin.to_string_lossy(),
            ])
            .output();
    }

    Ok(())
}

/// Fix shebangs in bin/ scripts (gem, bundle, irb, rake, etc.)
/// They reference the GitHub Actions runner path which doesn't exist locally.
pub fn fix_shebangs(target_dir: &std::path::Path) -> Result<(), String> {
    let bin_dir = target_dir.join("bin");
    if !bin_dir.exists() {
        return Ok(());
    }

    let ruby_bin = bin_dir.join("ruby");
    let new_shebang = format!("#!{}", ruby_bin.to_string_lossy());

    let entries = fs::read_dir(&bin_dir)
        .map_err(|e| format!("Failed to read bin dir: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();
        // Skip the ruby binary itself and non-files
        if path.file_name().map(|n| n == "ruby") == Some(true) {
            continue;
        }
        if !path.is_file() {
            continue;
        }

        // Read the first line to check if it's a script with a bad shebang
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue, // Binary file or unreadable, skip
        };

        if let Some(first_line) = content.lines().next() {
            if first_line.starts_with("#!") && first_line.contains("/runner/") {
                let new_content = format!("{}\n{}", new_shebang, &content[first_line.len()..].trim_start_matches('\n'));
                let _ = fs::write(&path, new_content);
            }
        }
    }

    Ok(())
}

pub fn find_ruby_bin_dir(dir: &std::path::Path) -> Option<PathBuf> {
    if dir.join("bin").join("ruby").exists() {
        return Some(dir.join("bin"));
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if let Some(found) = find_ruby_bin_dir(&entry.path()) {
                    return Some(found);
                }
            }
        }
    }
    None
}

pub fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("mkdir failed: {e}"))?;
    for entry in fs::read_dir(src).map_err(|e| format!("readdir failed: {e}"))? {
        let entry = entry.map_err(|e| format!("entry failed: {e}"))?;
        let target = dst.join(entry.file_name());
        if entry.file_type().map_err(|e| format!("{e}"))?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target).map_err(|e| format!("copy failed: {e}"))?;
        }
    }
    Ok(())
}

pub fn check_shell_hook() -> Result<Vec<ShellHookStatus>, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let mut results = Vec::new();

    // Check each common shell
    let shells: Vec<(&str, Vec<PathBuf>)> = vec![
        ("zsh", vec![home.join(".zshrc")]),
        ("bash", vec![home.join(".bashrc"), home.join(".bash_profile")]),
        ("fish", vec![
            dirs::config_dir().unwrap_or_else(|| home.join(".config")).join("fish/conf.d/rubynaut.fish"),
        ]),
    ];

    for (shell_name, rc_files) in shells {
        for rc_file in rc_files {
            let installed = rc_file.exists()
                && fs::read_to_string(&rc_file)
                    .map(|c| c.contains("rubynaut_switch") || c.contains("Rubynaut shell integration"))
                    .unwrap_or(false);

            results.push(ShellHookStatus {
                shell: shell_name.to_string(),
                rc_file: rc_file.to_string_lossy().to_string(),
                installed,
            });
        }
    }

    Ok(results)
}

pub fn install_shell_hook(shell: String) -> Result<String, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let rubies_path = rubies_dir().to_string_lossy().to_string();

    let (rc_file, hook_content) = match shell.as_str() {
        "zsh" => (
            home.join(".zshrc"),
            generate_posix_hook(&rubies_path),
        ),
        "bash" => (
            home.join(".bashrc"),
            generate_posix_hook(&rubies_path),
        ),
        "fish" => {
            let fish_dir = dirs::config_dir()
                .unwrap_or_else(|| home.join(".config"))
                .join("fish/conf.d");
            let _ = fs::create_dir_all(&fish_dir);
            (
                fish_dir.join("rubynaut.fish"),
                generate_fish_hook(&rubies_path),
            )
        }
        "powershell" => {
            return Err("PowerShell hook must be added manually to $PROFILE".to_string());
        }
        _ => return Err(format!("Unsupported shell: {shell}")),
    };

    // Check if already installed
    if rc_file.exists() {
        let content = fs::read_to_string(&rc_file)
            .map_err(|e| format!("Failed to read {}: {e}", rc_file.display()))?;
        if content.contains("rubynaut_switch") || content.contains("Rubynaut shell integration") {
            return Ok(format!("Hook already installed in {}", rc_file.display()));
        }
    }

    // For fish, write the entire file (it's a dedicated config file)
    // For bash/zsh, append to existing rc file
    if shell == "fish" {
        fs::write(&rc_file, &hook_content)
            .map_err(|e| format!("Failed to write {}: {e}", rc_file.display()))?;
    } else {
        let mut content = if rc_file.exists() {
            fs::read_to_string(&rc_file)
                .map_err(|e| format!("Failed to read {}: {e}", rc_file.display()))?
        } else {
            String::new()
        };

        // Add a newline separator if file doesn't end with one
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push('\n');
        content.push_str(&hook_content);

        fs::write(&rc_file, &content)
            .map_err(|e| format!("Failed to write {}: {e}", rc_file.display()))?;
    }

    Ok(format!("Hook installed to {}", rc_file.display()))
}

pub fn get_shell_hook(shell: String) -> Result<String, String> {
    let rubies_path = rubies_dir().to_string_lossy().to_string();

    match shell.as_str() {
        "bash" | "zsh" | "/bin/bash" | "/bin/zsh" | "/usr/bin/zsh" => {
            Ok(generate_posix_hook(&rubies_path))
        }
        "fish" | "/usr/bin/fish" | "/usr/local/bin/fish" | "/opt/homebrew/bin/fish" => {
            Ok(generate_fish_hook(&rubies_path))
        }
        "powershell" | "pwsh" => {
            Ok(generate_powershell_hook(&rubies_path))
        }
        _ => Err(format!("Unsupported shell: {shell}"))
    }
}

pub fn generate_posix_hook(rubies_path: &str) -> String {
    format!(r#"# Rubynaut shell integration
# Add this to your .bashrc or .zshrc
rubynaut_switch() {{
  local target_version=""
  local dir="$PWD"

  while [ "$dir" != "/" ]; do
    if [ -f "$dir/.ruby-version" ]; then
      target_version=$(cat "$dir/.ruby-version")
      break
    fi
    dir=$(dirname "$dir")
  done

  if [ -z "$target_version" ] && [ -f "{rubies_path}/config.json" ]; then
    target_version=$(ruby -rjson -e 'puts JSON.parse(File.read("{rubies_path}/config.json"))["global_version"] rescue nil' 2>/dev/null)
  fi

  [ "$target_version" = "$RUBYNAUT_VERSION" ] && return

  if [ -n "$RUBYNAUT_VERSION" ]; then
    PATH=$(echo "$PATH" | tr ':' '\n' | grep -v "{rubies_path}" | tr '\n' ':' | sed 's/:$//')
    unset GEM_HOME GEM_PATH RUBYNAUT_VERSION
  fi

  if [ -n "$target_version" ] && [ -d "{rubies_path}/$target_version/bin" ]; then
    export RUBYNAUT_VERSION="$target_version"
    export GEM_HOME="{rubies_path}/gems/$target_version"
    export GEM_PATH="$GEM_HOME:{rubies_path}/$target_version/lib/ruby/gems/${{target_version%.*}}.0"
    export PATH="{rubies_path}/$target_version/bin:$GEM_HOME/bin:$PATH"
    # Ensure Homebrew libraries (libyaml, openssl, etc.) are findable
    if [ -d /opt/homebrew/lib ]; then
      export DYLD_FALLBACK_LIBRARY_PATH="{rubies_path}/$target_version/lib:/opt/homebrew/lib:${{DYLD_FALLBACK_LIBRARY_PATH:-/usr/local/lib:/usr/lib}}"
    fi
  elif [ -n "$target_version" ] && ! [ -d "{rubies_path}/$target_version/bin" ]; then
    # Auto-install: version required but not installed
    if command -v rubynaut >/dev/null 2>&1; then
      echo "rubynaut: Ruby $target_version is not installed."
      printf "Install it now? [y/N] "
      read -r answer
      if [ "$answer" = "y" ] || [ "$answer" = "Y" ]; then
        rubynaut install "$target_version" && rubynaut_switch
      fi
    fi
  fi
}}

if [ -n "$ZSH_VERSION" ]; then
  chpwd_functions+=(rubynaut_switch)
else
  cd() {{ builtin cd "$@" && rubynaut_switch; }}
fi
rubynaut_switch
"#)
}

pub fn generate_fish_hook(rubies_path: &str) -> String {
    format!(r#"# Rubynaut shell integration for Fish
# Add this to ~/.config/fish/conf.d/rubynaut.fish
function rubynaut_switch --on-variable PWD
  set -l target_version ""
  set -l dir $PWD

  while test "$dir" != "/"
    if test -f "$dir/.ruby-version"
      set target_version (string trim (cat "$dir/.ruby-version"))
      break
    end
    set dir (dirname "$dir")
  end

  if test -z "$target_version"; and test -f "{rubies_path}/config.json"
    set target_version (ruby -rjson -e 'puts JSON.parse(File.read("{rubies_path}/config.json"))["global_version"] rescue nil' 2>/dev/null)
  end

  test "$target_version" = "$RUBYNAUT_VERSION"; and return

  if set -q RUBYNAUT_VERSION
    set PATH (string match -v "*{rubies_path}*" $PATH)
    set -e GEM_HOME
    set -e GEM_PATH
    set -e RUBYNAUT_VERSION
  end

  if test -n "$target_version"; and test -d "{rubies_path}/$target_version/bin"
    set -gx RUBYNAUT_VERSION "$target_version"
    set -gx GEM_HOME "{rubies_path}/gems/$target_version"
    set -gx GEM_PATH "$GEM_HOME:{rubies_path}/$target_version/lib/ruby/gems/"(string replace -r '\.\d+$' '.0' $target_version)
    set -gx PATH "{rubies_path}/$target_version/bin" "$GEM_HOME/bin" $PATH
  else if test -n "$target_version"; and not test -d "{rubies_path}/$target_version/bin"
    # Auto-install: version required but not installed
    if command -v rubynaut >/dev/null 2>&1
      echo "rubynaut: Ruby $target_version is not installed."
      read -P "Install it now? [y/N] " answer
      if test "$answer" = "y" -o "$answer" = "Y"
        rubynaut install "$target_version"; and rubynaut_switch
      end
    end
  end
end

rubynaut_switch
"#)
}

pub fn generate_powershell_hook(rubies_path: &str) -> String {
    format!(r#"# Rubynaut shell integration for PowerShell
# Add this to your $PROFILE
function Invoke-RubynautSwitch {{
  $targetVersion = $null
  $dir = Get-Location

  while ($dir -ne [System.IO.Path]::GetPathRoot($dir.Path)) {{
    $rvFile = Join-Path $dir.Path ".ruby-version"
    if (Test-Path $rvFile) {{
      $targetVersion = (Get-Content $rvFile).Trim()
      break
    }}
    $dir = Split-Path $dir.Path -Parent | Get-Item
  }}

  if (-not $targetVersion -and (Test-Path "{rubies_path}/config.json")) {{
    $config = Get-Content "{rubies_path}/config.json" | ConvertFrom-Json
    $targetVersion = $config.global_version
  }}

  if ($targetVersion -eq $env:RUBYNAUT_VERSION) {{ return }}

  if ($env:RUBYNAUT_VERSION) {{
    $env:PATH = ($env:PATH -split [IO.Path]::PathSeparator | Where-Object {{ $_ -notlike "*{rubies_path}*" }}) -join [IO.Path]::PathSeparator
  }}

  $rubyBin = Join-Path "{rubies_path}" "$targetVersion/bin"
  if ($targetVersion -and (Test-Path $rubyBin)) {{
    $env:RUBYNAUT_VERSION = $targetVersion
    $env:GEM_HOME = "{rubies_path}/gems/$targetVersion"
    $env:PATH = "$rubyBin$([IO.Path]::PathSeparator)$env:GEM_HOME/bin$([IO.Path]::PathSeparator)$env:PATH"
  }}
}}

# Hook into prompt
$originalPrompt = Get-Content Function:\prompt
Set-Content Function:\prompt {{ Invoke-RubynautSwitch; & $originalPrompt }}
"#)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ==========================================
    // Version comparison
    // ==========================================

    #[test]
    fn test_version_cmp_basic() {
        assert_eq!(version_cmp("4.0.2", "3.3.6"), std::cmp::Ordering::Greater);
        assert_eq!(version_cmp("3.3.6", "4.0.2"), std::cmp::Ordering::Less);
        assert_eq!(version_cmp("3.3.6", "3.3.6"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_version_cmp_different_lengths() {
        assert_eq!(version_cmp("4.0.2", "4.0"), std::cmp::Ordering::Greater);
        assert_eq!(version_cmp("3.3", "3.3.6"), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_version_cmp_double_digits() {
        assert_eq!(version_cmp("3.3.11", "3.3.9"), std::cmp::Ordering::Greater);
        assert_eq!(version_cmp("3.2.10", "3.2.2"), std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_version_cmp_patch_suffix() {
        // "2.0.0-p648" should parse as [2, 0, 0]
        assert_eq!(version_cmp("2.0.0-p648", "1.9.3-p551"), std::cmp::Ordering::Greater);
        assert_eq!(version_cmp("2.0.0-p648", "2.0.0"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_version_cmp_major_difference() {
        assert_eq!(version_cmp("4.0.0", "3.999.999"), std::cmp::Ordering::Greater);
        assert_eq!(version_cmp("1.9.3", "2.0.0"), std::cmp::Ordering::Less);
    }

    // ==========================================
    // Config read/write
    // ==========================================

    #[test]
    fn test_config_default() {
        let config = RubynautConfig::default();
        assert!(config.global_version.is_none());
        assert!(config.projects.is_empty());
    }

    #[test]
    fn test_config_deserialize_minimal() {
        let json = r#"{"global_version": "4.0.2"}"#;
        let config: RubynautConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.global_version, Some("4.0.2".to_string()));
        assert!(config.projects.is_empty()); // serde(default) kicks in
    }

    #[test]
    fn test_config_deserialize_with_projects() {
        let json = r#"{"global_version": "4.0.2", "projects": [{"path": "/tmp/foo", "name": "foo"}]}"#;
        let config: RubynautConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.projects.len(), 1);
        assert_eq!(config.projects[0].name, "foo");
    }

    #[test]
    fn test_config_roundtrip() {
        let config = RubynautConfig {
            global_version: Some("3.3.6".to_string()),
            projects: vec![TrackedProject {
                path: "/tmp/myproject".to_string(),
                name: "myproject".to_string(),
            }],
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: RubynautConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.global_version, config.global_version);
        assert_eq!(parsed.projects.len(), 1);
    }

    // ==========================================
    // Gemfile.lock parsing
    // ==========================================

    #[test]
    fn test_parse_gemfile_lock() {
        let dir = TempDir::new().unwrap();
        let lockfile = dir.path().join("Gemfile.lock");
        fs::write(&lockfile, r#"GEM
  remote: https://rubygems.org/
  specs:
    actioncable (7.1.0)
      actionpack (= 7.1.0)
      nio4r (~> 2.0)
    nokogiri (1.16.0-arm64-darwin)
      racc (~> 1.4)
    puma (6.4.0)
    rake (13.1.0)

PLATFORMS
  arm64-darwin-23

DEPENDENCIES
  rails (~> 7.1)
"#).unwrap();

        let result = get_project_gems(dir.path().to_string_lossy().to_string());
        let gems = result.unwrap();

        assert_eq!(gems.len(), 4);
        assert_eq!(gems[0].name, "actioncable");
        assert_eq!(gems[0].version, "7.1.0");
        assert_eq!(gems[1].name, "nokogiri");
        assert_eq!(gems[1].version, "1.16.0-arm64-darwin");
        assert_eq!(gems[2].name, "puma");
        assert_eq!(gems[3].name, "rake");
    }

    #[test]
    fn test_parse_gemfile_lock_empty() {
        let dir = TempDir::new().unwrap();
        let lockfile = dir.path().join("Gemfile.lock");
        fs::write(&lockfile, "GEM\n  remote: https://rubygems.org/\n  specs:\n\nPLATFORMS\n").unwrap();

        let gems = get_project_gems(dir.path().to_string_lossy().to_string()).unwrap();
        assert!(gems.is_empty());
    }

    #[test]
    fn test_parse_gemfile_lock_no_file() {
        let dir = TempDir::new().unwrap();
        let result = get_project_gems(dir.path().to_string_lossy().to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No Gemfile.lock"));
    }

    #[test]
    fn test_parse_gemfile_lock_skips_subdependencies() {
        let dir = TempDir::new().unwrap();
        let lockfile = dir.path().join("Gemfile.lock");
        fs::write(&lockfile, r#"GEM
  specs:
    parent-gem (1.0.0)
      child-dep (~> 2.0)
      another-child (>= 1.0)
    standalone (3.0.0)
"#).unwrap();

        let gems = get_project_gems(dir.path().to_string_lossy().to_string()).unwrap();
        // Should only get parent-gem and standalone, not child deps
        assert_eq!(gems.len(), 2);
        let names: Vec<&str> = gems.iter().map(|g| g.name.as_str()).collect();
        assert!(names.contains(&"parent-gem"));
        assert!(names.contains(&"standalone"));
    }

    // ==========================================
    // Project scanning
    // ==========================================

    #[test]
    fn test_scan_project_ruby_version() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(".ruby-version"), "3.3.6\n").unwrap();

        let result = scan_project(dir.path().to_string_lossy().to_string()).unwrap();
        assert_eq!(result.detected_version, Some("3.3.6".to_string()));
        assert_eq!(result.source, Some(".ruby-version".to_string()));
        assert!(result.folder_exists);
    }

    #[test]
    fn test_scan_project_tool_versions() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(".tool-versions"), "nodejs 18.0.0\nruby 3.2.4\npython 3.11\n").unwrap();

        let result = scan_project(dir.path().to_string_lossy().to_string()).unwrap();
        assert_eq!(result.detected_version, Some("3.2.4".to_string()));
        assert_eq!(result.source, Some(".tool-versions".to_string()));
    }

    #[test]
    fn test_scan_project_gemfile_ruby() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Gemfile"), "source 'https://rubygems.org'\nruby \"3.3.6\"\ngem 'rails'\n").unwrap();

        let result = scan_project(dir.path().to_string_lossy().to_string()).unwrap();
        assert_eq!(result.detected_version, Some("3.3.6".to_string()));
        assert_eq!(result.source, Some("Gemfile".to_string()));
        assert!(result.has_gemfile);
    }

    #[test]
    fn test_scan_project_gemfile_constraint() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Gemfile"), "ruby \"~> 3.3.0\"\n").unwrap();

        let result = scan_project(dir.path().to_string_lossy().to_string()).unwrap();
        assert_eq!(result.detected_version, Some("3.3.0".to_string()));
    }

    #[test]
    fn test_scan_project_priority_order() {
        let dir = TempDir::new().unwrap();
        // .ruby-version takes priority over .tool-versions and Gemfile
        fs::write(dir.path().join(".ruby-version"), "4.0.2\n").unwrap();
        fs::write(dir.path().join(".tool-versions"), "ruby 3.3.6\n").unwrap();
        fs::write(dir.path().join("Gemfile"), "ruby \"3.2.0\"\n").unwrap();

        let result = scan_project(dir.path().to_string_lossy().to_string()).unwrap();
        assert_eq!(result.detected_version, Some("4.0.2".to_string()));
        assert_eq!(result.source, Some(".ruby-version".to_string()));
    }

    #[test]
    fn test_scan_project_no_version() {
        let dir = TempDir::new().unwrap();
        let result = scan_project(dir.path().to_string_lossy().to_string()).unwrap();
        assert!(result.detected_version.is_none());
        assert!(result.source.is_none());
    }

    #[test]
    fn test_scan_project_not_a_directory() {
        let result = scan_project("/nonexistent/path/12345".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_project_has_gemfile_lock() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Gemfile"), "gem 'rails'\n").unwrap();
        fs::write(dir.path().join("Gemfile.lock"), "GEM\n").unwrap();

        let result = scan_project(dir.path().to_string_lossy().to_string()).unwrap();
        assert!(result.has_gemfile);
        assert!(result.has_gemfile_lock);
    }

    // ==========================================
    // scan_project_safe
    // ==========================================

    #[test]
    fn test_scan_project_safe_missing_dir() {
        let result = scan_project_safe("/nonexistent/12345", "testproject");
        assert!(!result.folder_exists);
        assert_eq!(result.project_name, "testproject");
        assert!(result.detected_version.is_none());
    }

    #[test]
    fn test_scan_project_safe_existing_dir() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(".ruby-version"), "3.3.6\n").unwrap();

        let result = scan_project_safe(&dir.path().to_string_lossy(), "myproject");
        assert!(result.folder_exists);
        assert_eq!(result.detected_version, Some("3.3.6".to_string()));
    }

    // ==========================================
    // Shell hook generation
    // ==========================================

    #[test]
    fn test_shell_hook_zsh() {
        let result = get_shell_hook("zsh".to_string());
        assert!(result.is_ok());
        let hook = result.unwrap();
        assert!(hook.contains("rubynaut_switch"));
        assert!(hook.contains("chpwd_functions"));
        assert!(hook.contains(".rubies"));
    }

    #[test]
    fn test_shell_hook_bash() {
        let result = get_shell_hook("bash".to_string());
        assert!(result.is_ok());
        let hook = result.unwrap();
        assert!(hook.contains("rubynaut_switch"));
        assert!(hook.contains("builtin cd"));
    }

    #[test]
    fn test_shell_hook_fish() {
        let result = get_shell_hook("fish".to_string());
        assert!(result.is_ok());
        let hook = result.unwrap();
        assert!(hook.contains("rubynaut_switch"));
        assert!(hook.contains("--on-variable PWD"));
    }

    #[test]
    fn test_shell_hook_powershell() {
        let result = get_shell_hook("powershell".to_string());
        assert!(result.is_ok());
        let hook = result.unwrap();
        assert!(hook.contains("Invoke-RubynautSwitch"));
    }

    #[test]
    fn test_shell_hook_unsupported() {
        let result = get_shell_hook("csh".to_string());
        assert!(result.is_err());
    }

    // ==========================================
    // RUBYLIB builder
    // ==========================================

    #[test]
    fn test_build_rubylib_empty_dir() {
        let dir = TempDir::new().unwrap();
        let result = build_rubylib(dir.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_build_rubylib_with_version_dir() {
        let dir = TempDir::new().unwrap();
        let ruby_lib = dir.path().join("lib").join("ruby").join("3.3.0");
        fs::create_dir_all(&ruby_lib).unwrap();

        let result = build_rubylib(dir.path());
        assert!(result.contains("3.3.0"));
    }

    #[test]
    fn test_build_rubylib_with_arch_subdir() {
        let dir = TempDir::new().unwrap();
        let arch_dir = dir.path().join("lib").join("ruby").join("4.0.0").join("arm64-darwin23");
        fs::create_dir_all(&arch_dir).unwrap();

        let result = build_rubylib(dir.path());
        assert!(result.contains("4.0.0"));
        assert!(result.contains("arm64-darwin23"));
    }

    #[test]
    fn test_build_rubylib_includes_site_ruby() {
        let dir = TempDir::new().unwrap();
        let site = dir.path().join("lib").join("ruby").join("site_ruby");
        fs::create_dir_all(&site).unwrap();

        let result = build_rubylib(dir.path());
        assert!(result.contains("site_ruby"));
    }

    // ==========================================
    // gem_env
    // ==========================================

    #[test]
    fn test_gem_env_contains_required_vars() {
        let dir = TempDir::new().unwrap();
        let env = gem_env(dir.path(), "3.3.6");

        let keys: Vec<&str> = env.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"GEM_HOME"));
        assert!(keys.contains(&"GEM_PATH"));
        assert!(keys.contains(&"PATH"));
    }

    #[test]
    fn test_gem_env_gem_home_includes_version() {
        let dir = TempDir::new().unwrap();
        let env = gem_env(dir.path(), "4.0.2");

        let gem_home = env.iter().find(|(k, _)| k == "GEM_HOME").unwrap();
        assert!(gem_home.1.contains("4.0.2"));
    }

    // ==========================================
    // Fallback versions
    // ==========================================

    #[test]
    fn test_fallback_versions_not_empty() {
        let versions = fallback_versions();
        assert!(!versions.is_empty());
        assert!(versions.len() > 50);
    }

    #[test]
    fn test_fallback_versions_includes_major_releases() {
        let versions = fallback_versions();
        assert!(versions.contains(&"4.0.2".to_string()));
        assert!(versions.contains(&"3.3.0".to_string()));
        assert!(versions.contains(&"2.7.0".to_string()));
        assert!(versions.contains(&"1.9.3-p551".to_string()));
    }

    #[test]
    fn test_fallback_versions_sorted_descending() {
        let versions = fallback_versions();
        // First should be higher than last
        assert!(version_cmp(&versions[0], &versions[versions.len() - 1]) == std::cmp::Ordering::Greater);
    }

    // ==========================================
    // Version cache
    // ==========================================

    #[test]
    fn test_version_cache_roundtrip() {
        let cache = VersionCache {
            versions: vec!["4.0.2".to_string(), "3.3.6".to_string()],
            fetched_at: 1234567890,
        };
        let json = serde_json::to_string(&cache).unwrap();
        let parsed: VersionCache = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.versions.len(), 2);
        assert_eq!(parsed.fetched_at, 1234567890);
    }

    // ==========================================
    // RubyVersion struct
    // ==========================================

    #[test]
    fn test_ruby_version_serialize() {
        let rv = RubyVersion {
            version: "3.3.6".to_string(),
            installed: true,
            active: false,
            path: Some("/home/user/.rubies/3.3.6".to_string()),
            prebuilt_available: true,
        };
        let json = serde_json::to_string(&rv).unwrap();
        assert!(json.contains("3.3.6"));
        assert!(json.contains("true"));
    }

    // ==========================================
    // ProjectScanResult struct
    // ==========================================

    #[test]
    fn test_project_scan_result_serialize() {
        let result = ProjectScanResult {
            path: "/tmp/test".to_string(),
            project_name: "test".to_string(),
            detected_version: Some("3.3.6".to_string()),
            source: Some(".ruby-version".to_string()),
            version_installed: false,
            has_gemfile: true,
            has_gemfile_lock: false,
            folder_exists: true,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("has_gemfile_lock"));
        assert!(json.contains("folder_exists"));
    }

    // ==========================================
    // TrackedProject struct
    // ==========================================

    #[test]
    fn test_tracked_project_clone() {
        let project = TrackedProject {
            path: "/tmp/foo".to_string(),
            name: "foo".to_string(),
        };
        let cloned = project.clone();
        assert_eq!(cloned.path, project.path);
        assert_eq!(cloned.name, project.name);
    }

    // ==========================================
    // find_ruby_bin_dir
    // ==========================================

    #[test]
    fn test_find_ruby_bin_dir_direct() {
        let dir = TempDir::new().unwrap();
        let bin_dir = dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("ruby"), "fake").unwrap();

        let result = find_ruby_bin_dir(dir.path());
        assert!(result.is_some());
        assert!(result.unwrap().ends_with("bin"));
    }

    #[test]
    fn test_find_ruby_bin_dir_nested() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("arm64").join("bin");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("ruby"), "fake").unwrap();

        let result = find_ruby_bin_dir(dir.path());
        assert!(result.is_some());
        assert!(result.unwrap().to_string_lossy().contains("arm64"));
    }

    #[test]
    fn test_find_ruby_bin_dir_not_found() {
        let dir = TempDir::new().unwrap();
        let result = find_ruby_bin_dir(dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_find_ruby_bin_dir_empty() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("bin")).unwrap();
        // bin/ exists but no ruby binary inside
        let result = find_ruby_bin_dir(dir.path());
        assert!(result.is_none());
    }

    // ==========================================
    // copy_dir_recursive
    // ==========================================

    #[test]
    fn test_copy_dir_recursive_basic() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let dst_path = dst.path().join("copy");

        fs::write(src.path().join("file.txt"), "hello").unwrap();
        fs::create_dir_all(src.path().join("sub")).unwrap();
        fs::write(src.path().join("sub").join("nested.txt"), "world").unwrap();

        copy_dir_recursive(src.path(), &dst_path).unwrap();

        assert_eq!(fs::read_to_string(dst_path.join("file.txt")).unwrap(), "hello");
        assert_eq!(fs::read_to_string(dst_path.join("sub").join("nested.txt")).unwrap(), "world");
    }

    #[test]
    fn test_copy_dir_recursive_empty() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let dst_path = dst.path().join("copy");

        copy_dir_recursive(src.path(), &dst_path).unwrap();
        assert!(dst_path.exists());
    }

    #[test]
    fn test_copy_dir_recursive_preserves_content() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let dst_path = dst.path().join("copy");

        let content = "line1\nline2\nline3\n";
        fs::write(src.path().join("data.txt"), content).unwrap();

        copy_dir_recursive(src.path(), &dst_path).unwrap();
        assert_eq!(fs::read_to_string(dst_path.join("data.txt")).unwrap(), content);
    }

    // ==========================================
    // read_gemspecs
    // ==========================================

    #[test]
    fn test_read_gemspecs_basic() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("rake-13.1.0.gemspec"), "spec").unwrap();
        fs::write(dir.path().join("bundler-2.5.0.gemspec"), "spec").unwrap();

        let gems = read_gemspecs(dir.path(), true);
        assert_eq!(gems.len(), 2);

        let names: Vec<&str> = gems.iter().map(|g| g.name.as_str()).collect();
        assert!(names.contains(&"rake"));
        assert!(names.contains(&"bundler"));
        assert!(gems.iter().all(|g| g.is_default));
    }

    #[test]
    fn test_read_gemspecs_user_gems() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("rails-7.2.0.gemspec"), "spec").unwrap();

        let gems = read_gemspecs(dir.path(), false);
        assert_eq!(gems.len(), 1);
        assert_eq!(gems[0].name, "rails");
        assert_eq!(gems[0].version, "7.2.0");
        assert!(!gems[0].is_default);
    }

    #[test]
    fn test_read_gemspecs_empty_dir() {
        let dir = TempDir::new().unwrap();
        let gems = read_gemspecs(dir.path(), true);
        assert!(gems.is_empty());
    }

    #[test]
    fn test_read_gemspecs_ignores_non_gemspec() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("readme.md"), "hello").unwrap();
        fs::write(dir.path().join("rake-13.1.0.gemspec"), "spec").unwrap();

        let gems = read_gemspecs(dir.path(), true);
        assert_eq!(gems.len(), 1);
    }

    #[test]
    fn test_read_gemspecs_hyphenated_name() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("net-http-0.4.0.gemspec"), "spec").unwrap();

        let gems = read_gemspecs(dir.path(), true);
        assert_eq!(gems.len(), 1);
        assert_eq!(gems[0].name, "net-http");
        assert_eq!(gems[0].version, "0.4.0");
    }

    #[test]
    fn test_read_gemspecs_nonexistent_dir() {
        let gems = read_gemspecs(std::path::Path::new("/nonexistent/12345"), true);
        assert!(gems.is_empty());
    }

    // ==========================================
    // fix_shebangs
    // ==========================================

    #[test]
    fn test_fix_shebangs_rewrites_runner_path() {
        let dir = TempDir::new().unwrap();
        let bin_dir = dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("ruby"), "binary").unwrap();

        let old_shebang = "#!/Users/runner/hostedtoolcache/Ruby/4.0.2/x64/bin/ruby";
        fs::write(bin_dir.join("gem"), format!("{old_shebang}\nputs 'hello'\n")).unwrap();
        fs::write(bin_dir.join("bundle"), format!("{old_shebang}\nputs 'bundle'\n")).unwrap();

        fix_shebangs(dir.path()).unwrap();

        let gem_content = fs::read_to_string(bin_dir.join("gem")).unwrap();
        let expected_shebang = format!("#!{}", bin_dir.join("ruby").display());
        assert!(gem_content.starts_with(&expected_shebang));
        assert!(gem_content.contains("puts 'hello'"));

        let bundle_content = fs::read_to_string(bin_dir.join("bundle")).unwrap();
        assert!(bundle_content.starts_with(&expected_shebang));
    }

    #[test]
    fn test_fix_shebangs_skips_ruby_binary() {
        let dir = TempDir::new().unwrap();
        let bin_dir = dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("ruby"), "original binary content").unwrap();

        fix_shebangs(dir.path()).unwrap();

        assert_eq!(fs::read_to_string(bin_dir.join("ruby")).unwrap(), "original binary content");
    }

    #[test]
    fn test_fix_shebangs_leaves_good_shebangs_alone() {
        let dir = TempDir::new().unwrap();
        let bin_dir = dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("ruby"), "binary").unwrap();

        let good_script = "#!/usr/bin/env ruby\nputs 'hello'\n";
        fs::write(bin_dir.join("myscript"), good_script).unwrap();

        fix_shebangs(dir.path()).unwrap();

        assert_eq!(fs::read_to_string(bin_dir.join("myscript")).unwrap(), good_script);
    }

    #[test]
    fn test_fix_shebangs_no_bin_dir() {
        let dir = TempDir::new().unwrap();
        // No bin/ directory — should not error
        let result = fix_shebangs(dir.path());
        assert!(result.is_ok());
    }

    // ==========================================
    // collect_files_recursive
    // ==========================================

    #[test]
    fn test_collect_files_recursive_finds_extensions() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("lib.dylib"), "").unwrap();
        fs::create_dir_all(dir.path().join("ext")).unwrap();
        fs::write(dir.path().join("ext").join("native.bundle"), "").unwrap();
        fs::write(dir.path().join("ext").join("readme.txt"), "").unwrap();

        let mut files = Vec::new();
        collect_files_recursive(dir.path(), &["dylib", "bundle"], &mut files);

        assert_eq!(files.len(), 2);
        let names: Vec<String> = files.iter().map(|f| f.file_name().unwrap().to_string_lossy().to_string()).collect();
        assert!(names.contains(&"lib.dylib".to_string()));
        assert!(names.contains(&"native.bundle".to_string()));
    }

    #[test]
    fn test_collect_files_recursive_empty() {
        let dir = TempDir::new().unwrap();
        let mut files = Vec::new();
        collect_files_recursive(dir.path(), &["dylib"], &mut files);
        assert!(files.is_empty());
    }

    // ==========================================
    // scan_project edge cases
    // ==========================================

    #[test]
    fn test_scan_project_gemfile_single_quotes() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Gemfile"), "ruby '3.2.0'\n").unwrap();

        let result = scan_project(dir.path().to_string_lossy().to_string()).unwrap();
        assert_eq!(result.detected_version, Some("3.2.0".to_string()));
    }

    #[test]
    fn test_scan_project_gemfile_gte_constraint() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Gemfile"), "ruby \">= 3.1.0\"\n").unwrap();

        let result = scan_project(dir.path().to_string_lossy().to_string()).unwrap();
        assert_eq!(result.detected_version, Some("3.1.0".to_string()));
    }

    #[test]
    fn test_scan_project_tool_versions_ignores_other_tools() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(".tool-versions"), "nodejs 20.0.0\npython 3.12.0\n").unwrap();

        let result = scan_project(dir.path().to_string_lossy().to_string()).unwrap();
        assert!(result.detected_version.is_none());
    }

    #[test]
    fn test_scan_project_empty_ruby_version_file() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(".ruby-version"), "  \n").unwrap();

        let result = scan_project(dir.path().to_string_lossy().to_string()).unwrap();
        assert!(result.detected_version.is_none());
    }

    #[test]
    fn test_scan_project_name_from_directory() {
        let dir = TempDir::new().unwrap();
        let result = scan_project(dir.path().to_string_lossy().to_string()).unwrap();
        // project_name should be the last dir component
        assert!(!result.project_name.is_empty());
    }

    // ==========================================
    // version_cmp edge cases
    // ==========================================

    #[test]
    fn test_version_cmp_identical() {
        assert_eq!(version_cmp("1.0.0", "1.0.0"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_version_cmp_single_component() {
        assert_eq!(version_cmp("4", "3"), std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_version_cmp_empty_string() {
        assert_eq!(version_cmp("", ""), std::cmp::Ordering::Equal);
    }

    // ==========================================
    // config write/read roundtrip via temp dir
    // ==========================================

    #[test]
    fn test_config_serialization_roundtrip_with_empty_projects() {
        let config = RubynautConfig {
            global_version: Some("4.0.2".to_string()),
            projects: vec![],
            ..Default::default()
        };
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: RubynautConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.global_version, Some("4.0.2".to_string()));
        assert!(parsed.projects.is_empty());
    }

    #[test]
    fn test_config_serialization_with_null_global() {
        let config = RubynautConfig {
            global_version: None,
            projects: vec![TrackedProject {
                path: "/home/user/app".to_string(),
                name: "app".to_string(),
            }],
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: RubynautConfig = serde_json::from_str(&json).unwrap();
        assert!(parsed.global_version.is_none());
        assert_eq!(parsed.projects.len(), 1);
    }

    // ==========================================
    // Gemfile.lock parsing edge cases
    // ==========================================

    #[test]
    fn test_parse_gemfile_lock_multiple_specs_sections() {
        let dir = TempDir::new().unwrap();
        let lockfile = dir.path().join("Gemfile.lock");
        fs::write(&lockfile, r#"GEM
  remote: https://rubygems.org/
  specs:
    rails (7.1.0)

PATH
  remote: .
  specs:
    mygem (0.1.0)

PLATFORMS
  ruby
"#).unwrap();

        let gems = get_project_gems(dir.path().to_string_lossy().to_string()).unwrap();
        // Should capture gems from both specs sections
        assert_eq!(gems.len(), 2);
    }

    #[test]
    fn test_parse_gemfile_lock_gem_with_platform_suffix() {
        let dir = TempDir::new().unwrap();
        let lockfile = dir.path().join("Gemfile.lock");
        fs::write(&lockfile, r#"GEM
  specs:
    google-protobuf (4.26.0-arm64-darwin)
    grpc (1.62.0-arm64-darwin)
"#).unwrap();

        let gems = get_project_gems(dir.path().to_string_lossy().to_string()).unwrap();
        assert_eq!(gems.len(), 2);
        assert_eq!(gems[0].name, "google-protobuf");
        assert_eq!(gems[0].version, "4.26.0-arm64-darwin");
    }

    // ==========================================
    // Shell hook generation edge cases
    // ==========================================

    #[test]
    fn test_shell_hook_with_full_path() {
        let result = get_shell_hook("/bin/bash".to_string());
        assert!(result.is_ok());
        assert!(result.unwrap().contains("rubynaut_switch"));
    }

    #[test]
    fn test_shell_hook_fish_full_path() {
        let result = get_shell_hook("/usr/bin/fish".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_shell_hook_pwsh() {
        let result = get_shell_hook("pwsh".to_string());
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Invoke-RubynautSwitch"));
    }

    #[test]
    fn test_generate_posix_hook_contains_required_parts() {
        let hook = generate_posix_hook("/home/user/.rubies");
        assert!(hook.contains("rubynaut_switch"));
        assert!(hook.contains(".ruby-version"));
        assert!(hook.contains("GEM_HOME"));
        assert!(hook.contains("GEM_PATH"));
        assert!(hook.contains("/home/user/.rubies"));
    }

    #[test]
    fn test_generate_fish_hook_contains_required_parts() {
        let hook = generate_fish_hook("/home/user/.rubies");
        assert!(hook.contains("rubynaut_switch"));
        assert!(hook.contains("--on-variable PWD"));
        assert!(hook.contains("GEM_HOME"));
        assert!(hook.contains("/home/user/.rubies"));
    }

    #[test]
    fn test_generate_powershell_hook_contains_required_parts() {
        let hook = generate_powershell_hook("/home/user/.rubies");
        assert!(hook.contains("Invoke-RubynautSwitch"));
        assert!(hook.contains(".ruby-version"));
        assert!(hook.contains("/home/user/.rubies"));
    }

    // ==========================================
    // find_gem_spec_dir
    // ==========================================

    #[test]
    fn test_find_gem_spec_dir_default() {
        let dir = TempDir::new().unwrap();
        let spec_dir = dir.path().join("lib/ruby/gems/3.3.0/specifications/default");
        fs::create_dir_all(&spec_dir).unwrap();

        let result = find_gem_spec_dir(dir.path(), true);
        assert!(result.is_some());
        assert!(result.unwrap().to_string_lossy().contains("default"));
    }

    #[test]
    fn test_find_gem_spec_dir_user() {
        let dir = TempDir::new().unwrap();
        let spec_dir = dir.path().join("lib/ruby/gems/3.3.0/specifications");
        fs::create_dir_all(&spec_dir).unwrap();

        let result = find_gem_spec_dir(dir.path(), false);
        assert!(result.is_some());
    }

    #[test]
    fn test_find_gem_spec_dir_no_gems() {
        let dir = TempDir::new().unwrap();
        let result = find_gem_spec_dir(dir.path(), true);
        assert!(result.is_none());
    }

    // ==========================================
    // gem_env edge cases
    // ==========================================

    #[test]
    fn test_gem_env_path_includes_bin_dir() {
        let dir = TempDir::new().unwrap();
        let env = gem_env(dir.path(), "4.0.2");

        let path_val = env.iter().find(|(k, _)| k == "PATH").unwrap();
        assert!(path_val.1.contains("bin"));
    }

    #[test]
    fn test_gem_env_gem_path_includes_gems_dir() {
        let dir = TempDir::new().unwrap();
        let env = gem_env(dir.path(), "3.3.6");

        let gem_path = env.iter().find(|(k, _)| k == "GEM_PATH").unwrap();
        assert!(gem_path.1.contains("3.3.6"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_gem_env_macos_dyld_path() {
        let dir = TempDir::new().unwrap();
        let env = gem_env(dir.path(), "4.0.2");

        let dyld = env.iter().find(|(k, _)| k == "DYLD_FALLBACK_LIBRARY_PATH");
        assert!(dyld.is_some());
    }

    // ==========================================
    // build_rubylib with multiple dirs
    // ==========================================

    #[test]
    fn test_build_rubylib_with_vendor_ruby() {
        let dir = TempDir::new().unwrap();
        let vendor = dir.path().join("lib/ruby/vendor_ruby");
        fs::create_dir_all(&vendor).unwrap();

        let result = build_rubylib(dir.path());
        assert!(result.contains("vendor_ruby"));
    }

    #[test]
    fn test_build_rubylib_with_multiple_version_dirs() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("lib/ruby/4.0.0")).unwrap();
        fs::create_dir_all(dir.path().join("lib/ruby/site_ruby")).unwrap();

        let result = build_rubylib(dir.path());
        assert!(result.contains("4.0.0"));
        assert!(result.contains("site_ruby"));
    }

    // ==========================================
    // version_cache serialization
    // ==========================================

    #[test]
    fn test_version_cache_empty_versions() {
        let cache = VersionCache {
            versions: vec![],
            fetched_at: 0,
        };
        let json = serde_json::to_string(&cache).unwrap();
        let parsed: VersionCache = serde_json::from_str(&json).unwrap();
        assert!(parsed.versions.is_empty());
    }

    // ==========================================
    // Version string validation
    // ==========================================

    #[test]
    fn test_is_valid_version_standard() {
        assert!(is_valid_version("4.0.2"));
        assert!(is_valid_version("3.3.6"));
        assert!(is_valid_version("3.3.11"));
        assert!(is_valid_version("2.7.0"));
        assert!(is_valid_version("1.9.3-p551"));
        assert!(is_valid_version("2.0.0-p648"));
    }

    #[test]
    fn test_is_valid_version_rejects_empty() {
        assert!(!is_valid_version(""));
    }

    #[test]
    fn test_is_valid_version_rejects_path_traversal() {
        assert!(!is_valid_version("../../../etc/passwd"));
        assert!(!is_valid_version("4.0.2/../../etc"));
    }

    #[test]
    fn test_is_valid_version_rejects_special_chars() {
        assert!(!is_valid_version("4.0.2; rm -rf /"));
        assert!(!is_valid_version("4.0.2 && echo pwned"));
        assert!(!is_valid_version("$(whoami)"));
        assert!(!is_valid_version("4.0.2`id`"));
        assert!(!is_valid_version("<script>"));
    }

    #[test]
    fn test_is_valid_version_rejects_too_long() {
        // Over 40 characters should be rejected
        assert!(!is_valid_version("1.2.3.4.5.6.7.8.9.10.11.12.13.14.15.16.17.18"));
    }

    #[test]
    fn test_is_valid_version_rejects_non_digit_start() {
        assert!(!is_valid_version("ruby-4.0.2"));
        assert!(!is_valid_version("v4.0.2"));
    }

    #[test]
    fn test_is_valid_version_rejects_double_dots() {
        assert!(!is_valid_version("4..0.2"));
    }

    // ==========================================
    // SHA256 checksum
    // ==========================================

    #[test]
    fn test_sha256_hex_known_value() {
        // SHA256 of empty string is well-known
        let hash = sha256_hex(b"");
        assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn test_sha256_hex_hello_world() {
        let hash = sha256_hex(b"hello world");
        assert_eq!(hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

    #[test]
    fn test_sha256_hex_deterministic() {
        let data = b"rubynaut test data 12345";
        let hash1 = sha256_hex(data);
        let hash2 = sha256_hex(data);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
        assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sha256_hex_different_inputs_differ() {
        assert_ne!(sha256_hex(b"hello"), sha256_hex(b"world"));
    }

    // ==========================================
    // Default gems file parsing
    // ==========================================

    #[test]
    fn test_read_default_gems_no_file() {
        // When no default-gems file exists, should return empty
        let gems = read_default_gems();
        // This test depends on whether the file exists on the test machine
        // Just verify it returns a Vec without panicking
        let _ = gems;
    }

    #[test]
    fn test_parse_default_gems_format() {
        // Test the parsing logic directly by creating a temp file
        let dir = TempDir::new().unwrap();
        let gems_file = dir.path().join("default-gems");
        fs::write(&gems_file, "# My default gems\nbundler\nrails 7.2.0\npuma\n\n# another comment\nnokogiri 1.16.0\n").unwrap();

        // Parse it manually since read_default_gems reads from a fixed path
        let content = fs::read_to_string(&gems_file).unwrap();
        let parsed: Vec<(String, Option<String>)> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| {
                let mut parts = l.split_whitespace();
                let name = parts.next().unwrap_or("").to_string();
                let version = parts.next().map(|v| v.to_string());
                (name, version)
            })
            .filter(|(name, _)| !name.is_empty())
            .collect();

        assert_eq!(parsed.len(), 4);
        assert_eq!(parsed[0], ("bundler".to_string(), None));
        assert_eq!(parsed[1], ("rails".to_string(), Some("7.2.0".to_string())));
        assert_eq!(parsed[2], ("puma".to_string(), None));
        assert_eq!(parsed[3], ("nokogiri".to_string(), Some("1.16.0".to_string())));
    }

    #[test]
    fn test_parse_default_gems_empty_file() {
        let dir = TempDir::new().unwrap();
        let gems_file = dir.path().join("default-gems");
        fs::write(&gems_file, "").unwrap();

        let content = fs::read_to_string(&gems_file).unwrap();
        let parsed: Vec<(String, Option<String>)> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| {
                let mut parts = l.split_whitespace();
                let name = parts.next().unwrap_or("").to_string();
                let version = parts.next().map(|v| v.to_string());
                (name, version)
            })
            .filter(|(name, _)| !name.is_empty())
            .collect();

        assert!(parsed.is_empty());
    }

    #[test]
    fn test_parse_default_gems_comments_only() {
        let dir = TempDir::new().unwrap();
        let gems_file = dir.path().join("default-gems");
        fs::write(&gems_file, "# comment 1\n# comment 2\n").unwrap();

        let content = fs::read_to_string(&gems_file).unwrap();
        let parsed: Vec<(String, Option<String>)> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| {
                let mut parts = l.split_whitespace();
                let name = parts.next().unwrap_or("").to_string();
                let version = parts.next().map(|v| v.to_string());
                (name, version)
            })
            .filter(|(name, _)| !name.is_empty())
            .collect();

        assert!(parsed.is_empty());
    }

    // ==========================================
    // parse_engine
    // ==========================================

    #[test]
    fn test_parse_engine_cruby() {
        let (engine, version, tag) = parse_engine("4.0.2");
        assert_eq!(engine, "ruby");
        assert_eq!(version, "4.0.2");
        assert_eq!(tag, "ruby");
    }

    #[test]
    fn test_parse_engine_jruby() {
        let (engine, version, tag) = parse_engine("jruby-9.4.9.0");
        assert_eq!(engine, "jruby");
        assert_eq!(version, "9.4.9.0");
        assert_eq!(tag, "jruby");
    }

    #[test]
    fn test_parse_engine_truffleruby() {
        let (engine, version, tag) = parse_engine("truffleruby-24.1.1");
        assert_eq!(engine, "truffleruby");
        assert_eq!(version, "24.1.1");
        assert_eq!(tag, "truffleruby");
    }

    #[test]
    fn test_parse_engine_truffleruby_graalvm() {
        let (engine, version, tag) = parse_engine("truffleruby+graalvm-24.1.1");
        assert_eq!(engine, "truffleruby+graalvm");
        assert_eq!(version, "24.1.1");
        assert_eq!(tag, "truffleruby+graalvm");
    }

    #[test]
    fn test_parse_engine_cruby_with_patch() {
        let (engine, version, tag) = parse_engine("2.0.0-p648");
        assert_eq!(engine, "ruby");
        assert_eq!(version, "2.0.0-p648");
        assert_eq!(tag, "ruby");
    }

    // ==========================================
    // is_valid_version with JRuby/TruffleRuby
    // ==========================================

    #[test]
    fn test_is_valid_version_jruby() {
        assert!(is_valid_version("jruby-9.4.9.0"));
        assert!(is_valid_version("jruby-9.3.0.0"));
    }

    #[test]
    fn test_is_valid_version_truffleruby() {
        assert!(is_valid_version("truffleruby-24.1.1"));
        assert!(is_valid_version("truffleruby+graalvm-24.1.1"));
    }

    #[test]
    fn test_is_valid_version_rejects_bad_engine_prefix() {
        assert!(!is_valid_version("mruby-3.0.0")); // not a supported prefix
        assert!(!is_valid_version("jruby-")); // empty version
        assert!(!is_valid_version("truffleruby-")); // empty version
    }

    // ==========================================
    // download_base_url
    // ==========================================

    #[test]
    fn test_download_base_url_default() {
        let url = download_base_url();
        assert!(url.contains("github.com/ruby/ruby-builder"));
    }

    // ==========================================
    // build_http_client
    // ==========================================

    #[test]
    fn test_build_http_client_succeeds() {
        let client = build_http_client();
        assert!(client.is_ok());
    }

    // ==========================================
    // install_ruby_from_archive validation
    // ==========================================

    #[tokio::test]
    async fn test_install_from_archive_rejects_missing_file() {
        let result = install_ruby_from_archive(
            "4.0.2".to_string(),
            "/nonexistent/ruby.tar.gz".to_string(),
            None,
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn test_install_from_archive_rejects_wrong_extension() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("ruby.zip");
        fs::write(&file, "fake").unwrap();

        let result = install_ruby_from_archive(
            "4.0.2".to_string(),
            file.to_string_lossy().to_string(),
            None,
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains(".tar.gz"));
    }

    #[tokio::test]
    async fn test_install_from_archive_rejects_invalid_version() {
        let result = install_ruby_from_archive(
            "../evil".to_string(),
            "/some/file.tar.gz".to_string(),
            None,
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid version"));
    }

    // ==========================================
    // Config with mirror/proxy fields
    // ==========================================

    #[test]
    fn test_config_with_mirror_and_proxy() {
        let json = r#"{"global_version": "4.0.2", "mirror_url": "https://mirror.example.com/ruby-builder", "http_proxy": "http://proxy:8080"}"#;
        let config: RubynautConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.mirror_url, Some("https://mirror.example.com/ruby-builder".to_string()));
        assert_eq!(config.http_proxy, Some("http://proxy:8080".to_string()));
    }

    #[test]
    fn test_config_without_mirror_proxy_defaults_to_none() {
        let json = r#"{"global_version": "4.0.2"}"#;
        let config: RubynautConfig = serde_json::from_str(json).unwrap();
        assert!(config.mirror_url.is_none());
        assert!(config.http_proxy.is_none());
    }

    #[test]
    fn test_config_roundtrip_with_mirror() {
        let config = RubynautConfig {
            global_version: Some("4.0.2".to_string()),
            projects: vec![],
            mirror_url: Some("https://mirror.example.com".to_string()),
            http_proxy: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: RubynautConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.mirror_url, config.mirror_url);
        assert!(parsed.http_proxy.is_none());
        // http_proxy should not appear in JSON when None (skip_serializing_if)
        assert!(!json.contains("http_proxy"));
    }

    // ==========================================
    // Shell hooks contain auto-install
    // ==========================================

    #[test]
    fn test_posix_hook_contains_auto_install() {
        let hook = generate_posix_hook("/home/user/.rubies");
        assert!(hook.contains("Install it now?"));
        assert!(hook.contains("rubynaut install"));
    }

    #[test]
    fn test_fish_hook_contains_auto_install() {
        let hook = generate_fish_hook("/home/user/.rubies");
        assert!(hook.contains("Install it now?"));
        assert!(hook.contains("rubynaut install"));
    }
}
