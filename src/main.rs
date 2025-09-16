//! Peter Hook - A hierarchical git hooks manager for monorepos

use anyhow::{Context, Result};
use clap::Parser;
use peter_hook::{
    cli::{Cli, Commands},
    debug,
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

    // Enable debug mode if requested
    if cli.debug {
        debug::enable();
    }

    match cli.command {
        Commands::Install { force, worktree_strategy } => install_hooks(force, &worktree_strategy),
        Commands::Uninstall { yes } => uninstall_hooks(yes),
        Commands::Run { event, git_args } => run_hooks(&event, &git_args),
        Commands::Validate { trace_imports, json } => validate_config(trace_imports, json),
        Commands::List => list_hooks(),
        Commands::RunHook { event } => run_hook_simulation(&event),
        Commands::RunByName { hook_name, files } => run_specific_hook(&hook_name, files),
        Commands::ListWorktrees => list_worktrees(),
        Commands::Version => show_version(),
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

/// Show version information
fn show_version() -> Result<()> {
    println!("{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

/// Run hooks for a specific git event
fn run_hooks(event: &str, _git_args: &[String]) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current working directory")?;

    let resolver = HookResolver::new(&current_dir);

    // File filtering is now always enabled - determine change detection mode based on event
    let change_mode = Some(match event {
        "pre-push" => ChangeDetectionMode::Push {
            remote: "origin".to_string(),
            remote_branch: "main".to_string(), // TODO: detect actual default branch
        },
        _ => ChangeDetectionMode::WorkingDirectory,
    });

    match resolver.resolve_hooks_with_files(event, change_mode)? {
        Some(resolved_hooks) => {
            if debug::is_enabled() && atty::is(atty::Stream::Stdout) {
                println!("\x1b[38;5;201m🎪 \x1b[1m\x1b[38;5;51mPETER-HOOK EXECUTION EXTRAVAGANZA!\x1b[0m");
                println!("\x1b[38;5;198m📋 Config: \x1b[38;5;87m{}\x1b[0m", resolved_hooks.config_path.display());

                if let Some(ref changed_files) = resolved_hooks.changed_files {
                    println!("\x1b[38;5;214m🎯 \x1b[1m\x1b[38;5;208mFile targeting activated!\x1b[0m \x1b[38;5;118m{} files detected\x1b[0m", changed_files.len());
                    if changed_files.is_empty() {
                        println!("\x1b[38;5;226m⚡ \x1b[1mNo files changed - hooks may skip for maximum speed!\x1b[0m");
                    } else {
                        // Show first few files with rotating emojis
                        let file_emojis = ["📄", "📝", "🔧", "⚙️", "🎨", "🚀"];
                        for (i, file) in changed_files.iter().take(6).enumerate() {
                            let emoji = file_emojis[i % file_emojis.len()];
                            println!("\x1b[38;5;147m    {} \x1b[38;5;183m{}\x1b[0m", emoji, file.display());
                        }
                        if changed_files.len() > 6 {
                            println!("\x1b[38;5;147m    🌟 \x1b[38;5;105m... and {} more files!\x1b[0m", changed_files.len() - 6);
                        }
                    }
                }

                println!("\x1b[38;5;46m🚀 \x1b[1m\x1b[38;5;82mLaunching {} hooks for event:\x1b[0m \x1b[38;5;226m{}\x1b[0m", resolved_hooks.hooks.len(), event);

                // Show hook configuration summary with crazy colors and emojis
                println!("\x1b[38;5;198m🎭 \x1b[1m\x1b[38;5;207mHOOK CONFIGURATION EXTRAVAGANZA!\x1b[0m");

                // Group hooks by file patterns for visual organization
                let mut pattern_groups = std::collections::HashMap::new();
                for (hook_name, hook) in &resolved_hooks.hooks {
                    let patterns = hook.definition.files.as_ref()
                        .map(|f| f.join(", "))
                        .unwrap_or_else(|| if hook.definition.run_always { "🌍 ALL FILES (run_always)".to_string() } else { "🎯 NO PATTERNS".to_string() });
                    pattern_groups.entry(patterns).or_insert_with(Vec::new).push(hook_name);
                }

                let colors = [196, 208, 226, 118, 51, 99, 201, 165, 129, 93];
                for (i, (pattern, hooks)) in pattern_groups.iter().enumerate() {
                    let color = colors[i % colors.len()];
                    let emoji = match i % 8 {
                        0 => "🐍", 1 => "⚡", 2 => "🔧", 3 => "🎨",
                        4 => "🛡️", 5 => "📊", 6 => "🌐", _ => "✨"
                    };
                    println!("\x1b[38;5;{}m{} Pattern: \x1b[38;5;159m{}\x1b[0m", color, emoji, pattern);
                    for hook in hooks {
                        println!("\x1b[38;5;147m      🎪 \x1b[38;5;183m{}\x1b[0m", hook);
                    }
                }

                println!("\x1b[38;5;198m{}\x1b[0m", "═".repeat(60));
            } else {
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
            }

            let results =
                HookExecutor::execute(&resolved_hooks).context("Failed to execute hooks")?;

            if debug::is_enabled() && atty::is(atty::Stream::Stdout) {
                println!("\x1b[38;5;198m{}\x1b[0m", "═".repeat(60));
                if results.success {
                    println!("\x1b[38;5;46m🎊 \x1b[1m\x1b[38;5;82mALL HOOKS SUCCEEDED!\x1b[0m \x1b[38;5;46m🎊\x1b[0m");
                    println!("\x1b[38;5;118m✨ Your code is \x1b[1m\x1b[38;5;159mPERFECT\x1b[0m\x1b[38;5;118m! Ready to commit! ✨\x1b[0m");
                } else {
                    println!("\x1b[38;5;196m💥 \x1b[1m\x1b[38;5;199mSOME HOOKS FAILED!\x1b[0m \x1b[38;5;196m💥\x1b[0m");
                    let failed = results.get_failed_hooks();
                    println!("\x1b[38;5;197m🚨 Failed hooks: \x1b[38;5;167m{}\x1b[0m", failed.join(", "));
                }
                println!("\x1b[38;5;198m{}\x1b[0m", "═".repeat(60));
            }

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
    // Use the exact same logic as git hooks (file filtering is now always enabled)
    run_hooks(event, &[])
}

/// Run a specific hook by name
fn run_specific_hook(hook_name: &str, enable_file_filtering: bool) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current working directory")?;

    let resolver = HookResolver::new(&current_dir);

    match resolver.resolve_hook_by_name(hook_name, enable_file_filtering)? {
        Some(resolved_hooks) => {
            if debug::is_enabled() && atty::is(atty::Stream::Stdout) {
                println!("\x1b[38;5;201m🎪 \x1b[1m\x1b[38;5;51mPETER-HOOK INDIVIDUAL RUN!\x1b[0m");
                println!("\x1b[38;5;198m📋 Config: \x1b[38;5;87m{}\x1b[0m", resolved_hooks.config_path.display());
                println!("\x1b[38;5;46m🎯 \x1b[1m\x1b[38;5;82mRunning hook:\x1b[0m \x1b[38;5;226m{}\x1b[0m", hook_name);

                if let Some(ref changed_files) = resolved_hooks.changed_files {
                    println!("\x1b[38;5;214m📁 \x1b[1m\x1b[38;5;208mFile filtering enabled!\x1b[0m \x1b[38;5;118m{} files detected\x1b[0m", changed_files.len());
                    if changed_files.is_empty() {
                        println!("\x1b[38;5;226m⚡ \x1b[1mNo files changed - hook may skip for maximum speed!\x1b[0m");
                    } else {
                        // Show first few files with rotating emojis
                        let file_emojis = ["📄", "📝", "🔧", "⚙️", "🎨", "🚀"];
                        for (i, file) in changed_files.iter().take(6).enumerate() {
                            let emoji = file_emojis[i % file_emojis.len()];
                            println!("\x1b[38;5;147m    {} \x1b[38;5;183m{}\x1b[0m", emoji, file.display());
                        }
                        if changed_files.len() > 6 {
                            println!("\x1b[38;5;147m    🌟 \x1b[38;5;105m... and {} more files!\x1b[0m", changed_files.len() - 6);
                        }
                    }
                } else {
                    println!("\x1b[38;5;118m📂 \x1b[1mFile filtering disabled - running on all files\x1b[0m");
                }

                println!("\x1b[38;5;198m{}\x1b[0m", "═".repeat(60));
            } else {
                println!(
                    "Found hooks configuration: {}",
                    resolved_hooks.config_path.display()
                );

                if let Some(ref changed_files) = resolved_hooks.changed_files {
                    println!("File filtering enabled: {} changed files", changed_files.len());
                    if changed_files.is_empty() {
                        println!("No files changed - hook may be skipped");
                    }
                } else {
                    println!("File filtering disabled");
                }

                println!(
                    "Running hook: {} ({} resolved hooks)",
                    hook_name,
                    resolved_hooks.hooks.len()
                );
            }

            let results =
                HookExecutor::execute(&resolved_hooks).context("Failed to execute hook")?;

            if debug::is_enabled() && atty::is(atty::Stream::Stdout) {
                println!("\x1b[38;5;198m{}\x1b[0m", "═".repeat(60));
                if results.success {
                    println!("\x1b[38;5;46m🎊 \x1b[1m\x1b[38;5;82mHOOK SUCCEEDED!\x1b[0m \x1b[38;5;46m🎊\x1b[0m");
                    println!("\x1b[38;5;118m✨ Hook '{}' completed successfully! ✨\x1b[0m", hook_name);
                } else {
                    println!("\x1b[38;5;196m💥 \x1b[1m\x1b[38;5;199mHOOK FAILED!\x1b[0m \x1b[38;5;196m💥\x1b[0m");
                    let failed = results.get_failed_hooks();
                    println!("\x1b[38;5;197m🚨 Failed hooks: \x1b[38;5;167m{}\x1b[0m", failed.join(", "));
                }
                println!("\x1b[38;5;198m{}\x1b[0m", "═".repeat(60));
            }

            results.print_summary();

            if !results.success {
                process::exit(1);
            }
        }
        None => {
            println!("No hook found with name: {hook_name}");
            println!("Available hooks can be found by running: peter-hook validate");
            process::exit(1);
        }
    }

    Ok(())
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
