use crate::types::PlatformInfo;

pub fn detect_platform() -> Result<PlatformInfo, String> {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    let is_wsl = detect_wsl();
    let shell = detect_shell();
    let package_manager = detect_package_manager(&os);
    let has_c_compiler = detect_c_compiler();
    let existing_ruby_managers = detect_ruby_managers();

    Ok(PlatformInfo {
        os,
        arch,
        package_manager,
        shell,
        is_wsl,
        has_c_compiler,
        existing_ruby_managers,
    })
}

fn detect_wsl() -> bool {
    if std::env::consts::OS != "linux" {
        return false;
    }
    std::fs::read_to_string("/proc/version")
        .map(|v| v.to_lowercase().contains("microsoft"))
        .unwrap_or(false)
}

fn detect_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(windows) {
            "powershell".to_string()
        } else {
            "/bin/sh".to_string()
        }
    })
}

fn detect_package_manager(os: &str) -> Option<String> {
    match os {
        "macos" => {
            if which::which("brew").is_ok() {
                Some("homebrew".to_string())
            } else if which::which("port").is_ok() {
                Some("macports".to_string())
            } else {
                None
            }
        }
        "linux" => {
            let managers = [
                ("apt-get", "apt"),
                ("dnf", "dnf"),
                ("yum", "yum"),
                ("pacman", "pacman"),
                ("zypper", "zypper"),
                ("apk", "apk"),
            ];
            for (bin, name) in managers {
                if which::which(bin).is_ok() {
                    return Some(name.to_string());
                }
            }
            None
        }
        "windows" => Some("winget".to_string()),
        _ => None,
    }
}

fn detect_c_compiler() -> bool {
    which::which("cc").is_ok()
        || which::which("gcc").is_ok()
        || which::which("clang").is_ok()
}

fn detect_ruby_managers() -> Vec<String> {
    let managers = [
        ("rbenv", "rbenv"),
        ("rvm", "rvm"),
        ("mise", "mise"),
        ("asdf", "asdf"),
        ("chruby-exec", "chruby"),
    ];
    managers
        .iter()
        .filter(|(bin, _)| which::which(bin).is_ok())
        .map(|(_, name)| name.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_platform_succeeds() {
        let result = detect_platform();
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
        assert!(!info.shell.is_empty());
    }

    #[test]
    fn test_platform_os_is_known() {
        let info = detect_platform().unwrap();
        assert!(["macos", "linux", "windows"].contains(&info.os.as_str()));
    }

    #[test]
    fn test_platform_arch_is_known() {
        let info = detect_platform().unwrap();
        assert!(["aarch64", "x86_64", "x86"].contains(&info.arch.as_str()));
    }

    #[test]
    fn test_detect_shell_returns_something() {
        let shell = detect_shell();
        assert!(!shell.is_empty());
    }

    #[test]
    fn test_detect_wsl_false_on_macos() {
        if cfg!(target_os = "macos") {
            assert!(!detect_wsl());
        }
    }

    #[test]
    fn test_detect_c_compiler() {
        let _ = detect_c_compiler();
    }

    #[test]
    fn test_detect_package_manager_unknown_os() {
        let result = detect_package_manager("freebsd");
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_package_manager_windows() {
        let result = detect_package_manager("windows");
        assert_eq!(result, Some("winget".to_string()));
    }

    #[test]
    fn test_platform_info_serialization() {
        let info = PlatformInfo {
            os: "macos".into(),
            arch: "aarch64".into(),
            package_manager: Some("homebrew".into()),
            shell: "/bin/zsh".into(),
            is_wsl: false,
            has_c_compiler: true,
            existing_ruby_managers: vec!["rbenv".into()],
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"os\":\"macos\""));
    }

    #[test]
    fn test_detect_ruby_managers_returns_vec() {
        let managers = detect_ruby_managers();
        assert!(managers.len() <= 5);
    }
}
