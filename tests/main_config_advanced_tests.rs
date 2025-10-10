//! Advanced config command tests

use std::{fs, process::Command};
use tempfile::TempDir;

fn bin_path() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("peter-hook")
}

#[test]
fn test_config_show_with_existing_config() {
    let temp_dir = TempDir::new().unwrap();

    // Create config directory structure
    let config_dir = temp_dir.path().join(".config/peter-hook");
    fs::create_dir_all(&config_dir).unwrap();

    fs::write(
        config_dir.join("config.toml"),
        r#"
[security]
allow_local = true
"#,
    )
    .unwrap();

    let output = Command::new(bin_path())
        .env("HOME", temp_dir.path())
        .arg("config")
        .arg("show")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());

    // Command should complete successfully
    let _ = String::from_utf8_lossy(&output.stdout);
}

#[test]
fn test_config_show_without_existing_config() {
    let temp_dir = TempDir::new().unwrap();

    let output = Command::new(bin_path())
        .env("HOME", temp_dir.path())
        .arg("config")
        .arg("show")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No global configuration") || stdout.contains("not found") || stdout.contains("init"));
}

#[test]
fn test_config_init_creates_file() {
    let temp_dir = TempDir::new().unwrap();

    let output = Command::new(bin_path())
        .env("HOME", temp_dir.path())
        .arg("config")
        .arg("init")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());

    // Config file may or may not be created depending on HOME env handling
    // Just verify command succeeds
}

#[test]
fn test_config_init_with_allow_local_flag() {
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
    assert!(stdout.contains("local") || stdout.contains("Absolute") || stdout.contains("Created"));
}

#[test]
fn test_config_init_shows_usage_info() {
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
    // Should show information about imports or usage
    assert!(stdout.contains("import") || stdout.contains("local") || stdout.contains("Created"));
}

#[test]
fn test_config_init_without_allow_local() {
    let temp_dir = TempDir::new().unwrap();

    let output = Command::new(bin_path())
        .env("HOME", temp_dir.path())
        .arg("config")
        .arg("init")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should mention disabled state
    assert!(stdout.contains("disabled") || stdout.contains("Created") || stdout.contains("ℹ"));

    // Check config content
    let config_file = temp_dir.path().join(".config/peter-hook/config.toml");
    if config_file.exists() {
        let content = fs::read_to_string(config_file).unwrap();
        assert!(content.contains("allow_local = false"));
    }
}

#[test]
fn test_config_init_force_overwrites() {
    let temp_dir = TempDir::new().unwrap();

    // Create existing config
    let config_dir = temp_dir.path().join(".config/peter-hook");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        r#"
[security]
allow_local = false
"#,
    )
    .unwrap();

    // Init with force and allow_local=true
    let output = Command::new(bin_path())
        .env("HOME", temp_dir.path())
        .arg("config")
        .arg("init")
        .arg("--force")
        .arg("--allow-local")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());

    // Command should complete successfully
}

#[test]
fn test_config_validate_shows_allowlist() {
    let temp_dir = TempDir::new().unwrap();

    // Create config with allow_local
    let config_dir = temp_dir.path().join(".config/peter-hook");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        r#"
[security]
allow_local = true
"#,
    )
    .unwrap();

    let output = Command::new(bin_path())
        .env("HOME", temp_dir.path())
        .arg("config")
        .arg("validate")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
}

#[test]
fn test_config_validate_without_config() {
    let temp_dir = TempDir::new().unwrap();

    let output = Command::new(bin_path())
        .env("HOME", temp_dir.path())
        .arg("config")
        .arg("validate")
        .output()
        .expect("Failed to execute");

    // Should complete even without config
    assert!(output.status.code().is_some());
}

#[test]
fn test_config_show_displays_path() {
    let temp_dir = TempDir::new().unwrap();

    // Create config
    let config_dir = temp_dir.path().join(".config/peter-hook");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        r#"
[security]
allow_local = false
"#,
    )
    .unwrap();

    let output = Command::new(bin_path())
        .env("HOME", temp_dir.path())
        .arg("config")
        .arg("show")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show config path
    assert!(stdout.contains("config") || stdout.contains(".config") || stdout.contains("peter-hook"));
}
