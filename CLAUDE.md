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

# Validate configuration
cargo run -- validate
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
env = { KEY = "value" }              # Optional: environment variables
files = ["**/*.rs", "Cargo.toml"]    # Optional: file patterns for targeting
depends_on = ["format", "setup"]     # Optional: hook dependencies
run_always = false                   # Optional: ignore file changes
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

- Hook scripts run from their configuration file directory (NOT git root)
- Hierarchical resolution: child directories override parent configurations
- Thread-safe parallel execution with proper error handling
- Backward compatibility maintained for deprecated `parallel` field in groups

## Advanced Features

### Template Variables
- `{HOOK_DIR}`, `{REPO_ROOT}`, `{PROJECT_NAME}` for dynamic paths
- All environment variables available as templates
- Shell-style expansions supported (`${PWD##*/}`)

### Hook Dependencies  
- Use `depends_on = ["hook1", "hook2"]` to ensure execution order
- Automatic topological sorting with cycle detection
- Dependencies respected even in parallel execution groups

### File Pattern Targeting
- Use `files = ["**/*.rs"]` to run hooks only when specific files change
- Supports glob patterns for precise targeting
- Use `run_always = true` to bypass file filtering
- Enable with `--files` flag: `peter-hook run pre-commit --files`

### Git Integration
- Supports 15+ git hook events (pre-commit, commit-msg, pre-push, etc.)
- Automatic git argument passing for hooks that need them
- Smart change detection for file targeting (working directory vs push changes)