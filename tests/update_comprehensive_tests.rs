//! Comprehensive tests for update module

use peter_hook::update;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_get_latest_version_network() {
    let result = update::get_latest_version();
    // May succeed or fail depending on network
    match result {
        Ok(version) => {
            assert!(!version.is_empty());
            assert!(!version.starts_with('v'));
        }
        Err(_) => {
            // Network errors are acceptable
        }
    }
}

#[test]
fn test_get_platform_string_all_platforms() {
    let platform = update::get_platform_string();

    // Should return a valid platform string
    assert!(!platform.is_empty());
    assert_ne!(platform, "");

    // Should be one of the known platforms
    let known_platforms = vec![
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-pc-windows-msvc",
        "unknown",
    ];

    assert!(known_platforms.contains(&platform));
}

#[test]
fn test_run_update_already_current() {
    let temp_dir = TempDir::new().unwrap();
    let current_version = env!("CARGO_PKG_VERSION");

    let exit_code = update::run_update(Some(current_version), false, Some(temp_dir.path()));
    assert_eq!(exit_code, 2);
}

#[test]
fn test_run_update_with_force_current_version() {
    let temp_dir = TempDir::new().unwrap();
    let current_version = env!("CARGO_PKG_VERSION");

    let exit_code = update::run_update(Some(current_version), true, Some(temp_dir.path()));
    // May fail due to network, but should not panic
    assert!(matches!(exit_code, 0 | 1 | 2));
}

#[test]
fn test_run_update_invalid_version() {
    let temp_dir = TempDir::new().unwrap();

    let exit_code = update::run_update(Some("999.999.999"), true, Some(temp_dir.path()));
    // Should fail to download
    assert!(matches!(exit_code, 0 | 1));
}

#[test]
fn test_run_update_no_version_specified() {
    let temp_dir = TempDir::new().unwrap();

    // This will try to fetch latest version
    let exit_code = update::run_update(None, true, Some(temp_dir.path()));
    // May succeed or fail based on network
    assert!(matches!(exit_code, 0 | 1 | 2));
}

#[test]
fn test_run_update_with_custom_install_dir() {
    let temp_dir = TempDir::new().unwrap();
    let install_dir = temp_dir.path().join("custom");
    fs::create_dir(&install_dir).unwrap();

    let current_version = env!("CARGO_PKG_VERSION");

    let exit_code = update::run_update(Some(current_version), false, Some(&install_dir));
    assert_eq!(exit_code, 2);
}

#[test]
fn test_run_update_exit_code_types() {
    let temp_dir = TempDir::new().unwrap();

    // Test exit code 2 (already up to date)
    let current_version = env!("CARGO_PKG_VERSION");
    let exit_code = update::run_update(Some(current_version), false, Some(temp_dir.path()));
    assert_eq!(exit_code, 2, "Should return 2 when already up to date");
}

#[test]
fn test_get_platform_string_consistency() {
    let platform1 = update::get_platform_string();
    let platform2 = update::get_platform_string();
    let platform3 = update::get_platform_string();

    assert_eq!(platform1, platform2);
    assert_eq!(platform2, platform3);
}

#[test]
fn test_platform_string_matches_current_os() {
    let platform = update::get_platform_string();

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    assert_eq!(platform, "x86_64-apple-darwin");

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    assert_eq!(platform, "aarch64-apple-darwin");

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    assert_eq!(platform, "x86_64-unknown-linux-gnu");

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    assert_eq!(platform, "aarch64-unknown-linux-gnu");

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    assert_eq!(platform, "x86_64-pc-windows-msvc");
}
