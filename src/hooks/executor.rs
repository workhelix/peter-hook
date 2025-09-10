//! Hook execution engine

use crate::config::{ExecutionStrategy, HookCommand, TemplateResolver};
use crate::hooks::{DependencyResolver, ResolvedHook, ResolvedHooks};
use crate::output::formatter;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::git::FilePatternMatcher;

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

    /// Execute all resolved hooks using their configured execution strategy
    ///
    /// # Errors
    ///
    /// Returns an error if any hook fails to execute due to system issues
    /// (not hook failure - that's reported in the results)
    pub fn execute(resolved_hooks: &ResolvedHooks) -> Result<ExecutionResults> {
        // Check if we need dependency resolution
        let needs_dependencies = resolved_hooks.hooks.values()
            .any(|hook| hook.definition.depends_on.is_some());

        if needs_dependencies {
            Self::execute_with_dependencies(resolved_hooks)
        } else {
            Self::execute_with_strategy(resolved_hooks, resolved_hooks.execution_strategy)
        }
    }

    /// Execute hooks with a specific execution strategy
    ///
    /// # Errors
    ///
    /// Returns an error if any hook fails to execute due to system issues
    pub fn execute_with_strategy(
        resolved_hooks: &ResolvedHooks,
        strategy: ExecutionStrategy,
    ) -> Result<ExecutionResults> {
        match strategy {
            ExecutionStrategy::Sequential => Self::execute_sequential(resolved_hooks),
            ExecutionStrategy::Parallel => Self::execute_parallel_safe(resolved_hooks),
            ExecutionStrategy::ForceParallel => Ok(Self::execute_parallel_unsafe(resolved_hooks)),
        }
    }

    /// Execute hooks sequentially (original behavior)
    fn execute_sequential(resolved_hooks: &ResolvedHooks) -> Result<ExecutionResults> {
        let mut results = HashMap::new();
        let mut overall_success = true;

        for (name, hook) in &resolved_hooks.hooks {
            let result = Self::execute_single_hook(
                name,
                hook,
                &resolved_hooks.worktree_context,
                resolved_hooks.changed_files.as_deref(),
            )
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

    /// Execute hooks in parallel, respecting repository modification safety
    fn execute_parallel_safe(resolved_hooks: &ResolvedHooks) -> Result<ExecutionResults> {
        // Separate hooks into safe-to-parallelize and repository-modifying
        let mut safe_hooks = Vec::new();
        let mut modifying_hooks = Vec::new();

        for (name, hook) in &resolved_hooks.hooks {
            if hook.definition.modifies_repository {
                modifying_hooks.push((name.clone(), hook));
            } else {
                safe_hooks.push((name.clone(), hook));
            }
        }

        let results = Arc::new(Mutex::new(HashMap::new()));
        let overall_success = Arc::new(Mutex::new(true));

        // First, run all safe hooks in parallel
        if !safe_hooks.is_empty() {
            let mut handles = Vec::new();

            for (name, hook) in safe_hooks {
                let name = name.clone();
                let hook = hook.clone();
                let results = Arc::clone(&results);
                let overall_success = Arc::clone(&overall_success);

                let worktree_context = resolved_hooks.worktree_context.clone();
                let changed_files = resolved_hooks.changed_files.clone();
                let handle = thread::spawn(move || {
                    match Self::execute_single_hook(
                        &name,
                        &hook,
                        &worktree_context,
                        changed_files.as_deref(),
                    ) {
                        Ok(result) => {
                            let success = result.success;
                            results.lock().unwrap().insert(name, result);
                            if !success {
                                *overall_success.lock().unwrap() = false;
                            }
                        }
                        Err(e) => {
                            // Create a failed result for execution errors
                            let result = ExecutionResult {
                                exit_code: -1,
                                stdout: String::new(),
                                stderr: format!("Execution error: {e:#}"),
                                success: false,
                            };
                            results.lock().unwrap().insert(name, result);
                            *overall_success.lock().unwrap() = false;
                        }
                    }
                });
                handles.push(handle);
            }

            // Wait for all parallel hooks to complete
            for handle in handles {
                if handle.join().is_err() {
                    *overall_success.lock().unwrap() = false;
                }
            }
        }

        // Then, run repository-modifying hooks sequentially
        for (name, hook) in modifying_hooks {
            let result = Self::execute_single_hook(
                &name,
                hook,
                &resolved_hooks.worktree_context,
                resolved_hooks.changed_files.as_deref(),
            )
            .with_context(|| format!("Failed to execute hook: {name}"))?;

            if !result.success {
                *overall_success.lock().unwrap() = false;
            }

            results.lock().unwrap().insert(name.clone(), result);
        }

        let results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
        let overall_success = Arc::try_unwrap(overall_success)
            .unwrap()
            .into_inner()
            .unwrap();

        Ok(ExecutionResults {
            results,
            success: overall_success,
        })
    }

    /// Execute all hooks in parallel (unsafe - ignores repository modification)
    fn execute_parallel_unsafe(resolved_hooks: &ResolvedHooks) -> ExecutionResults {
        let results = Arc::new(Mutex::new(HashMap::new()));
        let overall_success = Arc::new(Mutex::new(true));
        let mut handles = Vec::new();

        for (name, hook) in &resolved_hooks.hooks {
            let name = name.clone();
            let hook = hook.clone();
            let results = Arc::clone(&results);
            let overall_success = Arc::clone(&overall_success);

            let worktree_context = resolved_hooks.worktree_context.clone();
            let changed_files = resolved_hooks.changed_files.clone();
            let handle = thread::spawn(move || match Self::execute_single_hook(&name, &hook, &worktree_context, changed_files.as_deref()) {
                Ok(result) => {
                    let success = result.success;
                    results.lock().unwrap().insert(name, result);
                    if !success {
                        *overall_success.lock().unwrap() = false;
                    }
                }
                Err(e) => {
                    let result = ExecutionResult {
                        exit_code: -1,
                        stdout: String::new(),
                        stderr: format!("Execution error: {e:#}"),
                        success: false,
                    };
                    results.lock().unwrap().insert(name, result);
                    *overall_success.lock().unwrap() = false;
                }
            });
            handles.push(handle);
        }

        // Wait for all hooks to complete
        for handle in handles {
            if handle.join().is_err() {
                *overall_success.lock().unwrap() = false;
            }
        }

        let results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
        let overall_success = Arc::try_unwrap(overall_success)
            .unwrap()
            .into_inner()
            .unwrap();

        ExecutionResults {
            results,
            success: overall_success,
        }
    }

    /// Execute hooks respecting dependencies
    fn execute_with_dependencies(resolved_hooks: &ResolvedHooks) -> Result<ExecutionResults> {
        let mut resolver = DependencyResolver::new();
        let hook_names: Vec<String> = resolved_hooks.hooks.keys().cloned().collect();

        // Build dependency graph
        for (name, hook) in &resolved_hooks.hooks {
            let dependencies = hook.definition.depends_on.clone().unwrap_or_default();
            resolver.add_hook(name.clone(), dependencies);
        }

        // Resolve execution plan
        let plan = resolver.resolve(&hook_names)
            .context("Failed to resolve hook dependencies")?;

        let mut all_results = HashMap::new();
        let mut overall_success = true;

        // Execute hooks phase by phase
        for phase in &plan.phases {
            let mut phase_results = HashMap::new();

            if phase.parallel && phase.hooks.len() > 1 {
                // Execute phase hooks in parallel
                let results = Arc::new(Mutex::new(HashMap::new()));
                let phase_success = Arc::new(Mutex::new(true));
                let mut handles = Vec::new();

                for hook_name in &phase.hooks {
                    let hook = &resolved_hooks.hooks[hook_name];
                    let name = hook_name.clone();
                    let hook = hook.clone();
                    let results = Arc::clone(&results);
                    let phase_success = Arc::clone(&phase_success);

                    let worktree_context = resolved_hooks.worktree_context.clone();
                    let changed_files = resolved_hooks.changed_files.clone();
                    let handle = thread::spawn(move || {
                        match Self::execute_single_hook(&name, &hook, &worktree_context, changed_files.as_deref()) {
                            Ok(result) => {
                                let success = result.success;
                                results.lock().unwrap().insert(name, result);
                                if !success {
                                    *phase_success.lock().unwrap() = false;
                                }
                            }
                            Err(e) => {
                                let result = ExecutionResult {
                                    exit_code: -1,
                                    stdout: String::new(),
                                    stderr: format!("Execution error: {e:#}"),
                                    success: false,
                                };
                                results.lock().unwrap().insert(name, result);
                                *phase_success.lock().unwrap() = false;
                            }
                        }
                    });
                    handles.push(handle);
                }

                // Wait for all hooks in this phase
                for handle in handles {
                    if handle.join().is_err() {
                        *phase_success.lock().unwrap() = false;
                    }
                }

                phase_results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
                let phase_success = Arc::try_unwrap(phase_success).unwrap().into_inner().unwrap();
                
                if !phase_success {
                    overall_success = false;
                    // Stop execution if any hook in this phase failed
                    all_results.extend(phase_results);
                    break;
                }
            } else {
                // Execute phase hooks sequentially
                for hook_name in &phase.hooks {
                    let hook = &resolved_hooks.hooks[hook_name];
                    let result = Self::execute_single_hook(
                        hook_name,
                        hook,
                        &resolved_hooks.worktree_context,
                        resolved_hooks.changed_files.as_deref(),
                    )
                    .with_context(|| format!("Failed to execute hook: {hook_name}"))?;
                    
                    let success = result.success;
                    phase_results.insert(hook_name.clone(), result);
                    
                    if !success {
                        // Stop execution if hook failed
                        all_results.extend(phase_results);
                        return Ok(ExecutionResults {
                            results: all_results,
                            success: false,
                        });
                    }
                }
            }

            all_results.extend(phase_results);
        }

        Ok(ExecutionResults {
            results: all_results,
            success: overall_success,
        })
    }

    /// Execute a single hook
    fn execute_single_hook(
        name: &str,
        hook: &ResolvedHook,
        worktree_context: &crate::hooks::resolver::WorktreeContext,
        changed_files: Option<&[PathBuf]>,
    ) -> Result<ExecutionResult> {
        // Create template resolver with worktree context
        let config_dir = hook.source_file.parent()
            .context("Hook source file has no parent directory")?;
        let template_resolver = TemplateResolver::with_worktree_context(config_dir, &hook.working_directory, worktree_context);

        // Resolve command templates
        let mut command = match &hook.definition.command {
            HookCommand::Shell(cmd) => {
                let resolved_cmd = template_resolver.resolve_string(cmd)
                    .context("Failed to resolve command template")?;
                let mut command = Command::new("sh");
                command.args(["-c", &resolved_cmd]);
                command
            }
            HookCommand::Args(args) => {
                if args.is_empty() {
                    return Err(anyhow::anyhow!("Empty command for hook: {}", name));
                }
                let resolved_args = template_resolver.resolve_command_args(args)
                    .context("Failed to resolve command arguments")?;
                let mut command = Command::new(&resolved_args[0]);
                if resolved_args.len() > 1 {
                    command.args(&resolved_args[1..]);
                }
                command
            }
        };

        // Set working directory (resolve template if needed)
        let working_dir = if let Some(workdir_template) = &hook.definition.workdir {
            let resolved_workdir = template_resolver.resolve_string(workdir_template)
                .context("Failed to resolve workdir template")?;
            PathBuf::from(resolved_workdir)
        } else {
            hook.working_directory.clone()
        };
        command.current_dir(&working_dir);

        // Set environment variables with template resolution
        if let Some(env) = &hook.definition.env {
            let resolved_env = template_resolver.resolve_env(env)
                .context("Failed to resolve environment variable templates")?;
            for (key, value) in resolved_env {
                command.env(key, value);
            }
        }

        // Inject changed files env vars (repo-relative paths)
        let relevant_changed: Vec<PathBuf> = if let Some(cf) = changed_files {
            if let Some(patterns) = &hook.definition.files {
                match FilePatternMatcher::new(patterns) {
                    Ok(matcher) => cf.iter().filter(|p| matcher.matches(p)).cloned().collect(),
                    Err(_) => cf.to_vec(),
                }
            } else {
                cf.to_vec()
            }
        } else {
            Vec::new()
        };
        let changed_space = relevant_changed
            .iter()
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");
        let changed_list = relevant_changed
            .iter()
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n");
        command.env("CHANGED_FILES", changed_space);
        command.env("CHANGED_FILES_LIST", changed_list.clone());

        // Optional: write changed files to a temp file and expose path
        let mut changed_files_file: Option<PathBuf> = None;
        if changed_files.is_some() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let tmp_path = std::env::temp_dir().join(format!(
                "peter-hook-changed-{}-{}.lst",
                std::process::id(),
                now
            ));
            if std::fs::write(&tmp_path, &changed_list).is_ok() {
                command.env("CHANGED_FILES_FILE", &tmp_path);
                changed_files_file = Some(tmp_path);
            } else {
                command.env("CHANGED_FILES_FILE", "");
            }
        } else {
            command.env("CHANGED_FILES_FILE", "");
        }

        // Configure stdio
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Execute the command
        let output = command
            .output()
            .with_context(|| format!("Failed to execute hook command: {name}"))?;

        // Cleanup temp file, if any
        if let Some(p) = changed_files_file {
            let _ = std::fs::remove_file(p);
        }

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
        let fmt = formatter();
        
        println!("{}", fmt.section_header("Hook Execution Summary"));

        for (name, result) in &self.results {
            println!("{}", fmt.hook_result(name, result.success, result.exit_code));

            if !result.stdout.is_empty() {
                println!("  stdout: {}", result.stdout.trim());
            }

            if !result.stderr.is_empty() {
                println!("  stderr: {}", result.stderr.trim());
            }
        }

        println!("{}", fmt.overall_result(self.success));
    }

    /// Print execution with progress bar (TTY only)
    pub fn print_with_progress(&self, hook_names: &[String]) {
        let fmt = formatter();
        
        if let Some(pb) = fmt.create_progress_bar(hook_names.len() as u64) {
            pb.set_message("Starting hooks...");
            
            for (i, name) in hook_names.iter().enumerate() {
                pb.set_message(format!("Running {name}"));
                pb.set_position(i as u64);
                
                // Simulate some work for demo
                std::thread::sleep(Duration::from_millis(100));
                
                if let Some(result) = self.results.get(name) {
                    let status = if result.success { "✅" } else { "❌" };
                    pb.println(format!("{status} {name}"));
                }
            }
            
            pb.finish_with_message("Hook execution completed!");
        } else {
            // Fallback to regular summary for non-TTY
            self.print_summary();
        }
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
    use crate::config::{HookCommand, HookDefinition};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_test_hook(command: HookCommand, workdir: Option<String>) -> ResolvedHook {
        ResolvedHook {
            definition: HookDefinition {
                command,
                workdir,
                env: None,
                description: None,
                modifies_repository: false,
                files: None,
                run_always: false,
                depends_on: None,
            },
            working_directory: std::env::temp_dir(),
            source_file: PathBuf::from("test.toml"),
        }
    }

    fn create_test_worktree_context() -> crate::hooks::resolver::WorktreeContext {
        crate::hooks::resolver::WorktreeContext {
            is_worktree: false,
            worktree_name: None,
            repo_root: std::env::temp_dir(),
            common_dir: std::env::temp_dir().join(".git"),
            working_dir: std::env::temp_dir(),
        }
    }

    #[test]
    fn test_execute_shell_command_success() {
        let hook = create_test_hook(HookCommand::Shell("echo 'hello world'".to_string()), None);

        let worktree_context = create_test_worktree_context();
        let result = HookExecutor::execute_single_hook("test", &hook, &worktree_context, None).unwrap();

        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout.trim(), "hello world");
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn test_execute_shell_command_failure() {
        let hook = create_test_hook(HookCommand::Shell("exit 1".to_string()), None);

        let worktree_context = create_test_worktree_context();
        let result = HookExecutor::execute_single_hook("test", &hook, &worktree_context, None).unwrap();

        assert!(!result.success);
        assert_eq!(result.exit_code, 1);
    }

    #[test]
    fn test_execute_args_command() {
        let hook = create_test_hook(
            HookCommand::Args(vec![
                "echo".to_string(),
                "hello".to_string(),
                "args".to_string(),
            ]),
            None,
        );

        let worktree_context = create_test_worktree_context();
        let result = HookExecutor::execute_single_hook("test", &hook, &worktree_context, None).unwrap();

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
            execution_strategy: ExecutionStrategy::Sequential,
            changed_files: None,
            worktree_context: create_test_worktree_context(),
        };

        let results = HookExecutor::execute(&resolved_hooks).unwrap();

        assert!(!results.success); // Overall failure due to one failed hook
        assert_eq!(results.results.len(), 2);

        assert!(results.results["success"].success);
        assert!(!results.results["failure"].success);

        let failed = results.get_failed_hooks();
        assert_eq!(failed, vec!["failure"]);
    }

    #[test]
    fn test_parallel_safe_execution() {
        let mut hooks = HashMap::new();

        // Safe hooks that can run in parallel
        hooks.insert(
            "lint".to_string(),
            create_test_hook_with_modification(HookCommand::Shell("echo lint".to_string()), false),
        );

        hooks.insert(
            "test".to_string(),
            create_test_hook_with_modification(HookCommand::Shell("echo test".to_string()), false),
        );

        // Repository-modifying hook that must run sequentially
        hooks.insert(
            "format".to_string(),
            create_test_hook_with_modification(HookCommand::Shell("echo format".to_string()), true),
        );

        let resolved_hooks = ResolvedHooks {
            config_path: PathBuf::from("test.toml"),
            hooks,
            execution_strategy: ExecutionStrategy::Parallel,
            changed_files: None,
            worktree_context: create_test_worktree_context(),
        };

        let results = HookExecutor::execute(&resolved_hooks).unwrap();

        assert!(results.success);
        assert_eq!(results.results.len(), 3);
        assert!(results.results["lint"].success);
        assert!(results.results["test"].success);
        assert!(results.results["format"].success);
    }

    #[test]
    fn test_sequential_execution() {
        let mut hooks = HashMap::new();

        hooks.insert(
            "hook1".to_string(),
            create_test_hook(HookCommand::Shell("echo hook1".to_string()), None),
        );

        hooks.insert(
            "hook2".to_string(),
            create_test_hook(HookCommand::Shell("echo hook2".to_string()), None),
        );

        let resolved_hooks = ResolvedHooks {
            config_path: PathBuf::from("test.toml"),
            hooks,
            execution_strategy: ExecutionStrategy::Sequential,
            changed_files: None,
            worktree_context: create_test_worktree_context(),
        };

        let results = HookExecutor::execute(&resolved_hooks).unwrap();

        assert!(results.success);
        assert_eq!(results.results.len(), 2);
    }

    #[test]
    fn test_force_parallel_execution() {
        let mut hooks = HashMap::new();

        // Even repository-modifying hooks run in parallel (unsafe mode)
        hooks.insert(
            "format1".to_string(),
            create_test_hook_with_modification(
                HookCommand::Shell("echo format1".to_string()),
                true,
            ),
        );

        hooks.insert(
            "format2".to_string(),
            create_test_hook_with_modification(
                HookCommand::Shell("echo format2".to_string()),
                true,
            ),
        );

        let resolved_hooks = ResolvedHooks {
            config_path: PathBuf::from("test.toml"),
            hooks,
            execution_strategy: ExecutionStrategy::ForceParallel,
            changed_files: None,
            worktree_context: create_test_worktree_context(),
        };

        let results = HookExecutor::execute(&resolved_hooks).unwrap();

        assert!(results.success);
        assert_eq!(results.results.len(), 2);
    }

    fn create_test_hook_with_modification(
         command: HookCommand,
         modifies_repository: bool,
     ) -> ResolvedHook {
         ResolvedHook {
             definition: HookDefinition {
                 command,
                 workdir: None,
                 env: None,
                 description: None,
                 modifies_repository,
                 files: None,
                 run_always: false,
                 depends_on: None,
             },
             working_directory: std::env::temp_dir(),
             source_file: PathBuf::from("test.toml"),
         }
     }

    #[test]
    fn test_env_vars_filtered_changed_files() {
        // Hook with file filter should receive only matching changes
        let hook = ResolvedHook {
            definition: HookDefinition {
                command: HookCommand::Shell("printf '%s\n' \"$CHANGED_FILES\" && printf '%s\n' \"$CHANGED_FILES_LIST\" && cat \"$CHANGED_FILES_FILE\"".to_string()),
                workdir: None,
                env: None,
                description: None,
                modifies_repository: false,
                files: Some(vec!["**/*.rs".to_string()]),
                run_always: false,
                depends_on: None,
            },
            working_directory: std::env::temp_dir(),
            source_file: PathBuf::from("test.toml"),
        };
        let worktree_context = create_test_worktree_context();
        let changes = vec![PathBuf::from("src/a.rs"), PathBuf::from("README.md")];
        let result = HookExecutor::execute_single_hook("filtered", &hook, &worktree_context, Some(&changes)).unwrap();
        assert!(result.success);
        let out = result.stdout;
        assert!(out.contains("src/a.rs"));
        assert!(!out.contains("README.md"));
    }

    #[test]
    fn test_env_vars_all_changed_files_no_filter() {
        let hook = ResolvedHook {
            definition: HookDefinition {
                command: HookCommand::Shell("printf '%s\n' \"$CHANGED_FILES\"".to_string()),
                workdir: None,
                env: None,
                description: None,
                modifies_repository: false,
                files: None,
                run_always: false,
                depends_on: None,
            },
            working_directory: std::env::temp_dir(),
            source_file: PathBuf::from("test.toml"),
        };
        let worktree_context = create_test_worktree_context();
        let changes = vec![PathBuf::from("a"), PathBuf::from("b/c")];
        let result = HookExecutor::execute_single_hook("nofilter", &hook, &worktree_context, Some(&changes)).unwrap();
        assert!(result.success);
        let out = result.stdout;
        assert!(out.contains("a"));
        assert!(out.contains("b/c"));
    }

    #[test]
    fn test_env_vars_empty_when_no_changes() {
        let hook = ResolvedHook {
            definition: HookDefinition {
                command: HookCommand::Shell("printf '[%s]-[%s]-[%s]\n' \"$CHANGED_FILES\" \"$CHANGED_FILES_LIST\" \"$CHANGED_FILES_FILE\"".to_string()),
                workdir: None,
                env: None,
                description: None,
                modifies_repository: false,
                files: None,
                run_always: false,
                depends_on: None,
            },
            working_directory: std::env::temp_dir(),
            source_file: PathBuf::from("test.toml"),
        };
        let worktree_context = create_test_worktree_context();
        let result = HookExecutor::execute_single_hook("empty", &hook, &worktree_context, None).unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("[]-[]-[]"));
    }
 }

