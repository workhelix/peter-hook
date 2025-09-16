# Peter Hook

A hierarchical git hooks manager designed for monorepos with safe parallel execution.

> **⚠️ Breaking Change in v3.0.0**: Template syntax changed from `${VAR}` to `{VAR}` for enhanced security. See [Template Variables](#template-variables) section for details.

## Overview

Peter Hook enables different paths within a monorepo to have their own custom git hooks while maintaining repository-wide quality standards. It features intelligent parallel execution that respects file system safety - read-only hooks run concurrently for speed, while repository-modifying hooks run sequentially to prevent conflicts.

## Key Features

- **🏗️ Hierarchical Configuration**: Nearest `hooks.toml` file wins, enabling path-specific customization
- **⚡ Safe Parallel Execution**: Automatic parallelization of compatible hooks for 2-3x speed improvement
- **🔗 Hook Composition**: Combine individual hooks into reusable groups
- **🛡️ Repository Safety**: File-modifying hooks never run simultaneously, preventing race conditions
- **🌳 Git Worktree Support**: Native support for git worktrees with flexible hook installation strategies
- **🌍 Cross-Platform**: Native binaries for macOS, Linux, and Windows
- **📦 Easy Installation**: Single-command installation with automatic PATH setup

## Quick Start

### Installation

#### Option 1: Using GitHub CLI (Recommended)

```bash
# Create installation directory
mkdir -p "$HOME/.local/bin"

# Download and extract latest release directly (v3.0.1)
gh release download --repo example/peter-hook --pattern '*-apple-darwin.tar.gz' -O - | tar -xz -C "$HOME/.local/bin"

# Add to PATH if not already present
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc  # or ~/.zshrc
source ~/.bashrc  # or source ~/.zshrc

# Verify installation
peter-hook --version
```

#### Option 2: Install Script

```bash
# Install via curl (internal project)
curl -fsSL https://raw.githubusercontent.com/example/peter-hook/main/install.sh | bash
```

#### Option 3: Manual Download

Visit: https://github.com/example/peter-hook/releases (latest: v3.0.1)

### Basic Usage

1. **Create a configuration file** (`hooks.toml`):

```toml
# Individual hooks
[hooks.lint]
command = "cargo clippy --all-targets -- -D warnings"
description = "Run Clippy linter"
modifies_repository = false

[hooks.test]
command = "cargo test --all"
description = "Run test suite"
modifies_repository = false

[hooks.format]
command = "cargo fmt"
description = "Format code"
modifies_repository = true

# Hook group with parallel execution
[groups.pre-commit]
includes = ["lint", "test", "format"]
execution = "parallel"
description = "Pre-commit validation - safe hooks run parallel, format runs after"
```

2. **Validate your configuration**:

```bash
peter-hook validate
```

3. **Run hooks manually**:

```bash
# Run individual hook by name
peter-hook run-by-name lint

# Run hook group (with intelligent parallel execution)
peter-hook run-hook pre-commit
```

4. **Install git hooks**:

```bash
peter-hook install
```

## Configuration Reference

### Complete Hook Definition

```toml
[hooks.example]
# REQUIRED: Command to execute
command = "echo hello"                      # String format
# OR
command = ["echo", "hello", "world"]        # Array format (preferred for complex commands)

# REQUIRED: Repository safety flag
modifies_repository = false                 # true = modifies files, false = read-only

# OPTIONAL: Description
description = "Example hook description"

# OPTIONAL: File targeting (performance optimization)
files = ["**/*.rs", "Cargo.toml"]          # Glob patterns - hook only runs if these files changed
run_always = false                         # true = ignore file changes, always run

# OPTIONAL: Hook dependencies  
depends_on = ["format", "setup"]           # This hook runs after these hooks complete successfully

# OPTIONAL: Working directory
workdir = "custom/path"                    # Relative to config file directory
# OR with templating
workdir = "{REPO_ROOT}/backend"           # Template variables available

# OPTIONAL: Environment variables
env = { KEY = "value" }                    # Simple key-value pairs
# OR with templating
env = { 
    PROJECT_ROOT = "{REPO_ROOT}", 
    BUILD_DIR = "{HOOK_DIR}/target",
    PROJECT_NAME = "{PROJECT_NAME}"
}
```

### Hook Groups

```toml
[groups.example-group]
# REQUIRED: Hooks and groups to include
includes = ["hook1", "hook2", "other-group"]

# OPTIONAL: Execution strategy
execution = "parallel"                     # parallel | sequential | force-parallel

# OPTIONAL: Description  
description = "Example group description"

# DEPRECATED (but supported): Legacy parallel flag
parallel = true                            # Use execution = "parallel" instead
```

### Imports (Hook Libraries)

Share and reuse hooks/groups across files, with local overrides.

```toml
# hooks.toml (project)
imports = ["../hooks.lib.toml", ".hooks/common.toml"]

[groups.pre-commit]
includes = ["lint", "format", "test"]   # names from imported files

# Local definitions override imported ones on name conflicts
[hooks.lint]
command = "cargo clippy -- -D warnings"
modifies_repository = false
```

Rules:
- Paths must be relative to the importing file (absolute paths are rejected).
- All imported files must reside under the repository root.
- Imports are merged in listed order; later imports override earlier ones; local overrides all.
- Recursive imports supported with cycle detection; cycles are ignored safely.

### Execution Strategies Explained

- **`sequential`** (default): Run hooks one after another, respecting dependencies
- **`parallel`**: Intelligent execution:
  - Repository-safe hooks (`modifies_repository = false`) run in parallel
  - Repository-modifying hooks run sequentially  
  - Dependencies always respected
- **`force-parallel`**: Force all hooks to run in parallel (dangerous - can cause file conflicts)

### Template Variables

Peter Hook supports powerful template variables in commands, working directories, and environment variables:

#### Built-in Variables
```toml
{HOOK_DIR}         # Directory containing the hooks.toml file
{WORKING_DIR}      # Current working directory when hook runs
{REPO_ROOT}        # Git repository root directory
{HOOK_DIR_REL}     # Relative path from repo root to hook directory
{WORKING_DIR_REL}  # Relative path from repo root to working directory
{PROJECT_NAME}     # Name of the directory containing hooks.toml
{HOME_DIR}         # User's home directory
{IS_WORKTREE}      # "true" or "false" - whether running in a worktree
{WORKTREE_NAME}    # Name of current worktree (only available in worktrees)
{COMMON_DIR}       # Path to shared git directory (across worktrees)
{CHANGED_FILES}    # Space-delimited list of changed files (file filtering enabled)
{CHANGED_FILES_LIST} # Newline-delimited list of changed files (file filtering enabled)
{CHANGED_FILES_FILE} # Path to temp file containing changed files (file filtering enabled)
```

#### Security Note & Breaking Changes

**Breaking Change in v1.1.0**: Template syntax has changed from shell-style `${VAR}` to secure `{VAR}` syntax:

- ❌ **Old (removed)**: `${VARIABLE_NAME}`, `${PWD##*/}`
- ✅ **New (secure)**: `{VARIABLE_NAME}`, `{PROJECT_NAME}`

For security reasons, only the predefined template variables listed above are available. Arbitrary environment variables are not exposed to prevent potential security vulnerabilities.

#### Template Examples
```toml
[hooks.build]
command = "cargo build --manifest-path={HOOK_DIR}/Cargo.toml"
workdir = "{REPO_ROOT}/target"
env = { 
    CARGO_TARGET_DIR = "{HOOK_DIR}/target",
    PROJECT_NAME = "{PROJECT_NAME}",
    BUILD_MODE = "debug"                     # Static values - no shell expansions
}

[hooks.test-with-context]
command = ["cargo", "test", "--manifest-path={HOOK_DIR}/Cargo.toml", "--", "--test-threads=1"]
description = "Test {PROJECT_NAME} in {HOOK_DIR_REL}"
```

### Git Worktree Support

Peter Hook provides native support for Git worktrees with flexible hook installation strategies and worktree-aware template variables.

#### Worktree Installation Strategies

Choose how hooks are installed across worktrees:

```bash
# Shared hooks (default): All worktrees use the same hooks
peter-hook install --worktree-strategy shared

# Per-worktree hooks: Each worktree has its own hooks
peter-hook install --worktree-strategy per-worktree

# Auto-detect: Use existing strategy if found, otherwise default to shared
peter-hook install --worktree-strategy detect
```

#### Worktree-Specific Template Variables

In addition to standard variables, worktrees provide additional context:

```toml
{IS_WORKTREE}      # "true" or "false" - whether running in a worktree
{WORKTREE_NAME}    # Name of the current worktree (only available in worktrees)
{COMMON_DIR}       # Path to the shared git directory across worktrees
```

#### Worktree Template Examples

```toml
[hooks.worktree-context]
command = "echo 'In worktree: {IS_WORKTREE}'"
description = "Show worktree context"

[hooks.worktree-specific]
command = "echo 'Working in {WORKTREE_NAME}'"
description = "Show current worktree name"

[hooks.backup-logs]
command = "cp logs/*.log {COMMON_DIR}/backup/"
description = "Backup logs to shared directory"
```

#### Managing Worktrees

List all worktrees and their hook status:

```bash
peter-hook list-worktrees
```

Example output:
```
Git worktrees in this repository:
=================================
📁 main [main] (current)
   Path: /Users/dev/project
   Hooks (shared): pre-commit, pre-push

📁 feature-auth
   Path: /Users/dev/project-auth
   Hooks (shared): pre-commit, pre-push

📁 hotfix-123
   Path: /Users/dev/project-hotfix
   Hooks (worktree-specific): pre-commit, custom-deploy
```

#### Worktree Configuration Examples

Per-worktree configuration allows different branches to have different validation requirements:

```toml
# Main repository hooks.toml - Strict validation
[hooks.full-test-suite]
command = "cargo test --all --release"
modifies_repository = false
description = "Complete test suite for main branch"

[hooks.security-audit]
command = "cargo audit"
modifies_repository = false 
description = "Security audit for production"

[groups.pre-commit]
includes = ["full-test-suite", "security-audit"]
execution = "parallel"
```

```toml
# Feature branch worktree hooks.toml - Fast iteration
[hooks.quick-test]
command = "cargo test --lib"
modifies_repository = false
description = "Quick unit tests for development"

[hooks.format-check]  
command = "cargo fmt --check"
modifies_repository = false
description = "Format check only"

[groups.pre-commit]
includes = ["quick-test", "format-check"]
execution = "parallel"
```

### File Pattern Targeting

Optimize performance by only running hooks when relevant files change:

#### Pattern Syntax (Glob Patterns)
```toml
# Language-specific patterns
files = ["**/*.rs", "Cargo.toml"]                    # Rust files
files = ["**/*.py", "requirements*.txt", "*.py"]     # Python files  
files = ["**/*.js", "**/*.ts", "package.json"]       # JavaScript/TypeScript
files = ["**/*.go", "go.mod", "go.sum"]              # Go files

# Build and configuration patterns
files = ["Dockerfile*", "docker-compose*.yml"]       # Docker files
files = ["**/*.yml", "**/*.yaml"]                    # YAML files
files = ["*.toml", "**/*.toml"]                      # TOML files
files = ["**/*.json"]                                # JSON files

# Documentation patterns
files = ["**/*.md", "docs/**/*", "README*"]          # Documentation
files = ["**/*.rst", "**/*.txt"]                     # Text documentation

# Mixed patterns
files = ["src/**/*.rs", "tests/**/*.rs", "Cargo.*"]  # Rust source and config
files = ["frontend/**/*", "!frontend/node_modules"]  # Frontend (excluding node_modules)
```

#### File Targeting Behavior
```toml
# No files specified = always run
[hooks.always-run-hook]
command = "security-scan"
# (no files field means run regardless of changes)

# Specific files = run only if those files changed  
[hooks.rust-only]
command = "cargo clippy"
files = ["**/*.rs"]
# Runs only if .rs files changed

# Force always run = ignore file changes
[hooks.critical-security]
command = "secret-scan" 
files = ["**/*"]           # Would normally check all files
run_always = true          # But this overrides and always runs
```

### Hook Dependencies

Control execution order with dependencies:

```toml
# Basic dependency chain
[hooks.format]
command = "cargo fmt"
modifies_repository = true

[hooks.lint] 
command = "cargo clippy"
depends_on = ["format"]              # Runs after format completes successfully
modifies_repository = false

[hooks.test]
command = "cargo test" 
depends_on = ["lint"]                # Runs after lint completes successfully
modifies_repository = false

# Multiple dependencies
[hooks.integration-test]
command = "cargo test --test integration"
depends_on = ["lint", "test"]        # Runs after BOTH lint and test complete
modifies_repository = false

# Complex dependency tree
[hooks.build]
command = "cargo build"
depends_on = ["format", "lint"]

[hooks.package]
command = "cargo package" 
depends_on = ["test", "build"]       # Depends on test AND build

[groups.release-pipeline]
includes = ["format", "lint", "test", "build", "package"]
execution = "sequential"             # Respects all dependencies
```

### Advanced Command Usage

#### Enable File Targeting
```bash
# Run with automatic file detection (faster)
peter-hook run pre-commit --files

# Run all hooks regardless of files (slower but complete)
peter-hook run pre-commit
```

#### Validate With Import Diagnostics
```bash
# Basic validation
peter-hook validate

# Trace imports (order, overrides, cycles, unused)
peter-hook validate --trace-imports

# JSON diagnostics (useful for tooling)
peter-hook validate --trace-imports --json
```

#### Git Integration
```bash
# Install hooks to run automatically with git
peter-hook install


# Install with force (backup existing hooks)
peter-hook install --force

# Install with worktree strategy
peter-hook install --worktree-strategy shared
peter-hook install --worktree-strategy per-worktree

# List all git hooks
peter-hook list

# List all worktrees and their hooks
peter-hook list-worktrees

# Uninstall peter-hook managed hooks
peter-hook uninstall

# Uninstall without confirmation
peter-hook uninstall --yes
```

#### Hook Management
```bash
# Validate configuration
peter-hook validate

# Validate with import diagnostics
peter-hook validate --trace-imports

# Run specific hook by name
peter-hook run-by-name lint

# Run with file detection (only changed files)
peter-hook run-by-name pre-commit

# Run all files (ignore file filtering)
peter-hook run-by-name pre-commit --all-files

# Test hook with git arguments (for commit-msg, pre-push hooks)
peter-hook run commit-msg /tmp/commit-msg-file
```

#### Global Configuration
```bash
# Show current global configuration
peter-hook config show

# Initialize global configuration (with absolute imports disabled)
peter-hook config init

# Initialize with absolute imports enabled
peter-hook config init --allow-local

# Validate global configuration
peter-hook config validate
```

## Hierarchical Configuration

The hook system uses a hierarchical configuration approach where the **nearest** `hooks.toml` file takes precedence:

```
/monorepo/hooks.toml                    # Repository-wide defaults
/monorepo/backend/hooks.toml           # Backend-specific hooks  
/monorepo/backend/api/hooks.toml       # API-specific hooks
```

- Changes in `/monorepo/backend/api/` use `/monorepo/backend/api/hooks.toml`
- Changes in `/monorepo/backend/auth/` use `/monorepo/backend/hooks.toml`  
- Changes in `/monorepo/frontend/` use `/monorepo/hooks.toml`

## Complete Real-World Examples

### Multi-Language Monorepo Configuration

```toml
# ===== RUST BACKEND =====
[hooks.rust-format]
command = "cargo fmt --manifest-path={HOOK_DIR}/Cargo.toml"
description = "Format Rust code"
modifies_repository = true
files = ["**/*.rs"]
workdir = "{REPO_ROOT}/backend"

[hooks.rust-lint]
command = "cargo clippy --manifest-path={HOOK_DIR}/Cargo.toml -- -D warnings"
description = "Lint Rust code (after formatting)"
modifies_repository = false
files = ["**/*.rs", "Cargo.toml"]
depends_on = ["rust-format"]
workdir = "{REPO_ROOT}/backend"

[hooks.rust-test]
command = "cargo test --manifest-path={HOOK_DIR}/Cargo.toml"
description = "Test Rust code (after linting)"
modifies_repository = false  
files = ["**/*.rs", "Cargo.toml"]
depends_on = ["rust-lint"]
workdir = "{REPO_ROOT}/backend"
env = { RUST_BACKTRACE = "1" }

# ===== FRONTEND =====
[hooks.js-format]
command = "npm run format"
description = "Format JavaScript/TypeScript"
modifies_repository = true
files = ["**/*.js", "**/*.ts", "**/*.jsx", "**/*.tsx"]
workdir = "{REPO_ROOT}/frontend"

[hooks.js-lint]  
command = "npm run lint"
description = "Lint JavaScript/TypeScript (after formatting)"
modifies_repository = false
files = ["**/*.js", "**/*.ts", "package.json"]
depends_on = ["js-format"]
workdir = "{REPO_ROOT}/frontend"

[hooks.js-test]
command = "npm test -- --passWithNoTests"
description = "Test JavaScript/TypeScript (after linting)"
modifies_repository = false
files = ["**/*.js", "**/*.ts", "**/*.test.*"]
depends_on = ["js-lint"]
workdir = "{REPO_ROOT}/frontend"

# ===== SECURITY (ALWAYS RUN) =====
[hooks.security-scan]
command = "semgrep --config=auto {REPO_ROOT}"
description = "Security scan (always runs)"
modifies_repository = false
run_always = true

[hooks.secret-scan]
command = "gitleaks detect --source={REPO_ROOT}"
description = "Secret detection (always runs)"  
modifies_repository = false
run_always = true

# ===== SMART GROUPS =====
[groups.backend-pipeline]
includes = ["rust-format", "rust-lint", "rust-test"]
execution = "sequential"  # Respects dependencies: format → lint → test
description = "Complete backend validation"

[groups.frontend-pipeline]
includes = ["js-format", "js-lint", "js-test"] 
execution = "sequential"  # Respects dependencies: format → lint → test
description = "Complete frontend validation"

[groups.security-suite]
includes = ["security-scan", "secret-scan"]
execution = "parallel"    # Both always run, no dependencies
description = "Security validation"

# ===== MAIN HOOKS =====
[groups.pre-commit]
includes = ["backend-pipeline", "frontend-pipeline", "security-suite"]
execution = "parallel"
description = "Complete validation with file targeting and dependencies"
# Result: Only relevant language pipelines run based on changed files
#         Security always runs regardless of changes
#         Dependencies respected within each pipeline

[groups.pre-push] 
includes = ["security-suite", "rust-test", "js-test"]
execution = "parallel"
description = "Quick validation before push"
```

### File Targeting in Action

```bash
# Scenario 1: Only Rust files changed
# Changed files: ["backend/src/lib.rs", "backend/Cargo.toml"]
peter-hook run-hook pre-commit
# Result: Only rust-format → rust-lint → rust-test + security runs
#         Frontend pipeline skipped (no JS/TS files changed)
#         5x faster than running everything

# Scenario 2: Only documentation changed
# Changed files: ["README.md", "docs/api.md"]
peter-hook run-hook pre-commit  
# Result: Only security-suite runs (has run_always = true)
#         All language-specific hooks skipped
#         10x faster than running everything

# Scenario 3: Mixed changes
# Changed files: ["backend/src/lib.rs", "frontend/src/app.js", "package.json"]
peter-hook run-hook pre-commit
# Result: backend-pipeline + frontend-pipeline + security-suite
#         All relevant hooks run, nothing wasted

# Scenario 4: Run everything regardless of files
peter-hook run-hook pre-commit --all-files
# Result: All hooks run (slower but comprehensive)
```

### Performance Benefits

**Real-World Monorepo Impact:**
- **🚀 5-10x faster** for single-language changes
- **🎯 Intelligent targeting** - only run what's needed
- **⚡ Parallel execution** where safe
- **🔒 Dependency guarantees** - formatters always run before linters
- **🛡️ Repository safety** - no file conflicts from parallel modification

## Repository Structure

This project follows standard Rust conventions:

- **`src/config/`**: TOML configuration parsing and validation
- **`src/hooks/`**: Hook resolution and execution engine  
- **`src/cli/`**: Command-line interface
- **`tests/`**: Integration tests and test fixtures
- **`examples/`**: Example configurations

## Development

### Prerequisites

- Rust 1.70.0 or later
- Standard development tools (git, etc.)

### Building

```bash
# Clone and build
git clone https://github.com/workhelix/peter-hook.git
cd peter-hook
cargo build --release

# Run tests
cargo test

# Run with strict linting
cargo clippy --all-targets -- -D warnings
```

### Running Locally

```bash
# Validate configuration
cargo run -- validate

# Trace imports (human-readable)
cargo run -- validate --trace-imports

# Trace imports (JSON)
cargo run -- validate --trace-imports --json

# Run hooks
cargo run -- run pre-commit

# See all options
cargo run -- --help
```

## Contributing

We welcome contributions! Please see our comprehensive CI/CD pipeline that ensures:

- ✅ Zero compiler warnings with strict linting
- ✅ 100% test coverage requirement  
- ✅ Cross-platform compatibility testing
- ✅ Security audits and dependency management
- ✅ Automated releases with checksums

## Architecture

Built with safety and performance in mind:

- **Thread-safe parallel execution** using Rust's ownership system
- **Zero-copy configuration** with efficient TOML parsing
- **Minimal dependencies** for security and reliability
- **Cross-platform native binaries** for optimal performance
- **Comprehensive error handling** with detailed diagnostics

## License

CC0 1.0 Universal (Public Domain) - see [LICENSE](LICENSE) file for details.

This work has been released into the public domain.