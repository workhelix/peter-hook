//! Git Hook Manager - A hierarchical git hooks manager for monorepos

use anyhow::{Context, Result};
use clap::Parser;
use git_hook_manager::{
    cli::{Cli, Commands},
    hooks::{HookExecutor, HookResolver},
};
use std::env;
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
        Commands::Install => {
            install_hooks();
            Ok(())
        }
        Commands::Run { event } => run_hooks(&event),
        Commands::Validate => validate_config(),
    }
}

/// Install git hooks for the current repository
fn install_hooks() {
    println!("Installing git hooks...");
    
    // TODO: Implement git hook installation
    // This should:
    // 1. Find the git repository root
    // 2. Create/update hook files in .git/hooks/
    // 3. Make them executable
    // 4. Configure them to call this binary with the appropriate event
    
    println!("Hook installation not yet implemented");
}

/// Run hooks for a specific git event
fn run_hooks(event: &str) -> Result<()> {
    let current_dir = env::current_dir()
        .context("Failed to get current working directory")?;
    
    let resolver = HookResolver::new(&current_dir);
    
    match resolver.resolve_hooks(event)? {
        Some(resolved_hooks) => {
            println!("Found hooks configuration: {}", resolved_hooks.config_path.display());
            println!("Running {} hooks for event: {}", resolved_hooks.hooks.len(), event);
            
            let results = HookExecutor::execute(&resolved_hooks)
                .context("Failed to execute hooks")?;
            
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
fn validate_config() -> Result<()> {
    let current_dir = env::current_dir()
        .context("Failed to get current working directory")?;
    
    let resolver = HookResolver::new(&current_dir);
    
    match resolver.find_config_file()? {
        Some(config_path) => {
            println!("Validating config file: {}", config_path.display());
            
            // Try to parse the configuration
            match git_hook_manager::HookConfig::from_file(&config_path) {
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
        None => {
            println!("No hooks.toml file found in current directory or parent directories");
        }
    }
    
    Ok(())
}
