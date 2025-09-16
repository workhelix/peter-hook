//! Secure template variable system
//!
//! This module provides a secure template system that expands predefined variables
//! in hook commands, working directories, and environment variables. Unlike shell
//! expansion, this system uses a whitelist of allowed variables and does not expose
//! arbitrary environment variables.

use crate::hooks::resolver::WorktreeContext;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Template resolver for predefined template variables
///
/// This resolver maintains a whitelist of allowed template variables and expands
/// them using the secure {VARIABLE_NAME} syntax. It does NOT expose arbitrary
/// environment variables for security reasons.
pub struct TemplateResolver {
    /// Available template variables (whitelist only)
    variables: HashMap<String, String>,
}

impl TemplateResolver {
    /// Create a new template resolver with standard variables
    ///
    /// Only predefined template variables are available for security.
    #[must_use]
    pub fn new(config_dir: &Path, working_dir: &Path) -> Self {
        let mut variables = HashMap::new();

        // Standard path variables
        variables.insert("HOOK_DIR".to_string(), config_dir.display().to_string());
        variables.insert("WORKING_DIR".to_string(), working_dir.display().to_string());

        // Git repository root
        if let Ok(repo_root) = find_git_root(config_dir) {
            variables.insert("REPO_ROOT".to_string(), repo_root.display().to_string());

            // Relative paths
            if let Ok(relative_config) = config_dir.strip_prefix(&repo_root) {
                variables.insert("HOOK_DIR_REL".to_string(), relative_config.display().to_string());
            }
            if let Ok(relative_working) = working_dir.strip_prefix(&repo_root) {
                variables.insert("WORKING_DIR_REL".to_string(), relative_working.display().to_string());
            }
        }

        // Project name (directory name of config dir)
        if let Some(project_name) = config_dir.file_name().and_then(|n| n.to_str()) {
            variables.insert("PROJECT_NAME".to_string(), project_name.to_string());
        }

        // User home directory (from HOME env var)
        if let Ok(home) = std::env::var("HOME") {
            variables.insert("HOME_DIR".to_string(), home);
        }

        // Initialize CHANGED_FILES variables as empty (will be set when files are provided)
        variables.insert("CHANGED_FILES".to_string(), String::new());
        variables.insert("CHANGED_FILES_LIST".to_string(), String::new());
        variables.insert("CHANGED_FILES_FILE".to_string(), String::new());

        Self { variables }
    }

    /// Create a new template resolver with worktree-aware variables
    ///
    /// Only predefined template variables are available for security.
    #[must_use]
    pub fn with_worktree_context(config_dir: &Path, working_dir: &Path, worktree_context: &WorktreeContext) -> Self {
        let mut variables = HashMap::new();

        // Standard path variables
        variables.insert("HOOK_DIR".to_string(), config_dir.display().to_string());
        variables.insert("WORKING_DIR".to_string(), working_dir.display().to_string());

        // Git repository variables using worktree context
        variables.insert("REPO_ROOT".to_string(), worktree_context.repo_root.display().to_string());
        variables.insert("COMMON_DIR".to_string(), worktree_context.common_dir.display().to_string());

        // Worktree-specific variables
        variables.insert("IS_WORKTREE".to_string(), worktree_context.is_worktree.to_string());
        if let Some(ref worktree_name) = worktree_context.worktree_name {
            variables.insert("WORKTREE_NAME".to_string(), worktree_name.clone());
        }

        // Relative paths
        if let Ok(relative_config) = config_dir.strip_prefix(&worktree_context.repo_root) {
            variables.insert("HOOK_DIR_REL".to_string(), relative_config.display().to_string());
        }
        if let Ok(relative_working) = working_dir.strip_prefix(&worktree_context.repo_root) {
            variables.insert("WORKING_DIR_REL".to_string(), relative_working.display().to_string());
        }

        // Project name (directory name of config dir)
        if let Some(project_name) = config_dir.file_name().and_then(|n| n.to_str()) {
            variables.insert("PROJECT_NAME".to_string(), project_name.to_string());
        }

        // User home directory (from HOME env var)
        if let Ok(home) = std::env::var("HOME") {
            variables.insert("HOME_DIR".to_string(), home);
        }

        // Initialize CHANGED_FILES variables as empty (will be set when files are provided)
        variables.insert("CHANGED_FILES".to_string(), String::new());
        variables.insert("CHANGED_FILES_LIST".to_string(), String::new());
        variables.insert("CHANGED_FILES_FILE".to_string(), String::new());

        Self { variables }
    }

    /// Resolve templates in a string using {VARIABLE_NAME} syntax
    ///
    /// # Errors
    ///
    /// Returns an error if template resolution fails
    pub fn resolve_string(&self, input: &str) -> Result<String> {
        let mut result = input.to_string();

        // Find all {VAR} patterns and replace them
        while let Some(start) = result.find('{') {
            let end = result[start..].find('}')
                .ok_or_else(|| anyhow::anyhow!("Unclosed template variable: {}", &result[start..]))?;
            let end = start + end;

            let var_name = &result[start + 1..end];
            let replacement = self.resolve_variable(var_name)
                .with_context(|| format!("Failed to resolve template variable: {var_name}"))?;

            result.replace_range(start..=end, &replacement);
        }

        Ok(result)
    }

    /// Resolve a single template variable
    fn resolve_variable(&self, var_name: &str) -> Result<String> {
        // Only allow predefined template variables from our whitelist
        self.variables.get(var_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Unknown template variable: {var_name}. Available variables: {}",
                self.get_available_variable_names().join(", ")))
    }

    /// Resolve templates in environment variables
    ///
    /// # Errors
    ///
    /// Returns an error if template resolution fails
    pub fn resolve_env(&self, env_map: &HashMap<String, String>) -> Result<HashMap<String, String>> {
        let mut resolved = HashMap::new();
        
        for (key, value) in env_map {
            let resolved_key = self.resolve_string(key)?;
            let resolved_value = self.resolve_string(value)?;
            resolved.insert(resolved_key, resolved_value);
        }
        
        Ok(resolved)
    }

    /// Resolve templates in command arguments
    ///
    /// # Errors
    ///
    /// Returns an error if template resolution fails
    pub fn resolve_command_args(&self, args: &[String]) -> Result<Vec<String>> {
        args.iter()
            .map(|arg| self.resolve_string(arg))
            .collect()
    }

    /// Set CHANGED_FILES template variables
    pub fn set_changed_files(&mut self, changed_files: &[PathBuf], changed_files_file_path: Option<&Path>) {
        // Space-delimited list
        let changed_space = changed_files
            .iter()
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");

        // Newline-delimited list
        let changed_list = changed_files
            .iter()
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n");

        self.variables.insert("CHANGED_FILES".to_string(), changed_space);
        self.variables.insert("CHANGED_FILES_LIST".to_string(), changed_list);
        self.variables.insert("CHANGED_FILES_FILE".to_string(),
            changed_files_file_path.map_or(String::new(), |p| p.display().to_string()));
    }

    /// Get all available template variables
    #[must_use]
    pub const fn get_available_variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

    /// Get sorted list of available variable names for error messages
    #[must_use]
    pub fn get_available_variable_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.variables.keys().cloned().collect();
        names.sort();
        names
    }
}

/// Find git repository root by walking up directories
fn find_git_root(start_dir: &Path) -> Result<PathBuf> {
    let mut current = start_dir;
    
    loop {
        if current.join(".git").exists() {
            return Ok(current.to_path_buf());
        }
        
        match current.parent() {
            Some(parent) => current = parent,
            None => return Err(anyhow::anyhow!("Not in a git repository")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_basic_templating() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let config_dir = temp_dir.path().join("project");
        std::fs::create_dir_all(&config_dir).expect("failed to create config dir");

        let template_resolver = TemplateResolver::new(&config_dir, &config_dir);

        let template = "Build in {HOOK_DIR}/target with project {PROJECT_NAME}";
        let result = template_resolver.resolve_string(template).expect("resolve_string");

        assert!(result.contains("Build in"));
        assert!(result.contains("/project/target"));
        assert!(result.contains("project project")); // PROJECT_NAME should be "project"
    }

    #[test]
    fn test_changed_files_templating() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let mut template_resolver = TemplateResolver::new(temp_dir.path(), temp_dir.path());

        // Set some changed files
        let changed_files = vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("tests/test.rs"),
        ];
        let temp_file = temp_dir.path().join("changed.txt");
        template_resolver.set_changed_files(&changed_files, Some(&temp_file));

        let result = template_resolver.resolve_string("Changed: {CHANGED_FILES}").expect("resolve_string");
        assert!(result.contains("src/main.rs tests/test.rs"));

        let result = template_resolver.resolve_string("{CHANGED_FILES_FILE}").expect("resolve_string");
        assert!(result.contains("changed.txt"));
    }

    #[test]
    fn test_command_args_templating() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let template_resolver = TemplateResolver::new(temp_dir.path(), temp_dir.path());

        let args = vec![
            "cargo".to_string(),
            "test".to_string(),
            "--manifest-path".to_string(),
            "{HOOK_DIR}/Cargo.toml".to_string(),
        ];

        let resolved_args = template_resolver.resolve_command_args(&args).expect("resolve_command_args");

        assert_eq!(resolved_args[0], "cargo");
        assert_eq!(resolved_args[1], "test");
        assert_eq!(resolved_args[2], "--manifest-path");
        assert!(resolved_args[3].ends_with("/Cargo.toml"));
        assert!(resolved_args[3].contains(temp_dir.path().to_str().unwrap()));
    }

    #[test]
    fn test_env_map_templating() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let template_resolver = TemplateResolver::new(temp_dir.path(), temp_dir.path());

        let mut env_map = HashMap::new();
        env_map.insert("PROJECT_PATH".to_string(), "{HOOK_DIR}".to_string());
        env_map.insert("BUILD_DIR".to_string(), "{HOOK_DIR}/target".to_string());

        let resolved_env = template_resolver.resolve_env(&env_map).expect("resolve_env");

        assert!(resolved_env["PROJECT_PATH"].contains(temp_dir.path().to_str().unwrap()));
        assert!(resolved_env["BUILD_DIR"].ends_with("/target"));
    }

    #[test]
    fn test_invalid_template() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let resolver = TemplateResolver::new(temp_dir.path(), temp_dir.path());

        // Unclosed template
        let result = resolver.resolve_string("{UNCLOSED");
        assert!(result.is_err());

        // Unknown variable
        let result = resolver.resolve_string("{UNKNOWN_VAR}");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("UNKNOWN_VAR"));
    }

    #[test]
    fn test_whitelist_security() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let resolver = TemplateResolver::new(temp_dir.path(), temp_dir.path());

        // Verify that arbitrary environment variables are NOT available
        std::env::set_var("DANGEROUS_VAR", "malicious_value");

        let result = resolver.resolve_string("{DANGEROUS_VAR}");
        assert!(result.is_err(), "Should not resolve arbitrary environment variables");

        // But predefined variables should work
        let result = resolver.resolve_string("{HOOK_DIR}").expect("Should resolve predefined variables");
        assert!(result.contains(temp_dir.path().to_str().unwrap()));

        std::env::remove_var("DANGEROUS_VAR");
    }
}