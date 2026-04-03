use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn rubynaut() -> Command {
    Command::cargo_bin("rubynaut").unwrap()
}

// ==========================================
// Help and version
// ==========================================

#[test]
fn test_help_shows_usage() {
    rubynaut()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Ruby version manager"));
}

#[test]
fn test_version_shows_version() {
    rubynaut()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("rubynaut"));
}

// ==========================================
// Platform command
// ==========================================

#[test]
fn test_platform_shows_info() {
    rubynaut()
        .arg("platform")
        .assert()
        .success()
        .stdout(predicate::str::contains("Platform Info"))
        .stdout(predicate::str::contains("OS:"))
        .stdout(predicate::str::contains("Architecture:"));
}

// ==========================================
// Current command (no Ruby installed)
// ==========================================

#[test]
fn test_current_without_ruby() {
    // With a clean HOME, should show no active version or handle gracefully
    let tmp = TempDir::new().unwrap();
    rubynaut()
        .arg("current")
        .env("HOME", tmp.path())
        .assert()
        .success();
}

// ==========================================
// List command
// ==========================================

#[test]
fn test_list_installed_empty() {
    let tmp = TempDir::new().unwrap();
    rubynaut()
        .arg("list")
        .env("HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No Ruby versions installed"));
}

// ==========================================
// Doctor command
// ==========================================

#[test]
fn test_doctor_runs() {
    rubynaut()
        .args(["doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rubies directory"));
}

// ==========================================
// Shell hook command
// ==========================================

#[test]
fn test_shell_hook_zsh() {
    rubynaut()
        .args(["shell", "hook", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rubynaut_switch"));
}

#[test]
fn test_shell_hook_bash() {
    rubynaut()
        .args(["shell", "hook", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rubynaut_switch"));
}

#[test]
fn test_shell_hook_fish() {
    rubynaut()
        .args(["shell", "hook", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rubynaut_switch"));
}

#[test]
fn test_shell_hook_powershell() {
    rubynaut()
        .args(["shell", "hook", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Invoke-RubynautSwitch"));
}

#[test]
fn test_shell_hook_unsupported() {
    rubynaut()
        .args(["shell", "hook", "csh"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported shell"));
}

// ==========================================
// Shell status command
// ==========================================

#[test]
fn test_shell_status_runs() {
    rubynaut()
        .args(["shell", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Shell Hook Status"));
}

// ==========================================
// Project scan command
// ==========================================

#[test]
fn test_project_scan_empty_dir() {
    let tmp = TempDir::new().unwrap();
    rubynaut()
        .args(["project", "scan", &tmp.path().to_string_lossy()])
        .assert()
        .success();
}

#[test]
fn test_project_scan_with_ruby_version() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join(".ruby-version"), "4.0.2\n").unwrap();

    rubynaut()
        .args(["project", "scan", &tmp.path().to_string_lossy()])
        .assert()
        .success()
        .stdout(predicate::str::contains("4.0.2"))
        .stdout(predicate::str::contains(".ruby-version"));
}

#[test]
fn test_project_scan_nonexistent_dir() {
    rubynaut()
        .args(["project", "scan", "/nonexistent/path/12345"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a directory"));
}

// ==========================================
// Config commands
// ==========================================

#[test]
fn test_config_show() {
    rubynaut()
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rubynaut Configuration"))
        .stdout(predicate::str::contains("Config file:"))
        .stdout(predicate::str::contains("Global Ruby:"));
}

// ==========================================
// Alias commands
// ==========================================

#[test]
fn test_alias_list_empty() {
    let tmp = TempDir::new().unwrap();
    rubynaut()
        .args(["alias", "list"])
        .env("HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No aliases"));
}

#[test]
fn test_alias_remove_nonexistent() {
    let tmp = TempDir::new().unwrap();
    rubynaut()
        .args(["alias", "remove", "nonexistent"])
        .env("HOME", tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

// ==========================================
// Install validation
// ==========================================

#[test]
fn test_install_invalid_version() {
    rubynaut()
        .args(["install", "../evil"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid version"));
}

#[test]
fn test_install_from_file_missing() {
    rubynaut()
        .args(["install", "4.0.2", "--from-file", "/nonexistent/file.tar.gz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

// ==========================================
// Which command
// ==========================================

#[test]
fn test_which_without_active_ruby() {
    let tmp = TempDir::new().unwrap();
    rubynaut()
        .args(["which", "ruby"])
        .env("HOME", tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No active Ruby"));
}

// ==========================================
// Uninstall validation
// ==========================================

#[test]
fn test_uninstall_nonexistent() {
    let tmp = TempDir::new().unwrap();
    rubynaut()
        .args(["uninstall", "99.99.99", "--yes"])
        .env("HOME", tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed"));
}

// ==========================================
// Gems commands without active Ruby
// ==========================================

#[test]
fn test_gems_list_no_active() {
    let tmp = TempDir::new().unwrap();
    rubynaut()
        .args(["gems", "list"])
        .env("HOME", tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No active Ruby"));
}
