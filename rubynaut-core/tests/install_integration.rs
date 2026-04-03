//! Integration tests for install_ruby, install_gem, uninstall_gem, bundle_install.
//! These run in a separate process so setting HOME is safe.
//! NOTE: Run with --test-threads=1 since tests share process-global env vars.
//! NOTE: These tests are Unix-only (shell scripts, chmod, tar).

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use tempfile::TempDir;

/// Set HOME and clean related env vars for a clean test environment.
fn setup_test_home(home: &std::path::Path) {
    std::env::set_var("HOME", home);
    std::env::remove_var("RUBYNAUT_DOWNLOAD_BASE_URL");
    std::env::remove_var("RUBYNAUT_GITHUB_API_BASE");
}

/// Create a fake Ruby installation at `home/.rubies/{version}` with a shell script
/// that handles --version, gem install/uninstall, and bundle install.
fn create_fake_ruby_install(home: &std::path::Path, version: &str) {
    let rubies = home.join(".rubies");
    let version_dir = rubies.join(version);
    let bin_dir = version_dir.join("bin");
    let lib_dir = version_dir.join("lib");
    let gems_dir = rubies.join("gems").join(version);

    fs::create_dir_all(&bin_dir).unwrap();
    fs::create_dir_all(&lib_dir).unwrap();
    fs::create_dir_all(&gems_dir).unwrap();

    let ruby_script = format!(r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "ruby {version} (2025-12-25) [arm64-darwin23]"
elif [ "$1" = "-S" ] && [ "$2" = "gem" ] && [ "$3" = "install" ]; then
  echo "Successfully installed $4"
elif [ "$1" = "-S" ] && [ "$2" = "gem" ] && [ "$3" = "uninstall" ]; then
  echo "Successfully uninstalled $4"
elif [ "$1" = "-S" ] && [ "$2" = "bundle" ] && [ "$3" = "install" ]; then
  echo "Bundle complete! 3 gems installed."
else
  echo "unknown: $@" >&2
  exit 1
fi
"#);

    let ruby_path = bin_dir.join("ruby");
    fs::write(&ruby_path, ruby_script).unwrap();
    fs::set_permissions(&ruby_path, fs::Permissions::from_mode(0o755)).unwrap();
}

/// Create a .tar.gz archive containing a fake Ruby installation layout.
/// Each call uses a unique staging directory to avoid conflicts.
fn create_fake_archive(dir: &std::path::Path, version: &str) -> std::path::PathBuf {
    let staging = dir.join(format!("staging-{version}"));
    let inner = staging.join("ruby-root");
    let bin_dir = inner.join("bin");
    let lib_dir = inner.join("lib");

    // Clean any previous staging
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&bin_dir).unwrap();
    fs::create_dir_all(&lib_dir).unwrap();

    let ruby_script = format!(r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "ruby {version} (2025-12-25) [arm64-darwin23]"
elif [ "$1" = "-S" ] && [ "$2" = "gem" ] && [ "$3" = "install" ]; then
  echo "Successfully installed $4"
else
  echo "unknown: $@" >&2
  exit 1
fi
"#);

    let ruby_path = bin_dir.join("ruby");
    fs::write(&ruby_path, &ruby_script).unwrap();
    fs::set_permissions(&ruby_path, fs::Permissions::from_mode(0o755)).unwrap();

    // Create a helper script with a runner shebang (to test fix_shebangs)
    let gem_script = format!("#!/Users/runner/hostedtoolcache/Ruby/{version}/x64/bin/ruby\nputs 'gem'\n");
    let gem_path = bin_dir.join("irb");
    fs::write(&gem_path, &gem_script).unwrap();
    fs::set_permissions(&gem_path, fs::Permissions::from_mode(0o755)).unwrap();

    let archive_path = dir.join(format!("ruby-{version}.tar.gz"));
    let _ = fs::remove_file(&archive_path);
    let output = Command::new("tar")
        .args(["czf", &archive_path.to_string_lossy(), "-C", &staging.to_string_lossy(), "ruby-root"])
        .output()
        .unwrap();
    assert!(output.status.success(), "tar failed: {}", String::from_utf8_lossy(&output.stderr));

    archive_path
}

// ==========================================
// install_ruby_from_archive happy path
// ==========================================

#[tokio::test]
async fn test_install_from_archive_creates_ruby_dir() {
    let home = TempDir::new().unwrap();
    setup_test_home(home.path());

    let archive_dir = TempDir::new().unwrap();
    let archive = create_fake_archive(archive_dir.path(), "4.0.2");

    let result = rubynaut_core::install_ruby_from_archive(
        "4.0.2".to_string(),
        archive.to_string_lossy().to_string(),
        None,
    ).await;

    assert!(result.is_ok(), "install_from_archive failed: {:?}", result.err());

    // Verify the installation layout
    let ruby_bin = home.path().join(".rubies/4.0.2/bin/ruby");
    assert!(ruby_bin.exists(), "ruby binary should exist");

    // Verify ruby is executable
    let output = Command::new(&ruby_bin).arg("--version").output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("4.0.2"));

    // Verify gems directory was created
    let gems_dir = home.path().join(".rubies/gems/4.0.2");
    assert!(gems_dir.exists(), "gems directory should exist");
}

#[tokio::test]
async fn test_install_from_archive_fixes_shebangs() {
    let home = TempDir::new().unwrap();
    setup_test_home(home.path());

    let archive_dir = TempDir::new().unwrap();
    let archive = create_fake_archive(archive_dir.path(), "3.3.6");

    rubynaut_core::install_ruby_from_archive(
        "3.3.6".to_string(),
        archive.to_string_lossy().to_string(),
        None,
    ).await.unwrap();

    // The irb script had a runner shebang — it should be fixed
    let irb_script = home.path().join(".rubies/3.3.6/bin/irb");
    if irb_script.exists() {
        let content = fs::read_to_string(&irb_script).unwrap();
        assert!(!content.contains("/runner/"), "shebang should be fixed, but got: {}", content.lines().next().unwrap_or(""));
        assert!(content.contains(".rubies/3.3.6/bin/ruby"), "shebang should point to installed ruby");
    }
}

#[tokio::test]
async fn test_install_from_archive_cleans_up_tmp_on_success() {
    let home = TempDir::new().unwrap();
    setup_test_home(home.path());

    let archive_dir = TempDir::new().unwrap();
    let archive = create_fake_archive(archive_dir.path(), "3.2.0");

    rubynaut_core::install_ruby_from_archive(
        "3.2.0".to_string(),
        archive.to_string_lossy().to_string(),
        None,
    ).await.unwrap();

    let tmp_dir = home.path().join(".rubies/.tmp-install");
    assert!(!tmp_dir.exists(), ".tmp-install should be cleaned up");
}

#[tokio::test]
async fn test_install_from_archive_already_installed() {
    let home = TempDir::new().unwrap();
    setup_test_home(home.path());

    // Pre-create the installed version
    create_fake_ruby_install(home.path(), "4.0.2");

    let archive_dir = TempDir::new().unwrap();
    let archive = create_fake_archive(archive_dir.path(), "4.0.2");

    let result = rubynaut_core::install_ruby_from_archive(
        "4.0.2".to_string(),
        archive.to_string_lossy().to_string(),
        None,
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already installed"));
}

// ==========================================
// install_gem / uninstall_gem
// ==========================================

#[tokio::test]
async fn test_install_gem_happy_path() {
    let home = TempDir::new().unwrap();
    setup_test_home(home.path());
    create_fake_ruby_install(home.path(), "4.0.2");

    let result = rubynaut_core::install_gem(
        "4.0.2".to_string(),
        "rails".to_string(),
        Some("7.2.0".to_string()),
    ).await;

    assert!(result.is_ok(), "install_gem failed: {:?}", result.err());
    assert!(result.unwrap().contains("Successfully installed"));
}

#[tokio::test]
async fn test_install_gem_nonexistent_ruby() {
    let home = TempDir::new().unwrap();
    setup_test_home(home.path());

    let result = rubynaut_core::install_gem(
        "99.99.99".to_string(),
        "rails".to_string(),
        None,
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not installed"));
}

#[tokio::test]
async fn test_uninstall_gem_happy_path() {
    let home = TempDir::new().unwrap();
    setup_test_home(home.path());
    create_fake_ruby_install(home.path(), "4.0.2");

    let result = rubynaut_core::uninstall_gem(
        "4.0.2".to_string(),
        "rails".to_string(),
    ).await;

    assert!(result.is_ok(), "uninstall_gem failed: {:?}", result.err());
}

// ==========================================
// bundle_install
// ==========================================

#[tokio::test]
async fn test_bundle_install_happy_path() {
    let home = TempDir::new().unwrap();
    setup_test_home(home.path());
    create_fake_ruby_install(home.path(), "4.0.2");

    let project_dir = TempDir::new().unwrap();
    fs::write(project_dir.path().join("Gemfile"), "source 'https://rubygems.org'\ngem 'rails'\n").unwrap();

    let result = rubynaut_core::bundle_install(
        "4.0.2".to_string(),
        project_dir.path().to_string_lossy().to_string(),
        None,
    ).await;

    assert!(result.is_ok(), "bundle_install failed: {:?}", result.err());
    assert!(result.unwrap().contains("Bundle complete"));
}

#[tokio::test]
async fn test_bundle_install_no_gemfile() {
    let home = TempDir::new().unwrap();
    setup_test_home(home.path());
    create_fake_ruby_install(home.path(), "4.0.2");

    let project_dir = TempDir::new().unwrap();
    // No Gemfile

    let result = rubynaut_core::bundle_install(
        "4.0.2".to_string(),
        project_dir.path().to_string_lossy().to_string(),
        None,
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No Gemfile"));
}

#[tokio::test]
async fn test_bundle_install_nonexistent_ruby() {
    let home = TempDir::new().unwrap();
    setup_test_home(home.path());

    let project_dir = TempDir::new().unwrap();
    fs::write(project_dir.path().join("Gemfile"), "gem 'rails'\n").unwrap();

    let result = rubynaut_core::bundle_install(
        "99.99.99".to_string(),
        project_dir.path().to_string_lossy().to_string(),
        None,
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not installed"));
}

// ==========================================
// install_ruby with wiremock (download + checksum)
// ==========================================

#[tokio::test]
async fn test_install_ruby_with_mock_server() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path_regex};

    let home = TempDir::new().unwrap();
    setup_test_home(home.path());

    // Create a fake archive
    let archive_dir = TempDir::new().unwrap();
    let archive_path = create_fake_archive(archive_dir.path(), "4.0.2");
    let archive_bytes = fs::read(&archive_path).unwrap();
    let checksum = rubynaut_core::sha256_hex(&archive_bytes);

    let mock_server = MockServer::start().await;

    // Mock the archive download
    Mock::given(method("GET"))
        .and(path_regex(r".*/ruby-4\.0\.2.*\.tar\.gz$"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(archive_bytes.clone()))
        .mount(&mock_server)
        .await;

    // Mock the checksum file
    Mock::given(method("GET"))
        .and(path_regex(r".*/ruby-4\.0\.2.*\.tar\.gz\.sha256$"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!("{checksum}  ruby-4.0.2.tar.gz\n")))
        .mount(&mock_server)
        .await;

    std::env::set_var("RUBYNAUT_DOWNLOAD_BASE_URL", mock_server.uri());

    let result = rubynaut_core::install_ruby("4.0.2".to_string(), None).await;

    std::env::remove_var("RUBYNAUT_DOWNLOAD_BASE_URL");

    assert!(result.is_ok(), "install_ruby failed: {:?}", result.err());

    let ruby_bin = home.path().join(".rubies/4.0.2/bin/ruby");
    assert!(ruby_bin.exists(), "ruby binary should exist after install");
}

#[tokio::test]
async fn test_install_ruby_checksum_mismatch() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path_regex};

    let home = TempDir::new().unwrap();
    setup_test_home(home.path());

    let archive_dir = TempDir::new().unwrap();
    let archive_path = create_fake_archive(archive_dir.path(), "3.3.6");
    let archive_bytes = fs::read(&archive_path).unwrap();

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path_regex(r".*\.tar\.gz$"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(archive_bytes))
        .mount(&mock_server)
        .await;

    // Return wrong checksum
    Mock::given(method("GET"))
        .and(path_regex(r".*\.sha256$"))
        .respond_with(ResponseTemplate::new(200).set_body_string("0000000000000000000000000000000000000000000000000000000000000000  file.tar.gz\n"))
        .mount(&mock_server)
        .await;

    std::env::set_var("RUBYNAUT_DOWNLOAD_BASE_URL", mock_server.uri());

    let result = rubynaut_core::install_ruby("3.3.6".to_string(), None).await;

    std::env::remove_var("RUBYNAUT_DOWNLOAD_BASE_URL");

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Checksum mismatch"));
}

// ==========================================
// check_for_update with wiremock
// ==========================================

#[tokio::test]
async fn test_check_for_update_newer_version() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path_regex};

    let mock_server = MockServer::start().await;

    let release_json = r#"{
        "tag_name": "v99.0.0",
        "assets": [
            {
                "name": "rubynaut-darwin-arm64",
                "browser_download_url": "https://example.com/rubynaut-darwin-arm64"
            },
            {
                "name": "rubynaut-linux-x64",
                "browser_download_url": "https://example.com/rubynaut-linux-x64"
            }
        ]
    }"#;

    Mock::given(method("GET"))
        .and(path_regex(r".*/releases/latest$"))
        .respond_with(ResponseTemplate::new(200).set_body_string(release_json))
        .mount(&mock_server)
        .await;

    std::env::set_var("RUBYNAUT_GITHUB_API_BASE", mock_server.uri());

    let result = rubynaut_core::check_for_update("0.1.0", "aguspe/rubynaut").await;

    std::env::remove_var("RUBYNAUT_GITHUB_API_BASE");

    assert!(result.is_ok());
    let update = result.unwrap();
    assert!(update.is_some(), "Should detect newer version");
    let (tag, _url) = update.unwrap();
    assert_eq!(tag, "v99.0.0");
}

#[tokio::test]
async fn test_check_for_update_already_latest() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path_regex};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path_regex(r".*/releases/latest$"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"tag_name": "v0.1.0", "assets": []}"#))
        .mount(&mock_server)
        .await;

    std::env::set_var("RUBYNAUT_GITHUB_API_BASE", mock_server.uri());

    let result = rubynaut_core::check_for_update("0.1.0", "aguspe/rubynaut").await;

    std::env::remove_var("RUBYNAUT_GITHUB_API_BASE");

    assert!(result.is_ok());
    assert!(result.unwrap().is_none(), "Should be up to date");
}

// ==========================================
// fetch_checksum with wiremock
// ==========================================

#[tokio::test]
async fn test_fetch_checksum_valid() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path_regex};

    let mock_server = MockServer::start().await;
    let expected_hash = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

    Mock::given(method("GET"))
        .and(path_regex(r".*\.sha256$"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!("{expected_hash}  file.tar.gz\n")))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let result = rubynaut_core::fetch_checksum(&client, &format!("{}/file.tar.gz", mock_server.uri())).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(expected_hash.to_string()));
}

#[tokio::test]
async fn test_fetch_checksum_404() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path_regex};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path_regex(r".*\.sha256$"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let result = rubynaut_core::fetch_checksum(&client, &format!("{}/file.tar.gz", mock_server.uri())).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
}

#[tokio::test]
async fn test_fetch_checksum_invalid_format() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path_regex};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path_regex(r".*\.sha256$"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not a valid checksum\n"))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let result = rubynaut_core::fetch_checksum(&client, &format!("{}/file.tar.gz", mock_server.uri())).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
}
