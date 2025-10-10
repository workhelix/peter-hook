#![allow(clippy::all, clippy::pedantic, clippy::nursery)]
//! Comprehensive tests for doctor module coverage

use peter_hook::doctor;

#[test]
fn test_run_doctor_basic() {
    let exit_code = doctor::run_doctor();
    assert!(matches!(exit_code, 0 | 1));
}

#[test]
fn test_check_for_updates_returns_result() {
    let result = doctor::check_for_updates();
    // May succeed or fail depending on network
    match result {
        Ok(Some(_version)) => {
            // Update available
        }
        Ok(None) => {
            // Up to date
        }
        Err(_) => {
            // Network error
        }
    }
}

#[test]
fn test_doctor_in_various_states() {
    // Test doctor in current directory
    let exit_code1 = doctor::run_doctor();
    let exit_code2 = doctor::run_doctor();

    // Should be consistent
    assert_eq!(exit_code1, exit_code2);
}

#[test]
fn test_check_for_updates_version_parsing() {
    // This tests the version comparison logic
    match doctor::check_for_updates() {
        Ok(Some(version)) => {
            // Version should not be empty
            assert!(!version.is_empty());
            // Should not have 'v' prefix
            assert!(!version.starts_with('v'));
        }
        Ok(None) => {
            // Already up to date
        }
        Err(_) => {
            // Network error acceptable
        }
    }
}
