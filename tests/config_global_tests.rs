//! Comprehensive tests for global configuration

use peter_hook::config::GlobalConfig;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_global_config_default() {
    let config = GlobalConfig::default();
    assert!(!config.security.allow_local);
}

#[test]
fn test_global_config_clone() {
    let config1 = GlobalConfig::default();
    let config2 = config1.clone();
    assert_eq!(config1, config2);
}

#[test]
fn test_global_config_eq() {
    let config1 = GlobalConfig::default();
    let config2 = GlobalConfig::default();
    assert_eq!(config1, config2);

    let mut config3 = GlobalConfig::default();
    config3.security.allow_local = true;
    assert_ne!(config1, config3);
}

#[test]
fn test_global_config_from_file_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("nonexistent.toml");

    let result = GlobalConfig::from_file(&config_path);
    assert!(result.is_ok());

    let config = result.unwrap();
    assert!(!config.security.allow_local);
}

#[test]
fn test_global_config_from_file_valid() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    fs::write(
        &config_path,
        r#"
[security]
allow_local = true
"#,
    )
    .unwrap();

    let result = GlobalConfig::from_file(&config_path);
    assert!(result.is_ok());

    let config = result.unwrap();
    assert!(config.security.allow_local);
}

#[test]
fn test_global_config_from_file_invalid_toml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    fs::write(&config_path, "[invalid toml").unwrap();

    let result = GlobalConfig::from_file(&config_path);
    assert!(result.is_err());
}

#[test]
fn test_global_config_config_path() {
    let result = GlobalConfig::config_path();
    assert!(result.is_ok());

    if let Ok(path) = result {
        assert!(path.to_string_lossy().contains("peter-hook") || path.to_string_lossy().contains("config"));
    }
}

#[test]
fn test_global_config_get_local_dir() {
    let result = GlobalConfig::get_local_dir();
    assert!(result.is_ok());

    if let Ok(path) = result {
        assert!(path.to_string_lossy().contains("local") || path.to_string_lossy().contains("peter-hook"));
    }
}

#[test]
fn test_global_config_serialization() {
    let config = GlobalConfig {
        security: peter_hook::config::SecurityConfig { allow_local: true },
    };

    let serialized = toml::to_string(&config);
    assert!(serialized.is_ok());

    if let Ok(s) = serialized {
        assert!(s.contains("allow_local"));
        assert!(s.contains("true"));
    }
}

#[test]
fn test_global_config_deserialization() {
    let toml_str = r#"
[security]
allow_local = false
"#;

    let result: Result<GlobalConfig, _> = toml::from_str(toml_str);
    assert!(result.is_ok());

    let config = result.unwrap();
    assert!(!config.security.allow_local);
}

#[test]
fn test_is_absolute_path_allowed_when_disabled() {
    let config = GlobalConfig::default();
    // Use a path that would be in .local/peter-hook
    let home = dirs::home_dir().unwrap();
    let test_path = home.join(".local/peter-hook/test.toml");

    let result = config.is_absolute_path_allowed(&test_path);
    // Should be false when allow_local is false
    assert!(result.is_ok());
    if let Ok(allowed) = result {
        assert!(!allowed);
    }
}

#[test]
fn test_is_absolute_path_allowed_when_enabled() {
    let mut config = GlobalConfig::default();
    config.security.allow_local = true;

    let home = dirs::home_dir().unwrap();
    let test_path = home.join(".local/peter-hook/test.toml");

    let result = config.is_absolute_path_allowed(&test_path);
    assert!(result.is_ok());
    if let Ok(allowed) = result {
        assert!(allowed);
    }
}

#[test]
fn test_security_config_clone() {
    let sec1 = peter_hook::config::SecurityConfig { allow_local: true };
    let sec2 = sec1.clone();
    assert_eq!(sec1, sec2);
}

#[test]
fn test_global_config_partial_eq() {
    let config1 = GlobalConfig::default();
    let mut config2 = GlobalConfig::default();

    assert_eq!(config1, config2);

    config2.security.allow_local = true;
    assert_ne!(config1, config2);
}
