#![allow(clippy::items_after_test_module)]
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
    /// Whether this hook modifies the repository contents
    /// If true, this hook cannot run in parallel with other hooks
    #[serde(default)]
    pub modifies_repository: bool,
    /// File patterns that trigger this hook (glob patterns)
    /// If specified, hook only runs if changed files match these patterns
    pub files: Option<Vec<String>>,
    /// Run this hook always, regardless of file changes
    #[serde(default)]
    pub run_always: bool,
    /// Hooks that must complete successfully before this hook runs
    pub depends_on: Option<Vec<String>>,
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

/// Execution strategy for hook groups
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionStrategy {
    /// Run all hooks sequentially (default)
    #[default]
    Sequential,
    /// Run hooks in parallel where safe (respects `modifies_repository` flag)
    Parallel,
    /// Force parallel execution (unsafe - ignores `modifies_repository`)
    ForceParallel,
}

/// Group of hooks that run together
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HookGroup {
    /// List of hooks or other groups to include
    pub includes: Vec<String>,
    /// Description of what this group does
    pub description: Option<String>,
    /// Execution strategy for this group
    #[serde(default)]
    pub execution: ExecutionStrategy,
    /// Whether to run hooks in parallel (deprecated - use execution field)
    /// Kept for backward compatibility
    #[serde(skip_serializing)]
    pub parallel: Option<bool>,
}

impl HookGroup {
    /// Get the effective execution strategy, handling backward compatibility
    #[must_use]
    pub fn get_execution_strategy(&self) -> ExecutionStrategy {
        // Handle backward compatibility with deprecated `parallel` field
        self.parallel.map_or_else(
            || self.execution,
            |parallel| {
                if parallel {
                    ExecutionStrategy::Parallel
                } else {
                    ExecutionStrategy::Sequential
                }
            },
        )
    }
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
        assert_eq!(
            hook.command,
            HookCommand::Shell("echo 'hello world'".to_string())
        );
        assert_eq!(hook.description, Some("A simple test hook".to_string()));
        assert!(!hook.modifies_repository); // Default should be false
        assert!(hook.files.is_none()); // Default should be None
        assert!(!hook.run_always); // Default should be false
        assert!(hook.depends_on.is_none()); // Default should be None
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
        assert_eq!(group.get_execution_strategy(), ExecutionStrategy::Parallel);
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

    #[test]
    fn test_repository_modifying_hook() {
        let toml = r#"
[hooks.format]
command = "cargo fmt"
description = "Format Rust code"
modifies_repository = true
"#;

        let config = HookConfig::parse(toml).unwrap();
        let hooks = config.hooks.unwrap();
        let hook = &hooks["format"];

        assert!(hook.modifies_repository);
        assert_eq!(hook.description, Some("Format Rust code".to_string()));
    }

    #[test]
    fn test_execution_strategies() {
        let toml = r#"
[groups.sequential]
includes = ["test1", "test2"]
execution = "sequential"

[groups.parallel]
includes = ["test1", "test2"]  
execution = "parallel"

[groups.force-parallel]
includes = ["test1", "test2"]
execution = "force-parallel"

[groups.backward-compat]
includes = ["test1", "test2"]
parallel = true
"#;

        let config = HookConfig::parse(toml).unwrap();
        let groups = config.groups.unwrap();

        assert_eq!(
            groups["sequential"].get_execution_strategy(),
            ExecutionStrategy::Sequential
        );
        assert_eq!(
            groups["parallel"].get_execution_strategy(),
            ExecutionStrategy::Parallel
        );
        assert_eq!(
            groups["force-parallel"].get_execution_strategy(),
            ExecutionStrategy::ForceParallel
        );
        assert_eq!(
            groups["backward-compat"].get_execution_strategy(),
            ExecutionStrategy::Parallel
        );
    }

    #[test]
    fn test_file_pattern_hook() {
        let toml = r#"
[hooks.rust-lint]
command = "cargo clippy"
description = "Lint Rust code"
modifies_repository = false
files = ["**/*.rs", "Cargo.toml"]

[hooks.js-lint]
command = "eslint src/"
description = "Lint JavaScript code"
modifies_repository = false
files = ["**/*.js", "**/*.ts", "package.json"]
run_always = false

[hooks.format-all]
command = "prettier --write ."
description = "Format all files"
modifies_repository = true
run_always = true
"#;
        
        let config = HookConfig::parse(toml).unwrap();
        let hooks = config.hooks.unwrap();
        
        let rust_hook = &hooks["rust-lint"];
        assert_eq!(rust_hook.files, Some(vec!["**/*.rs".to_string(), "Cargo.toml".to_string()]));
        assert!(!rust_hook.run_always);
        
        let js_hook = &hooks["js-lint"];
        assert_eq!(js_hook.files, Some(vec!["**/*.js".to_string(), "**/*.ts".to_string(), "package.json".to_string()]));
        assert!(!js_hook.run_always);
        
        let format_hook = &hooks["format-all"];
        assert!(format_hook.run_always);
        assert!(format_hook.files.is_none()); // run_always hooks don't need file patterns
    }

    #[test]
    fn test_hook_dependencies_and_templating() {
        let toml = r#"
[hooks.format]
command = "cargo fmt --manifest-path=${HOOK_DIR}/Cargo.toml"
description = "Format code with template"
modifies_repository = true
env = { PROJECT_ROOT = "${REPO_ROOT}", BUILD_MODE = "debug" }

[hooks.lint]
command = ["cargo", "clippy", "--manifest-path=${HOOK_DIR}/Cargo.toml"]
description = "Lint after formatting"
modifies_repository = false
depends_on = ["format"]
files = ["**/*.rs"]

[hooks.test]
command = "cd ${WORKING_DIR} && cargo test"
description = "Test with working directory template"
modifies_repository = false
depends_on = ["lint"]
workdir = "${REPO_ROOT}/target"
"#;
        
        let config = HookConfig::parse(toml).unwrap();
        let hooks = config.hooks.unwrap();
        
        // Test format hook
        let format_hook = &hooks["format"];
        assert!(format_hook.command.to_string().contains("${HOOK_DIR}"));
        assert!(format_hook.modifies_repository);
        assert_eq!(format_hook.env, Some([
            ("PROJECT_ROOT".to_string(), "${REPO_ROOT}".to_string()),
            ("BUILD_MODE".to_string(), "debug".to_string()),
        ].iter().cloned().collect()));
        
        // Test lint hook
        let lint_hook = &hooks["lint"];
        assert_eq!(lint_hook.depends_on, Some(vec!["format".to_string()]));
        assert_eq!(lint_hook.files, Some(vec!["**/*.rs".to_string()]));
        
        // Test test hook
        let test_hook = &hooks["test"];
        assert_eq!(test_hook.depends_on, Some(vec!["lint".to_string()]));
        assert_eq!(test_hook.workdir, Some("${REPO_ROOT}/target".to_string()));
    }
}

impl std::fmt::Display for HookCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shell(cmd) => write!(f, "{cmd}"),
            Self::Args(args) => write!(f, "{}", args.join(" ")),
        }
    }
}
