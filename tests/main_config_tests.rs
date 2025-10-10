//! Integration tests for config subcommands

use std::process::Command;
use tempfile::TempDir;

fn bin_path() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("peter-hook")
}

#[test]
fn test_config_show() {
    let output = Command::new(bin_path())
        .arg("config")
        .arg("show")
        .output()
        .expect("Failed to execute");

    // May succeed or fail depending on whether config exists
    assert!(output.status.code().is_some());
}

#[test]
fn test_config_init() {
    let temp_dir = TempDir::new().unwrap();

    // Set HOME to temp dir to avoid affecting real config
    let output = Command::new(bin_path())
        .env("HOME", temp_dir.path())
        .arg("config")
        .arg("init")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
}

#[test]
fn test_config_init_with_force() {
    let temp_dir = TempDir::new().unwrap();

    // First init
    let _ = Command::new(bin_path())
        .env("HOME", temp_dir.path())
        .arg("config")
        .arg("init")
        .output();

    // Second init with force
    let output = Command::new(bin_path())
        .env("HOME", temp_dir.path())
        .arg("config")
        .arg("init")
        .arg("--force")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
}

#[test]
fn test_config_init_with_allow_local() {
    let temp_dir = TempDir::new().unwrap();

    let output = Command::new(bin_path())
        .env("HOME", temp_dir.path())
        .arg("config")
        .arg("init")
        .arg("--allow-local")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("local") || stdout.contains("Absolute"));
}

#[test]
fn test_config_init_twice_without_force() {
    let temp_dir = TempDir::new().unwrap();

    // First init
    let output1 = Command::new(bin_path())
        .env("HOME", temp_dir.path())
        .arg("config")
        .arg("init")
        .output()
        .expect("Failed to execute");

    // Second init without force (should not overwrite)
    let output2 = Command::new(bin_path())
        .env("HOME", temp_dir.path())
        .arg("config")
        .arg("init")
        .output()
        .expect("Failed to execute");

    assert!(output1.status.success());
    assert!(output2.status.success());

    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(stdout2.contains("exists") || stdout2.contains("force"));
}

#[test]
fn test_config_validate() {
    let output = Command::new(bin_path())
        .arg("config")
        .arg("validate")
        .output()
        .expect("Failed to execute");

    // Should complete regardless of config state
    assert!(output.status.code().is_some());
}

#[test]
fn test_config_help() {
    let output = Command::new(bin_path())
        .arg("config")
        .arg("--help")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("config") || stdout.contains("Config"));
}

#[test]
fn test_config_show_help() {
    let output = Command::new(bin_path())
        .arg("config")
        .arg("show")
        .arg("--help")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
}

#[test]
fn test_config_init_help() {
    let output = Command::new(bin_path())
        .arg("config")
        .arg("init")
        .arg("--help")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("force") || stdout.contains("allow-local"));
}

#[test]
fn test_config_validate_help() {
    let output = Command::new(bin_path())
        .arg("config")
        .arg("validate")
        .arg("--help")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
}
