//! Hook execution engine

use crate::config::HookCommand;
use crate::hooks::{ResolvedHook, ResolvedHooks};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::{Command, Stdio};

/// Executes resolved hooks
pub struct HookExecutor {
    /// Whether to run hooks in parallel when possible
    #[allow(dead_code)]
    parallel: bool,
}

/// Result of hook execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Exit code of the hook
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Whether the hook succeeded (exit code 0)
    pub success: bool,
}

/// Results from executing multiple hooks
#[derive(Debug, Clone)]
pub struct ExecutionResults {
    /// Results for each hook by name
    pub results: HashMap<String, ExecutionResult>,
    /// Overall success (all hooks succeeded)
    pub success: bool,
}

impl HookExecutor {
    /// Create a new hook executor
    #[must_use] 
    pub const fn new() -> Self {
        Self { parallel: false }
    }

    /// Create a new hook executor with parallel execution enabled
    #[must_use]
    pub const fn with_parallel() -> Self {
        Self { parallel: true }
    }

    /// Execute all resolved hooks
    /// 
    /// # Errors
    /// 
    /// Returns an error if any hook fails to execute due to system issues
    /// (not hook failure - that's reported in the results)
    pub fn execute(resolved_hooks: &ResolvedHooks) -> Result<ExecutionResults> {
        let mut results = HashMap::new();
        let mut overall_success = true;

        // For now, execute hooks sequentially
        // TODO: Implement parallel execution based on group configuration
        for (name, hook) in &resolved_hooks.hooks {
            let result = Self::execute_single_hook(name, hook)
                .with_context(|| format!("Failed to execute hook: {name}"))?;
            
            if !result.success {
                overall_success = false;
            }
            
            results.insert(name.clone(), result);
        }

        Ok(ExecutionResults {
            results,
            success: overall_success,
        })
    }

    /// Execute a single hook
    fn execute_single_hook(name: &str, hook: &ResolvedHook) -> Result<ExecutionResult> {
        let mut command = match &hook.definition.command {
            HookCommand::Shell(cmd) => {
                let mut command = Command::new("sh");
                command.args(["-c", cmd]);
                command
            }
            HookCommand::Args(args) => {
                if args.is_empty() {
                    return Err(anyhow::anyhow!("Empty command for hook: {}", name));
                }
                let mut command = Command::new(&args[0]);
                if args.len() > 1 {
                    command.args(&args[1..]);
                }
                command
            }
        };

        // Set working directory
        command.current_dir(&hook.working_directory);

        // Set environment variables
        if let Some(env) = &hook.definition.env {
            for (key, value) in env {
                command.env(key, value);
            }
        }

        // Configure stdio
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Execute the command
        let output = command.output()
            .with_context(|| format!("Failed to execute hook command: {name}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        let success = output.status.success();

        Ok(ExecutionResult {
            exit_code,
            stdout,
            stderr,
            success,
        })
    }
}

impl Default for HookExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionResults {
    /// Print a summary of execution results
    pub fn print_summary(&self) {
        println!("Hook Execution Summary:");
        println!("=====================");
        
        for (name, result) in &self.results {
            let status = if result.success { "✓" } else { "✗" };
            println!("{} {}: exit code {}", status, name, result.exit_code);
            
            if !result.stdout.is_empty() {
                println!("  stdout: {}", result.stdout.trim());
            }
            
            if !result.stderr.is_empty() {
                println!("  stderr: {}", result.stderr.trim());
            }
        }
        
        let status = if self.success { "SUCCESS" } else { "FAILURE" };
        println!("\nOverall: {status}");
    }

    /// Get failed hooks
    #[must_use] 
    pub fn get_failed_hooks(&self) -> Vec<&str> {
        self.results
            .iter()
            .filter_map(|(name, result)| {
                if result.success {
                    None
                } else {
                    Some(name.as_str())
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{HookDefinition, HookCommand};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_test_hook(command: HookCommand, workdir: Option<String>) -> ResolvedHook {
        ResolvedHook {
            definition: HookDefinition {
                command,
                workdir,
                env: None,
                description: None,
            },
            working_directory: std::env::temp_dir(),
            source_file: PathBuf::from("test.toml"),
        }
    }

    #[test]
    fn test_execute_shell_command_success() {
        let hook = create_test_hook(
            HookCommand::Shell("echo 'hello world'".to_string()),
            None,
        );
        
        let result = HookExecutor::execute_single_hook("test", &hook).unwrap();
        
        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout.trim(), "hello world");
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn test_execute_shell_command_failure() {
        let hook = create_test_hook(
            HookCommand::Shell("exit 1".to_string()),
            None,
        );
        
        let result = HookExecutor::execute_single_hook("test", &hook).unwrap();
        
        assert!(!result.success);
        assert_eq!(result.exit_code, 1);
    }

    #[test]
    fn test_execute_args_command() {
        let hook = create_test_hook(
            HookCommand::Args(vec!["echo".to_string(), "hello".to_string(), "args".to_string()]),
            None,
        );
        
        let result = HookExecutor::execute_single_hook("test", &hook).unwrap();
        
        assert!(result.success);
        assert_eq!(result.stdout.trim(), "hello args");
    }

    #[test]
    fn test_execute_multiple_hooks() {
        let mut hooks = HashMap::new();
        
        hooks.insert(
            "success".to_string(),
            create_test_hook(HookCommand::Shell("exit 0".to_string()), None),
        );
        
        hooks.insert(
            "failure".to_string(),
            create_test_hook(HookCommand::Shell("exit 1".to_string()), None),
        );
        
        let resolved_hooks = ResolvedHooks {
            config_path: PathBuf::from("test.toml"),
            hooks,
        };
        
        let results = HookExecutor::execute(&resolved_hooks).unwrap();
        
        assert!(!results.success); // Overall failure due to one failed hook
        assert_eq!(results.results.len(), 2);
        
        assert!(results.results["success"].success);
        assert!(!results.results["failure"].success);
        
        let failed = results.get_failed_hooks();
        assert_eq!(failed, vec!["failure"]);
    }
}