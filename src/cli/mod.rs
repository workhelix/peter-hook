use clap::{Parser, Subcommand};

/// Command-line interface for git hook manager
#[derive(Parser)]
#[command(name = "git-hook-manager")]
#[command(about = "A hierarchical git hooks manager for monorepos")]
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands
#[derive(Subcommand)]
pub enum Commands {
    /// Install hooks for the current repository
    Install,
    /// Run hooks for a specific git event
    Run {
        /// The git hook event (pre-commit, pre-push, etc.)
        event: String,
    },
    /// Validate hook configuration
    Validate,
}