//! Integration tests for set_global_version, set_local_version, and config operations.
//! Runs in a separate process so setting HOME is safe.
//! NOTE: Unix-only (uses chmod for fake Ruby binaries).

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

/// Create a fake Ruby binary so version checks pass.
fn create_fake_ruby(home: &std::path::Path, version: &str) {
    let bin_dir = home.join(".rubies").join(version).join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let ruby_path = bin_dir.join("ruby");
    fs::write(&ruby_path, format!("#!/bin/sh\necho 'ruby {version}'\n")).unwrap();
    fs::set_permissions(&ruby_path, fs::Permissions::from_mode(0o755)).unwrap();
}

// ==========================================
// set_global_version
// ==========================================

#[test]
fn test_set_global_version_writes_config() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());
    create_fake_ruby(home.path(), "4.0.2");

    let result = rubynaut_core::set_global_version("4.0.2".to_string());
    assert!(result.is_ok(), "set_global_version failed: {:?}", result.err());

    let config_path = home.path().join(".rubies/config.json");
    assert!(config_path.exists(), "config.json should exist");

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("4.0.2"), "config should contain version 4.0.2");
}

#[test]
fn test_set_global_version_updates_existing_config() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());
    create_fake_ruby(home.path(), "4.0.2");
    create_fake_ruby(home.path(), "3.3.6");

    rubynaut_core::set_global_version("4.0.2".to_string()).unwrap();
    rubynaut_core::set_global_version("3.3.6".to_string()).unwrap();

    let config = rubynaut_core::read_config();
    assert_eq!(config.global_version, Some("3.3.6".to_string()));
}

#[test]
fn test_set_global_version_not_installed() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());

    let result = rubynaut_core::set_global_version("99.99.99".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not installed"));
}

#[test]
fn test_set_global_version_invalid_format() {
    let result = rubynaut_core::set_global_version("../evil".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid version"));
}

// ==========================================
// set_local_version
// ==========================================

#[test]
fn test_set_local_version_writes_ruby_version_file() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());
    create_fake_ruby(home.path(), "4.0.2");

    let project_dir = TempDir::new().unwrap();

    let result = rubynaut_core::set_local_version(
        project_dir.path().to_string_lossy().to_string(),
        "4.0.2".to_string(),
    );
    assert!(result.is_ok(), "set_local_version failed: {:?}", result.err());

    let rv_file = project_dir.path().join(".ruby-version");
    assert!(rv_file.exists(), ".ruby-version should exist");
    let content = fs::read_to_string(&rv_file).unwrap();
    assert_eq!(content.trim(), "4.0.2");
}

#[test]
fn test_set_local_version_not_installed() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());

    let project_dir = TempDir::new().unwrap();
    let result = rubynaut_core::set_local_version(
        project_dir.path().to_string_lossy().to_string(),
        "99.99.99".to_string(),
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not installed"));
}

// ==========================================
// uninstall_ruby
// ==========================================

#[test]
fn test_uninstall_ruby_removes_directory() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());
    create_fake_ruby(home.path(), "4.0.2");

    // Set it as global first
    rubynaut_core::set_global_version("4.0.2".to_string()).unwrap();

    let result = rubynaut_core::uninstall_ruby("4.0.2".to_string());
    assert!(result.is_ok());

    let version_dir = home.path().join(".rubies/4.0.2");
    assert!(!version_dir.exists(), "Version directory should be removed");

    // Global version should be unset
    let config = rubynaut_core::read_config();
    assert!(config.global_version.is_none(), "Global version should be cleared after uninstall");
}

#[test]
fn test_uninstall_ruby_not_installed() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());

    let result = rubynaut_core::uninstall_ruby("99.99.99".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not installed"));
}

// ==========================================
// get_installed_rubies
// ==========================================

#[test]
fn test_get_installed_rubies_empty() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());

    let result = rubynaut_core::get_installed_rubies();
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_get_installed_rubies_finds_versions() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());
    create_fake_ruby(home.path(), "4.0.2");
    create_fake_ruby(home.path(), "3.3.6");

    let result = rubynaut_core::get_installed_rubies().unwrap();
    assert_eq!(result.len(), 2);

    let versions: Vec<&str> = result.iter().map(|r| r.version.as_str()).collect();
    assert!(versions.contains(&"4.0.2"));
    assert!(versions.contains(&"3.3.6"));
}

// ==========================================
// Alias integration
// ==========================================

#[test]
fn test_set_and_list_alias() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());

    rubynaut_core::set_alias("stable".to_string(), "4.0.2".to_string()).unwrap();
    rubynaut_core::set_alias("4.0".to_string(), "4.0.2".to_string()).unwrap();

    let aliases = rubynaut_core::list_aliases();
    assert_eq!(aliases.len(), 2);
    assert_eq!(aliases.get("stable"), Some(&"4.0.2".to_string()));
    assert_eq!(aliases.get("4.0"), Some(&"4.0.2".to_string()));
}

#[test]
fn test_remove_alias() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());

    rubynaut_core::set_alias("old".to_string(), "3.3.6".to_string()).unwrap();
    rubynaut_core::remove_alias("old".to_string()).unwrap();

    let aliases = rubynaut_core::list_aliases();
    assert!(!aliases.contains_key("old"));
}

#[test]
fn test_resolve_alias_integration() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());

    rubynaut_core::set_alias("stable".to_string(), "4.0.2".to_string()).unwrap();

    let resolved = rubynaut_core::resolve_alias("stable");
    assert_eq!(resolved, "4.0.2");

    // Non-aliased version resolves to itself
    let resolved2 = rubynaut_core::resolve_alias("3.3.6");
    assert_eq!(resolved2, "3.3.6");
}
