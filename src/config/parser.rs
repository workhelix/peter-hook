//! Configuration parsing for git hooks

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Represents a hook configuration file (hooks.toml)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HookConfig {
    /// Individual hook definitions
    pub hooks: Option<HashMap<String, HookDefinition>>,
    /// Hook groups that combine multiple hooks
    pub groups: Option<HashMap<String, HookGroup>>,
}

/// Definition of an individual hook
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HookDefinition {
    /// Command to execute (either as string or array)
    pub command: HookCommand,
    /// Working directory override (defaults to config file directory)
    pub workdir: Option<String>,
    /// Environment variables to set
    pub env: Option<HashMap<String, String>>,
    /// Description of what this hook does
    pub description: Option<String>,
}

/// Command specification for a hook
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum HookCommand {
    /// Shell command as a string
    Shell(String),
    /// Execve-style command as array
    Args(Vec<String>),
}

/// Group of hooks that run together
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HookGroup {
    /// List of hooks or other groups to include
    pub includes: Vec<String>,
    /// Description of what this group does
    pub description: Option<String>,
    /// Whether to run hooks in parallel (default: false)
    pub parallel: Option<bool>,
}

impl HookConfig {
    /// Parse a hooks.toml file from the given path
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;
        
        Self::parse(&content)
    }

    /// Parse a hooks.toml configuration from a string
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML content cannot be parsed
    pub fn parse(content: &str) -> Result<Self> {
        toml::from_str(content).context("Failed to parse TOML configuration")
    }

    /// Get all hook names defined in this configuration
    #[must_use]
    pub fn get_hook_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        
        if let Some(hooks) = &self.hooks {
            names.extend(hooks.keys().cloned());
        }
        
        if let Some(groups) = &self.groups {
            names.extend(groups.keys().cloned());
        }
        
        names.sort();
        names
    }

    /// Check if a hook or group exists
    #[must_use]
    pub fn has_hook(&self, name: &str) -> bool {
        self.hooks.as_ref().is_some_and(|h| h.contains_key(name))
            || self.groups.as_ref().is_some_and(|g| g.contains_key(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_hook() {
        let toml = r#"
[hooks.test]
command = "echo 'hello world'"
description = "A simple test hook"
"#;
        
        let config = HookConfig::parse(toml).unwrap();
        assert!(config.hooks.is_some());
        
        let hooks = config.hooks.unwrap();
        assert!(hooks.contains_key("test"));
        
        let hook = &hooks["test"];
        assert_eq!(hook.command, HookCommand::Shell("echo 'hello world'".to_string()));
        assert_eq!(hook.description, Some("A simple test hook".to_string()));
    }

    #[test]
    fn test_parse_array_command() {
        let toml = r#"
[hooks.lint]
command = ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]
"#;
        
        let config = HookConfig::parse(toml).unwrap();
        let hooks = config.hooks.unwrap();
        let hook = &hooks["lint"];
        
        assert_eq!(
            hook.command,
            HookCommand::Args(vec![
                "cargo".to_string(),
                "clippy".to_string(),
                "--all-targets".to_string(),
                "--".to_string(),
                "-D".to_string(),
                "warnings".to_string(),
            ])
        );
    }

    #[test]
    fn test_parse_hook_group() {
        let toml = r#"
[groups.python-lint]
includes = ["python.ruff", "python.type-check"]
description = "Python linting and type checking"
parallel = true
"#;
        
        let config = HookConfig::parse(toml).unwrap();
        assert!(config.groups.is_some());
        
        let groups = config.groups.unwrap();
        assert!(groups.contains_key("python-lint"));
        
        let group = &groups["python-lint"];
        assert_eq!(group.includes, vec!["python.ruff", "python.type-check"]);
        assert_eq!(group.parallel, Some(true));
    }

    #[test]
    fn test_get_hook_names() {
        let toml = r#"
[hooks.test1]
command = "echo test1"

[hooks.test2]
command = "echo test2"

[groups.all-tests]
includes = ["test1", "test2"]
"#;
        
        let config = HookConfig::parse(toml).unwrap();
        let names = config.get_hook_names();
        
        assert_eq!(names, vec!["all-tests", "test1", "test2"]);
    }
}