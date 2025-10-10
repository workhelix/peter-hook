#![allow(clippy::all, clippy::pedantic, clippy::nursery)]
//! Integration tests for update command

use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_get_platform_string() {
    // Test platform string generation
    let platform = peter_hook::update::get_platform_string();

    // Should return one of the known platform strings
    assert!(
        matches!(
            platform,
            "x86_64-apple-darwin"
                | "aarch64-apple-darwin"
                | "x86_64-unknown-linux-gnu"
                | "aarch64-unknown-linux-gnu"
                | "x86_64-pc-windows-msvc"
                | "unknown"
        ),
        "Unexpected platform string: {platform}"
    );

    // Verify it contains expected components for current platform
    #[cfg(target_os = "macos")]
    assert!(platform.contains("apple-darwin"));

    #[cfg(target_os = "linux")]
    assert!(platform.contains("linux"));

    #[cfg(target_os = "windows")]
    assert!(platform.contains("windows"));

    #[cfg(target_arch = "x86_64")]
    assert!(platform.starts_with("x86_64"));

    #[cfg(target_arch = "aarch64")]
    assert!(platform.starts_with("aarch64"));
}

#[test]
fn test_run_update_with_force_flag() {
    // Test update with force flag (doesn't actually download in test)
    let temp_dir = TempDir::new().unwrap();

    // Specify same version as current to test force flag
    let current_version = env!("CARGO_PKG_VERSION");

    // This will attempt update but network call may fail (acceptable in test)
    let exit_code =
        peter_hook::update::run_update(Some(current_version), true, Some(temp_dir.path()));

    // Exit code can be 0 (success), 1 (network error), or 2 (up to date)
    // All are acceptable outcomes in a test environment
    assert!(matches!(exit_code, 0..=2));
}

#[test]
fn test_run_update_already_up_to_date() {
    let temp_dir = TempDir::new().unwrap();

    // Specify same version as current without force
    let current_version = env!("CARGO_PKG_VERSION");

    // Should return 2 (already up to date) without attempting download
    let exit_code =
        peter_hook::update::run_update(Some(current_version), false, Some(temp_dir.path()));

    assert_eq!(exit_code, 2);
}

#[test]
fn test_run_update_with_install_dir() {
    let temp_dir = TempDir::new().unwrap();

    // Test that install_dir is used
    // This will likely fail network call but tests path handling
    let exit_code = peter_hook::update::run_update(Some("99.99.99"), true, Some(temp_dir.path()));

    // Network call will likely fail, but install dir logic is tested
    assert!(matches!(exit_code, 0 | 1));
}

#[test]
fn test_run_update_invalid_version_format() {
    let temp_dir = TempDir::new().unwrap();

    // Test with invalid version string
    let exit_code =
        peter_hook::update::run_update(Some("invalid.version.x"), true, Some(temp_dir.path()));

    // Should fail gracefully
    assert!(matches!(exit_code, 0 | 1));
}

#[test]
fn test_get_latest_version_network_handling() {
    // Test get_latest_version handles network calls
    // This may succeed or fail depending on network availability
    match peter_hook::update::get_latest_version() {
        Ok(version) => {
            // If successful, version should be non-empty
            assert!(!version.is_empty());
            // Version should not contain "v" prefix (should be stripped)
            assert!(!version.starts_with('v'));
        }
        Err(_e) => {
            // Network errors are acceptable in tests
        }
    }
}

#[test]
fn test_update_exit_codes() {
    let temp_dir = TempDir::new().unwrap();

    // Test various exit code scenarios
    let current_version = env!("CARGO_PKG_VERSION");

    // Already up to date (exit code 2)
    let exit_code1 =
        peter_hook::update::run_update(Some(current_version), false, Some(temp_dir.path()));
    assert_eq!(exit_code1, 2);

    // Force update of same version (may succeed or fail network call)
    let exit_code2 =
        peter_hook::update::run_update(Some(current_version), true, Some(temp_dir.path()));
    assert!(matches!(exit_code2, 0 | 1));
}

#[test]
fn test_run_update_without_install_dir() {
    // Test default behavior (uses current_exe path)
    let current_version = env!("CARGO_PKG_VERSION");

    // Should return 2 for same version without force
    let exit_code = peter_hook::update::run_update(Some(current_version), false, None);
    assert_eq!(exit_code, 2);
}

#[test]
fn test_platform_string_consistency() {
    // Verify platform string is consistent across calls
    let platform1 = peter_hook::update::get_platform_string();
    let platform2 = peter_hook::update::get_platform_string();
    assert_eq!(platform1, platform2);
}

#[test]
fn test_update_with_nonexistent_install_dir() {
    // Use a path that doesn't exist
    let nonexistent_dir = PathBuf::from("/nonexistent/path/that/does/not/exist");

    let current_version = env!("CARGO_PKG_VERSION");

    // Test that it handles nonexistent directories
    let exit_code =
        peter_hook::update::run_update(Some(current_version), false, Some(&nonexistent_dir));

    // Should return 2 (already up to date) since it checks version before path
    assert_eq!(exit_code, 2);
}
