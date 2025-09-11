//! Peter Hook - A hierarchical git hooks manager for monorepos

use anyhow::{Context, Result};
use clap::Parser;
use peter_hook::{
    cli::{Cli, Commands},
    git::{ChangeDetectionMode, GitHookInstaller, GitRepository, WorktreeHookStrategy},
    hooks::{HookExecutor, HookResolver},
};
use std::env;
use std::io::{self, Write};
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e:#}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { force, worktree_strategy } => install_hooks(force, &worktree_strategy),
        Commands::Uninstall { yes } => uninstall_hooks(yes),
        Commands::Run { event, files, git_args } => run_hooks(&event, files, &git_args),
        Commands::Validate { trace_imports, json } => validate_config(trace_imports, json),
        Commands::List => list_hooks(),
        Commands::RunHook { event } => run_hook_simulation(&event),
        Commands::ListWorktrees => list_worktrees(),
    }
}

/// Install git hooks for the current repository
fn install_hooks(force: bool, worktree_strategy: &str) -> Result<()> {
    println!("Installing git hooks...");

    // Parse the worktree strategy
    let strategy: WorktreeHookStrategy = worktree_strategy
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid worktree strategy: {}", worktree_strategy))?;

    let installer = GitHookInstaller::with_strategy(strategy)
        .context("Failed to initialize git hook installer")?;

    if !force {
        // Check if any hooks would be overwritten
        let repo = GitRepository::find_from_current_dir()?;
        let existing_hooks = repo.list_hooks()?;
        if !existing_hooks.is_empty() && existing_hooks
            .iter()
            .find_map(|hook| {
                repo.get_hook_info(hook).ok().flatten().and_then(|info| {
                    if info.is_managed {
                        Some(hook.clone())
                    } else {
                        None
                    }
                })
            }).is_none() {
            println!("⚠️  Found existing git hooks that are not managed by peter-hook:");
            for hook in &existing_hooks {
                println!("  - {hook}");
            }
            println!("\nUse --force to backup existing hooks and install peter-hook hooks.");
            println!("Or use 'peter-hook uninstall' to remove existing hooks first.");
            return Ok(());
        }
    }

    let report = installer
        .install_all()
        .context("Failed to install git hooks")?;

    report.print_summary();

    if !report.is_success() {
        process::exit(1);
    }

    Ok(())
}

/// Uninstall peter-hook managed hooks
fn uninstall_hooks(yes: bool) -> Result<()> {
    if !yes {
        println!("This will remove all peter-hook managed hooks from your repository.");
        println!("Backed up hooks will be restored if they exist.");
        print!("Are you sure you want to continue? [y/N]: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("Failed to read input")?;

        let input = input.trim().to_lowercase();
        if input != "y" && input != "yes" {
            println!("Uninstall cancelled.");
            return Ok(());
        }
    }

    let installer = GitHookInstaller::new().context("Failed to initialize git hook installer")?;

    let report = installer.uninstall_all();

    report.print_summary();

    if !report.is_success() {
        process::exit(1);
    }

    Ok(())
}

/// List all git hooks in the repository
fn list_hooks() -> Result<()> {
    let repo = GitRepository::find_from_current_dir().context("Failed to find git repository")?;

    let hooks = repo.list_hooks().context("Failed to list git hooks")?;

    if hooks.is_empty() {
        println!("No git hooks found in this repository.");
        return Ok(());
    }

    println!("Git hooks in this repository:");
    println!("============================");

    for hook_name in hooks {
        if let Some(info) = repo.get_hook_info(&hook_name)? {
            let status = if info.is_managed {
                "🔧 managed"
            } else {
                "📄 custom"
            };

            let executable = if info.is_executable { "✅" } else { "❌" };

            println!(
                "{} {} {} (executable: {})",
                executable,
                hook_name,
                status,
                if info.is_executable { "yes" } else { "no" }
            );
        }
    }

    Ok(())
}

/// Run hooks for a specific git event
fn run_hooks(event: &str, enable_file_filtering: bool, _git_args: &[String]) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current working directory")?;

    let resolver = HookResolver::new(&current_dir);

    // Determine change detection mode based on event and file filtering
    let change_mode = if enable_file_filtering {
        Some(match event {
            "pre-push" => ChangeDetectionMode::Push {
                remote: "origin".to_string(),
                remote_branch: "main".to_string(), // TODO: detect actual default branch
            },
            _ => ChangeDetectionMode::WorkingDirectory,
        })
    } else {
        None
    };

    match resolver.resolve_hooks_with_files(event, change_mode)? {
        Some(resolved_hooks) => {
            println!(
                "Found hooks configuration: {}",
                resolved_hooks.config_path.display()
            );
            
            if let Some(ref changed_files) = resolved_hooks.changed_files {
                println!("Detected {} changed files", changed_files.len());
                if changed_files.is_empty() {
                    println!("No files changed - some hooks may be skipped");
                }
            }
            
            println!(
                "Running {} hooks for event: {}",
                resolved_hooks.hooks.len(),
                event
            );

            let results =
                HookExecutor::execute(&resolved_hooks).context("Failed to execute hooks")?;

            results.print_summary();

            if !results.success {
                process::exit(1);
            }
        }
        None => {
            println!("No hooks found for event: {event}");
        }
    }

    Ok(())
}

/// Validate hook configuration
fn validate_config(trace_imports: bool, json: bool) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current working directory")?;

    let resolver = HookResolver::new(&current_dir);

    match resolver.find_config_file()? {
        Some(config_path) => {
            println!("Validating config file: {}", config_path.display());

            // Try to parse the configuration
            if trace_imports {
                match peter_hook::HookConfig::from_file_with_trace(&config_path) {
                    Ok((config, diag)) => {
                        println!("✓ Configuration is valid");

                        let hook_names = config.get_hook_names();
                        if hook_names.is_empty() {
                            println!("  No hooks or groups defined");
                        } else {
                            println!("  Found {} hooks/groups:", hook_names.len());
                            for name in hook_names {
                                println!("    - {name}");
                            }
                        }

                        if json {
                            // Print diagnostics as JSON
                            match serde_json::to_string_pretty(&diag) {
                                Ok(s) => println!("{s}"),
                                Err(e) => eprintln!("Failed to serialize diagnostics: {e:#}"),
                            }
                        } else {
                            // Human-readable diagnostics
                            if diag.imports.is_empty() {
                                println!("(no imports)");
                            } else {
                                println!("Imports (order):");
                                for r in &diag.imports {
                                    println!("  {} -> {}", r.from, r.resolved);
                                }
                            }
                            if !diag.overrides.is_empty() {
                                println!("Overrides:");
                                for o in &diag.overrides {
                                    println!("  {} {}: {} -> {}", o.kind, o.name, o.previous, o.new);
                                }
                            }
                            if !diag.cycles.is_empty() {
                                println!("Cycles (skipped):");
                                for c in &diag.cycles {
                                    println!("  {c}");
                                }
                            }
                            if !diag.unused.is_empty() {
                                println!("Unused imports (no contributions):");
                                for u in &diag.unused {
                                    println!("  {u}");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("✗ Configuration is invalid: {e:#}");
                        process::exit(1);
                    }
                }
            } else {
                match peter_hook::HookConfig::from_file(&config_path) {
                    Ok(config) => {
                        println!("✓ Configuration is valid");

                        let hook_names = config.get_hook_names();
                        if hook_names.is_empty() {
                            println!("  No hooks or groups defined");
                        } else {
                            println!("  Found {} hooks/groups:", hook_names.len());
                            for name in hook_names {
                                println!("    - {name}");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("✗ Configuration is invalid: {e:#}");
                        process::exit(1);
                    }
                }
            }
        }
        None => {
            println!("No hooks.toml file found in current directory or parent directories");
        }
    }

    Ok(())
}

/// Run git hooks without performing the git operation
fn run_hook_simulation(event: &str) -> Result<()> {
    // Use the exact same logic as git hooks, with file filtering enabled
    run_hooks(event, true, &[])
}

/// List all worktrees and their hook configuration
fn list_worktrees() -> Result<()> {
    let repo = GitRepository::find_from_current_dir()
        .context("Failed to find git repository")?;

    let worktrees = repo.list_worktrees()
        .context("Failed to list worktrees")?;

    if worktrees.is_empty() {
        println!("No worktrees found in this repository.");
        return Ok(());
    }

    println!("Git worktrees in this repository:");
    println!("=================================");

    for worktree in worktrees {
        let current_indicator = if worktree.is_current { " (current)" } else { "" };
        let main_indicator = if worktree.is_main { " [main]" } else { "" };
        
        println!("📁 {}{}{}", worktree.name, main_indicator, current_indicator);
        println!("   Path: {}", worktree.path.display());
        
        // Check for hooks in this worktree
        let hooks_dir = if worktree.is_main {
            repo.get_common_hooks_dir().to_path_buf()
        } else {
            // For non-main worktrees, check both shared and worktree-specific locations
            let common_hooks = repo.get_common_hooks_dir();
            let worktree_hooks = worktree.path.join(".git/hooks");
            
            if worktree_hooks.exists() {
                worktree_hooks
            } else {
                common_hooks.to_path_buf()
            }
        };

        if hooks_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&hooks_dir) {
                let mut hook_files: Vec<_> = entries
                    .filter_map(Result::ok)
                    .filter_map(|entry| {
                        let path = entry.path();
                        if path.is_file() {
                            path.file_name()
                                .and_then(|name| name.to_str())
                                .filter(|name| !name.ends_with(".sample") && !name.starts_with('.'))
                                .map(ToString::to_string)
                        } else {
                            None
                        }
                    })
                    .collect();
                
                hook_files.sort();
                
                if hook_files.is_empty() {
                    println!("   Hooks: none");
                } else {
                    let hooks_type = if worktree.is_main || hooks_dir == repo.get_common_hooks_dir() {
                        "shared"
                    } else {
                        "worktree-specific"
                    };
                    println!("   Hooks ({}): {}", hooks_type, hook_files.join(", "));
                }
            } else {
                println!("   Hooks: unable to read hooks directory");
            }
        } else {
            println!("   Hooks: none");
        }
        
        println!();
    }

    Ok(())
}
