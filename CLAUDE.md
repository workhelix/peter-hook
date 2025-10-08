# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a git hooks manager designed for monorepos, allowing individual paths within a monorepo to have custom hooks. The system supports hierarchical hook definitions with TOML configuration files and safe parallel execution.

## Development Commands

### Essential Commands
```bash
# Build the project
cargo build

# Run all tests
cargo test

# Run strict linting
cargo clippy --all-targets -- -D warnings

# Format code
cargo fmt

# Run the complete pre-commit check
cargo run -- run pre-commit

# Run a specific hook in lint mode (all matching files)
cargo run -- lint <hook-name>

# Validate configuration
cargo run -- validate

# Generate shell completions
cargo run -- completions bash|zsh|fish

# Health check and update notifications
cargo run -- doctor

# Self-update to latest version
cargo run -- update
```

### Testing Individual Components
```bash
# Test configuration parsing only
cargo test config::

# Test hook resolution only  
cargo test hooks::resolver::

# Test parallel execution
cargo test hooks::executor::test_parallel_safe_execution
```

## Architecture Overview

### Core Components
- **Config Parser** (`src/config/parser.rs`): TOML parsing with parallel execution flags
- **Hook Resolver** (`src/hooks/resolver.rs`): Hierarchical configuration resolution
- **Hook Executor** (`src/hooks/executor.rs`): Safe parallel execution engine
- **CLI Interface** (`src/cli/mod.rs`): Command-line interface

### Key Features
- **Hierarchical Configuration**: Nearest `hooks.toml` file wins
- **Safe Parallel Execution**: Repository-modifying hooks run sequentially, read-only hooks run in parallel
- **Hook Groups**: Combine individual hooks with execution strategies
- **Cross-platform**: Rust implementation supporting macOS, Linux, Windows

## Configuration System

### Hook Definition Structure
```toml
[hooks.example]
command = "echo hello"                # Required: command to run
description = "Example hook"          # Optional: description
modifies_repository = false           # Required: safety flag for parallel execution
workdir = "custom/path"              # Optional: override working directory
env = { KEY = "value" }              # Optional: environment variables (supports template variables)
files = ["**/*.rs", "Cargo.toml"]    # Optional: file patterns for targeting
depends_on = ["format", "setup"]     # Optional: hook dependencies
run_always = false                   # Optional: ignore file changes
run_at_root = false                  # Optional: run at repository root instead of config directory
```

**Example: Using tools from custom PATH locations**
```toml
[hooks.my-custom-tool]
command = "my-tool"
modifies_repository = false
# Extend PATH to include custom bin directory
env = { PATH = "{HOME_DIR}/.local/bin:{PATH}" }
```

### Execution Strategies
- `sequential`: Run hooks one after another (default)
- `parallel`: Run safely in parallel (respects `modifies_repository` flag) 
- `force-parallel`: Run all hooks in parallel (unsafe - ignores safety flags)

### Repository Safety Rules
- Hooks with `modifies_repository = true` NEVER run in parallel with other hooks
- Hooks with `modifies_repository = false` can run in parallel with each other
- Mixed groups run in phases: parallel safe hooks first, then sequential modifying hooks

## Code Quality Standards

- **Zero warnings policy**: All code must pass `cargo clippy -- -D warnings`
- **100% test coverage goal**: Comprehensive unit and integration tests
- **Cross-platform compatibility**: Primary macOS, support Linux/Windows
- **Security-first**: Regular dependency audits, no unsafe code allowed

## Important Implementation Details

- Hook scripts run from their configuration file directory by default (NOT git root)
- Use `run_at_root = true` to override this behavior and run at the repository root
- Hierarchical resolution: child directories override parent configurations
- Thread-safe parallel execution with proper error handling
- Backward compatibility maintained for deprecated `parallel` field in groups

## Advanced Features

### Template Variables

Template variables use `{VARIABLE_NAME}` syntax and can be used in:
- `command` field (shell commands or arguments)
- `env` field (environment variable values)
- `workdir` field (working directory paths)

**Available template variables:**
- `{HOOK_DIR}` - Directory containing the hooks.toml file
- `{REPO_ROOT}` - Git repository root directory
- `{PROJECT_NAME}` - Name of the directory containing hooks.toml
- `{HOME_DIR}` - User's home directory (from $HOME)
- `{PATH}` - Current PATH environment variable
- `{WORKING_DIR}` - Current working directory
- `{CHANGED_FILES}` - Space-delimited list of changed files (when using `--files`)
- `{CHANGED_FILES_LIST}` - Newline-delimited list of changed files
- `{CHANGED_FILES_FILE}` - Path to temporary file containing changed files

**Common use cases:**
```toml
# Run tool from custom PATH location (Method 1: extend PATH)
[hooks.custom-tool]
command = "my-tool --check"
env = { PATH = "{HOME_DIR}/.local/bin:{PATH}" }

# Run tool from custom PATH location (Method 2: absolute path)
[hooks.custom-tool-direct]
command = "{HOME_DIR}/.local/bin/my-tool --check"

# Use repository root in command
[hooks.build]
command = "make -C {REPO_ROOT} build"

# Set environment variables with templates
[hooks.test]
command = "pytest"
env = {
  PROJECT_ROOT = "{REPO_ROOT}",
  BUILD_DIR = "{REPO_ROOT}/target",
  PATH = "{HOME_DIR}/.local/bin:{PATH}"
}
```

**Security note:** Only whitelisted template variables are available. Arbitrary environment variables are not exposed to prevent security issues.

### Hook Dependencies  
- Use `depends_on = ["hook1", "hook2"]` to ensure execution order
- Automatic topological sorting with cycle detection
- Dependencies respected even in parallel execution groups

### File Pattern Targeting
- Use `files = ["**/*.rs"]` to run hooks only when specific files change
- Supports glob patterns for precise targeting
- Use `run_always = true` to bypass file filtering
- Enable with `--files` flag: `peter-hook run pre-commit --files`

### Lint Mode
- Run hooks on ALL matching files with `lint <hook-name>`
- Treats current directory as repository root
- Discovers all non-ignored files respecting .gitignore
- No git operations - pure file discovery and execution
- Usage: `peter-hook lint <hook-name> [--dry-run]`
- Perfect for:
  - Running linters/formatters on entire codebase
  - Pre-CI validation without git operations
  - Per-directory validation (e.g., `unvenv`)
  - One-off quality checks

**Execution modes in lint:**
- `per-file`: Files passed as arguments (e.g., `ruff check file1.py file2.py`)
- `per-directory`: Runs once per directory with matching files
- `other`: Uses template variables for manual file handling

### Git Integration
- Supports 15+ git hook events (pre-commit, commit-msg, pre-push, etc.)
- Automatic git argument passing for hooks that need them
- Smart change detection for file targeting (working directory vs push changes)