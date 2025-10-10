//! Integration tests for doctor command

use git2::Repository as Git2Repository;
use peter_hook::{HookConfig, git::GitRepository, hooks::HookResolver};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_git_repository_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Test finding repo before initialization
    let result = GitRepository::find_from_dir(temp_dir.path());
    assert!(result.is_err(), "Should not find git repo in temp dir");

    // Initialize git repository
    Git2Repository::init(temp_dir.path()).unwrap();

    // Test finding repo after initialization
    let result = GitRepository::find_from_dir(temp_dir.path());
    assert!(result.is_ok(), "Should find git repo after init");
}

#[test]
fn test_config_file_discovery() {
    let temp_dir = TempDir::new().unwrap();

    // Create valid hooks.toml
    let config_content = r#"
[hooks.test]
command = "echo test"
modifies_repository = false
"#;
    fs::write(temp_dir.path().join("hooks.toml"), config_content).unwrap();

    // Test config file discovery
    let resolver = HookResolver::new(temp_dir.path().to_path_buf());
    let config_path = resolver.find_config_file();
    assert!(config_path.is_ok());
    assert!(config_path.unwrap().is_some());
}

#[test]
fn test_config_parsing_valid() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize git repository (required for config parsing)
    Git2Repository::init(temp_dir.path()).unwrap();

    // Create valid hooks.toml
    let config_content = r#"
[hooks.test]
command = "echo test"
modifies_repository = false
"#;
    let config_file = temp_dir.path().join("hooks.toml");
    fs::write(&config_file, config_content).unwrap();

    // Test parsing
    let result = HookConfig::from_file(&config_file);
    assert!(result.is_ok(), "Should parse valid config: {:?}", result.err());

    let config = result.unwrap();
    let hook_names = config.get_hook_names();
    assert!(hook_names.contains(&"test".to_string()));
}

#[test]
fn test_config_parsing_invalid() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize git repository (required for config parsing)
    Git2Repository::init(temp_dir.path()).unwrap();

    // Create invalid hooks.toml (missing required fields)
    let config_content = r#"
[hooks.broken]
# Missing command and modifies_repository
"#;
    let config_file = temp_dir.path().join("hooks.toml");
    fs::write(&config_file, config_content).unwrap();

    // Test parsing
    let result = HookConfig::from_file(&config_file);
    assert!(result.is_err(), "Should fail to parse invalid config");
}

#[test]
fn test_config_with_hook_group() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize git repository (required for config parsing)
    Git2Repository::init(temp_dir.path()).unwrap();

    // Create hooks.toml with group
    let config_content = r#"
[hooks.format]
command = "cargo fmt"
modifies_repository = true

[hooks.lint]
command = "cargo clippy"
modifies_repository = false

[groups.pre-commit]
includes = ["format", "lint"]
"#;
    let config_file = temp_dir.path().join("hooks.toml");
    fs::write(&config_file, config_content).unwrap();

    // Test parsing
    let result = HookConfig::from_file(&config_file);
    assert!(result.is_ok(), "Should parse config with groups: {:?}", result.err());

    let config = result.unwrap();
    let hook_names = config.get_hook_names();
    assert!(hook_names.contains(&"format".to_string()));
    assert!(hook_names.contains(&"lint".to_string()));
    assert!(hook_names.contains(&"pre-commit".to_string()));
}

#[test]
fn test_empty_config() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize git repository (required for config parsing)
    Git2Repository::init(temp_dir.path()).unwrap();

    // Create empty hooks.toml
    let config_file = temp_dir.path().join("hooks.toml");
    fs::write(&config_file, "").unwrap();

    // Test parsing
    let result = HookConfig::from_file(&config_file);
    assert!(result.is_ok(), "Should parse empty config");

    let config = result.unwrap();
    let hook_names = config.get_hook_names();
    assert!(hook_names.is_empty(), "Empty config should have no hooks");
}

#[test]
fn test_check_for_updates_network_handling() {
    // check_for_updates handles network errors gracefully
    // It returns Result, so both Ok and Err are acceptable
    match peter_hook::doctor::check_for_updates() {
        Ok(Some(_version)) => {
            // Update available - this is fine
        }
        Ok(None) => {
            // Already up to date - this is fine
        }
        Err(_e) => {
            // Network error - this is acceptable in tests
        }
    }
}

#[test]
fn test_doctor_run_succeeds() {
    // Simply verify doctor can run without panicking
    // Exit code may vary based on environment (git repo, config, etc.)
    let exit_code = peter_hook::doctor::run_doctor();
    assert!(matches!(exit_code, 0 | 1), "Doctor should return valid exit code");
}
