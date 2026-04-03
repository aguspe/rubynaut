use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RubyVersion {
    pub version: String,
    pub installed: bool,
    pub active: bool,
    pub path: Option<String>,
    pub prebuilt_available: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackedProject {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RubynautConfig {
    #[serde(default)]
    pub global_version: Option<String>,
    #[serde(default)]
    pub projects: Vec<TrackedProject>,
    /// Custom mirror URL for ruby-builder downloads.
    /// Replaces "https://github.com/ruby/ruby-builder" in download URLs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirror_url: Option<String>,
    /// HTTP proxy URL (e.g. "http://proxy.corp:8080").
    /// Used for all network requests (downloads and API calls).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_proxy: Option<String>,
    /// Version aliases (e.g. "4.0" → "4.0.2", "stable" → "4.0.2").
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub aliases: HashMap<String, String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProjectScanResult {
    pub path: String,
    pub project_name: String,
    pub detected_version: Option<String>,
    pub source: Option<String>,
    pub version_installed: bool,
    pub has_gemfile: bool,
    pub has_gemfile_lock: bool,
    pub folder_exists: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProjectGem {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct GemInfo {
    pub name: String,
    pub version: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct DiagnosticResult {
    pub name: String,
    pub status: DiagnosticStatus,
    pub message: String,
    pub fix_hint: Option<String>,
    pub fix_command: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticStatus {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Serialize, Clone)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub package_manager: Option<String>,
    pub shell: String,
    pub is_wsl: bool,
    pub has_c_compiler: bool,
    pub existing_ruby_managers: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ShellHookStatus {
    pub shell: String,
    pub rc_file: String,
    pub installed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionCache {
    pub versions: Vec<String>,
    pub fetched_at: u64,
}

/// Progress callback for long-running operations (install_ruby, bundle_install).
/// Arguments: (stage, percent, message)
pub type ProgressCallback = Box<dyn Fn(&str, u8, &str) + Send + Sync>;
