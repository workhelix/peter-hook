//! Hierarchical hook resolution system

use crate::config::{HookConfig, HookDefinition, HookGroup};
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Resolves hooks hierarchically from the filesystem
pub struct HookResolver {
    /// Current working directory where hook resolution starts
    current_dir: PathBuf,
}

/// Result of hook resolution containing all applicable hooks
#[derive(Debug, Clone)]
pub struct ResolvedHooks {
    /// The configuration file that was used
    pub config_path: PathBuf,
    /// Individual hooks to execute
    pub hooks: HashMap<String, ResolvedHook>,
}

/// A resolved hook ready for execution
#[derive(Debug, Clone)]
pub struct ResolvedHook {
    /// Original hook definition
    pub definition: HookDefinition,
    /// Directory where this hook should run
    pub working_directory: PathBuf,
    /// Source configuration file
    pub source_file: PathBuf,
}

impl HookResolver {
    /// Create a new hook resolver for the current directory
    pub fn new<P: AsRef<Path>>(current_dir: P) -> Self {
        Self {
            current_dir: current_dir.as_ref().to_path_buf(),
        }
    }

    /// Find the nearest hooks.toml file by walking up the directory tree
    ///
    /// # Errors
    ///
    /// Returns an error if there are filesystem access issues
    pub fn find_config_file(&self) -> Result<Option<PathBuf>> {
        let mut current = self.current_dir.as_path();
        
        loop {
            let config_path = current.join("hooks.toml");
            if config_path.exists() {
                return Ok(Some(config_path));
            }
            
            match current.parent() {
                Some(parent) => current = parent,
                None => return Ok(None),
            }
        }
    }

    /// Resolve hooks for a given git event (e.g., "pre-commit", "pre-push")
    ///
    /// # Errors
    ///
    /// Returns an error if config file parsing fails or filesystem access issues occur
    pub fn resolve_hooks(&self, event: &str) -> Result<Option<ResolvedHooks>> {
        let Some(config_path) = self.find_config_file()? else { return Ok(None) };

        let config = HookConfig::from_file(&config_path)?;
        let config_dir = config_path
            .parent()
            .context("Config file has no parent directory")?;

        // Look for hooks that match the event name
        let mut resolved_hooks = HashMap::new();
        
        // First, try to find a hook or group with the exact event name
        if let Some(hooks) = &config.hooks {
            if let Some(hook_def) = hooks.get(event) {
                let resolved = ResolvedHook {
                    definition: hook_def.clone(),
                    working_directory: Self::resolve_working_directory(hook_def, config_dir),
                    source_file: config_path.clone(),
                };
                resolved_hooks.insert(event.to_string(), resolved);
            }
        }

        if let Some(groups) = &config.groups {
            if let Some(group) = groups.get(event) {
                self.resolve_group(group, &config, config_dir, &config_path, &mut resolved_hooks)?;
            }
        }

        if resolved_hooks.is_empty() {
            return Ok(None);
        }

        Ok(Some(ResolvedHooks {
            config_path,
            hooks: resolved_hooks,
        }))
    }

    /// Resolve all hooks in a group recursively
    ///
    /// # Errors
    ///
    /// Returns an error if hook resolution fails
    fn resolve_group(
        &self,
        group: &HookGroup,
        config: &HookConfig,
        config_dir: &Path,
        config_path: &Path,
        resolved_hooks: &mut HashMap<String, ResolvedHook>,
    ) -> Result<()> {
        let mut visited = HashSet::new();
        self.resolve_group_recursive(group, config, config_dir, config_path, resolved_hooks, &mut visited)
    }

    /// Internal recursive group resolution to handle nested groups
    ///
    /// # Errors
    ///
    /// Returns an error if hook resolution fails
    #[allow(clippy::only_used_in_recursion)]
    fn resolve_group_recursive(
        &self,
        group: &HookGroup,
        config: &HookConfig,
        config_dir: &Path,
        config_path: &Path,
        resolved_hooks: &mut HashMap<String, ResolvedHook>,
        visited: &mut HashSet<String>,
    ) -> Result<()> {
        for include in &group.includes {
            if visited.contains(include) {
                continue; // Avoid infinite loops
            }
            visited.insert(include.clone());

            // Try to resolve as individual hook first
            if let Some(hooks) = &config.hooks {
                if let Some(hook_def) = hooks.get(include) {
                    let resolved = ResolvedHook {
                        definition: hook_def.clone(),
                        working_directory: Self::resolve_working_directory(hook_def, config_dir),
                        source_file: config_path.to_path_buf(),
                    };
                    resolved_hooks.insert(include.clone(), resolved);
                    continue;
                }
            }

            // Try to resolve as group
            if let Some(groups) = &config.groups {
                if let Some(nested_group) = groups.get(include) {
                    self.resolve_group_recursive(
                        nested_group,
                        config,
                        config_dir,
                        config_path,
                        resolved_hooks,
                        visited,
                    )?;
                }
            }
        }

        Ok(())
    }

    /// Resolve the working directory for a hook
    fn resolve_working_directory(
        hook_def: &HookDefinition,
        config_dir: &Path,
    ) -> PathBuf {
        hook_def.workdir.as_ref().map_or_else(|| config_dir.to_path_buf(), |workdir| {
                let path = Path::new(workdir);
                if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    config_dir.join(path)
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::HookCommand;
    use tempfile::TempDir;

    fn create_test_config(dir: &Path, content: &str) -> PathBuf {
        let config_path = dir.join("hooks.toml");
        std::fs::write(&config_path, content).unwrap();
        config_path
    }

    #[test]
    fn test_find_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        
        // Create nested directory structure
        let nested = root.join("projects/frontend/src");
        std::fs::create_dir_all(&nested).unwrap();
        
        // Create config file in middle directory
        create_test_config(&root.join("projects"), "[hooks]");
        
        let resolver = HookResolver::new(&nested);
        let config_path = resolver.find_config_file().unwrap().unwrap();
        
        assert_eq!(config_path, root.join("projects/hooks.toml"));
    }

    #[test]
    fn test_resolve_simple_hook() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        
        let config_content = r#"
[hooks.pre-commit]
command = "echo 'running pre-commit'"
description = "Simple pre-commit hook"
"#;
        
        create_test_config(root, config_content);
        
        let resolver = HookResolver::new(root);
        let result = resolver.resolve_hooks("pre-commit").unwrap().unwrap();
        
        assert_eq!(result.hooks.len(), 1);
        assert!(result.hooks.contains_key("pre-commit"));
        
        let hook = &result.hooks["pre-commit"];
        assert_eq!(hook.working_directory, root);
        assert_eq!(
            hook.definition.command,
            HookCommand::Shell("echo 'running pre-commit'".to_string())
        );
    }

    #[test]
    fn test_resolve_hook_group() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        
        let config_content = r#"
[hooks.lint]
command = "cargo clippy"

[hooks.test]
command = "cargo test"

[groups.pre-commit]
includes = ["lint", "test"]
"#;
        
        create_test_config(root, config_content);
        
        let resolver = HookResolver::new(root);
        let result = resolver.resolve_hooks("pre-commit").unwrap().unwrap();
        
        assert_eq!(result.hooks.len(), 2);
        assert!(result.hooks.contains_key("lint"));
        assert!(result.hooks.contains_key("test"));
    }

    #[test]
    fn test_no_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        
        let resolver = HookResolver::new(root);
        let result = resolver.resolve_hooks("pre-commit").unwrap();
        
        assert!(result.is_none());
    }
}