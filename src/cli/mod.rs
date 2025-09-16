use clap::{Parser, Subcommand};

/// Command-line interface for peter hook manager
#[derive(Parser)]
#[command(name = "peter-hook")]
#[command(about = "A hierarchical git hooks manager for monorepos")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands
#[derive(Subcommand)]
pub enum Commands {
    /// Install hooks for the current repository
    Install {
        /// Force installation even if hooks already exist
        #[arg(long)]
        force: bool,
        /// Worktree hook installation strategy
        #[arg(long, default_value = "shared", value_parser = clap::builder::PossibleValuesParser::new(["shared", "per-worktree", "detect"]))]
        worktree_strategy: String,
    },
    /// Uninstall git-hook-manager managed hooks
    Uninstall {
        /// Remove hooks without prompting for confirmation
        #[arg(long)]
        yes: bool,
    },
    /// Run hooks for a specific git event
    Run {
        /// The git hook event (pre-commit, pre-push, etc.)
        event: String,
        /// Additional arguments passed from git (e.g., commit message file, refs)
        #[arg(trailing_var_arg = true)]
        git_args: Vec<String>,
    },
    /// Validate hook configuration
    Validate {
        /// Trace imports and show merge/override diagnostics
        #[arg(long)]
        trace_imports: bool,
        /// Output diagnostics as JSON (use with --trace-imports)
        #[arg(long)]
        json: bool,
    },
    /// List installed git hooks
    List,
    /// Run the same hooks that would run during a git operation (without doing the git operation)
    RunHook {
        /// Git event to simulate (pre-commit, pre-push, etc.)
        event: String,
    },
    /// List worktrees and their hook configuration
    ListWorktrees,
}
