//! Environment variable templating system

use crate::hooks::resolver::WorktreeContext;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

/// Template resolver for environment variables and dynamic values
pub struct TemplateResolver {
    /// Available template variables
    variables: HashMap<String, String>,
}

impl TemplateResolver {
    /// Create a new template resolver with standard variables
    ///
    /// # Errors
    ///
    /// Returns an error if environment variables cannot be accessed
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
        
        // Environment variables
        for (key, value) in env::vars() {
            variables.insert(key, value);
        }
        
        // Common derived variables
        if let Some(home) = variables.get("HOME") {
            variables.insert("HOME_DIR".to_string(), home.clone());
        }
        
        Self { variables }
    }

    /// Create a new template resolver with worktree-aware variables
    ///
    /// # Errors
    ///
    /// Returns an error if environment variables cannot be accessed
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
        
        // Environment variables
        for (key, value) in env::vars() {
            variables.insert(key, value);
        }
        
        // Common derived variables
        if let Some(home) = variables.get("HOME") {
            variables.insert("HOME_DIR".to_string(), home.clone());
        }
        
        Self { variables }
    }

    /// Resolve templates in a string
    ///
    /// # Errors
    ///
    /// Returns an error if template resolution fails
    pub fn resolve_string(&self, input: &str) -> Result<String> {
        let mut result = input.to_string();
        
        // Find all ${VAR} patterns and replace them
        while let Some(start) = result.find("${") {
            let end = result[start..].find('}')
                .ok_or_else(|| anyhow::anyhow!("Unclosed template variable: {}", &result[start..]))?;
            let end = start + end;
            
            let var_name = &result[start + 2..end];
            let replacement = self.resolve_variable(var_name)
                .with_context(|| format!("Failed to resolve template variable: {var_name}"))?;
            
            result.replace_range(start..=end, &replacement);
        }
        
        Ok(result)
    }

    /// Resolve a single template variable
    fn resolve_variable(&self, var_name: &str) -> Result<String> {
        // Handle shell-style expansions
        match var_name {
            // PWD basename (project directory name)
            "PWD##*/" => {
                if let Some(pwd) = self.variables.get("PWD") {
                    if let Some(basename) = Path::new(pwd).file_name().and_then(|n| n.to_str()) {
                        return Ok(basename.to_string());
                    }
                }
                Err(anyhow::anyhow!("Cannot resolve PWD basename"))
            }
            // Direct variable lookup
            _ => {
                self.variables.get(var_name)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Unknown template variable: {var_name}"))
            }
        }
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

    /// Get all available template variables
    #[must_use]
    pub const fn get_available_variables(&self) -> &HashMap<String, String> {
        &self.variables
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
        
        let template = "Build in ${HOOK_DIR}/target with project ${PROJECT_NAME}";
        let result = template_resolver.resolve_string(template).expect("resolve_string");
        
        assert!(result.contains("Build in"));
        assert!(result.contains("/project/target"));
        assert!(result.contains("project project")); // PROJECT_NAME should be "project"
    }

    #[test]
    fn test_env_variable_templating() {
        env::set_var("TEST_VAR", "test_value");
        
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let template_resolver = TemplateResolver::new(temp_dir.path(), temp_dir.path());
        
        let result = template_resolver.resolve_string("Value is ${TEST_VAR}").expect("resolve_string");
        assert_eq!(result, "Value is test_value");
        
        env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_command_args_templating() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let template_resolver = TemplateResolver::new(temp_dir.path(), temp_dir.path());
        
        let args = vec![
            "cargo".to_string(),
            "test".to_string(),
            "--manifest-path".to_string(),
            "${HOOK_DIR}/Cargo.toml".to_string(),
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
        env_map.insert("PROJECT_PATH".to_string(), "${HOOK_DIR}".to_string());
        env_map.insert("BUILD_DIR".to_string(), "${HOOK_DIR}/target".to_string());
        
        let resolved_env = template_resolver.resolve_env(&env_map).expect("resolve_env");
        
        assert!(resolved_env["PROJECT_PATH"].contains(temp_dir.path().to_str().unwrap()));
        assert!(resolved_env["BUILD_DIR"].ends_with("/target"));
    }

    #[test]
    fn test_invalid_template() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let resolver = TemplateResolver::new(temp_dir.path(), temp_dir.path());
        
        // Unclosed template
        let result = resolver.resolve_string("${UNCLOSED");
        assert!(result.is_err());
        
        // Unknown variable
        let result = resolver.resolve_string("${UNKNOWN_VAR}");
        assert!(result.is_err());
    }

    #[test]
    fn test_pwd_basename() {
        env::set_var("PWD", "/path/to/my-project");
        
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let template_resolver = TemplateResolver::new(temp_dir.path(), temp_dir.path());
        
        let result = template_resolver.resolve_string("Project: ${PWD##*/}").expect("resolve_string");
        assert_eq!(result, "Project: my-project");
        
        env::remove_var("PWD");
    }
}