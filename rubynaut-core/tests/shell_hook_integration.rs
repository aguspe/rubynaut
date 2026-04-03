//! Integration tests for install_shell_hook().
//! Runs in a separate process so setting HOME is safe.

use std::fs;
use tempfile::TempDir;

#[test]
fn test_install_shell_hook_zsh_creates_hook() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());

    let result = rubynaut_core::install_shell_hook("zsh".to_string());
    assert!(result.is_ok(), "install_shell_hook(zsh) failed: {:?}", result.err());

    let zshrc = home.path().join(".zshrc");
    assert!(zshrc.exists(), ".zshrc should exist");
    let content = fs::read_to_string(&zshrc).unwrap();
    assert!(content.contains("rubynaut_switch"), ".zshrc should contain rubynaut_switch");
    assert!(content.contains("chpwd_functions"), ".zshrc should contain chpwd_functions (zsh-specific)");
}

#[test]
fn test_install_shell_hook_bash_creates_hook() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());

    let result = rubynaut_core::install_shell_hook("bash".to_string());
    assert!(result.is_ok());

    let bashrc = home.path().join(".bashrc");
    assert!(bashrc.exists(), ".bashrc should exist");
    let content = fs::read_to_string(&bashrc).unwrap();
    assert!(content.contains("rubynaut_switch"));
    assert!(content.contains("builtin cd"), ".bashrc should contain bash cd override");
}

#[test]
fn test_install_shell_hook_fish_creates_file() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());

    let result = rubynaut_core::install_shell_hook("fish".to_string());
    assert!(result.is_ok(), "install_shell_hook(fish) failed: {:?}", result.err());

    // Fish hook goes to config dir — check the result message for the actual path
    let msg = result.unwrap();
    assert!(msg.contains("rubynaut.fish"), "Result should mention rubynaut.fish: {msg}");

    // Find the actual file — it could be in XDG_CONFIG_HOME, ~/.config, or Library/Application Support
    let possible_paths = vec![
        home.path().join(".config/fish/conf.d/rubynaut.fish"),
        home.path().join("Library/Application Support/fish/conf.d/rubynaut.fish"),
    ];
    let found = possible_paths.iter().find(|p| p.exists());
    assert!(found.is_some(), "rubynaut.fish should exist in one of: {:?}", possible_paths);

    let content = fs::read_to_string(found.unwrap()).unwrap();
    assert!(content.contains("rubynaut_switch"));
    assert!(content.contains("--on-variable PWD"));
}

#[test]
fn test_install_shell_hook_idempotent() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());

    // Install twice
    rubynaut_core::install_shell_hook("zsh".to_string()).unwrap();
    let result = rubynaut_core::install_shell_hook("zsh".to_string());
    assert!(result.is_ok());
    assert!(result.unwrap().contains("already installed"));

    // Hook should appear only once
    let content = fs::read_to_string(home.path().join(".zshrc")).unwrap();
    let count = content.matches("rubynaut_switch()").count();
    assert_eq!(count, 1, "Hook should appear exactly once, found {count}");
}

#[test]
fn test_install_shell_hook_appends_to_existing_rc() {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());

    // Write pre-existing content
    let zshrc = home.path().join(".zshrc");
    let original = "# My original zshrc\nexport FOO=bar\n";
    fs::write(&zshrc, original).unwrap();

    rubynaut_core::install_shell_hook("zsh".to_string()).unwrap();

    let content = fs::read_to_string(&zshrc).unwrap();
    assert!(content.contains("My original zshrc"), "Original content should be preserved");
    assert!(content.contains("export FOO=bar"), "Original exports should be preserved");
    assert!(content.contains("rubynaut_switch"), "Hook should be appended");
}

#[test]
fn test_install_shell_hook_powershell_returns_error() {
    let result = rubynaut_core::install_shell_hook("powershell".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("manually"));
}

#[test]
fn test_install_shell_hook_unsupported_shell() {
    let result = rubynaut_core::install_shell_hook("tcsh".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unsupported shell"));
}
