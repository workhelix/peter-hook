# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a git hooks manager designed for monorepos, allowing individual paths within a monorepo to have custom hooks. The system supports hierarchical hook definitions with TOML configuration files.

## Architecture Requirements

### Hook System Design
- Hooks are defined as shell executables (bash scripts or execve-style lists)
- Hooks must be combinable into larger groups (e.g., `python.lint` combining `python.ruff` and `python.ty`)
- Support for a library of reusable hooks that users can combine
- Quality bars can be imposed across the entire repository

### Hierarchical Configuration
- Uses `hooks.toml` files for configuration
- Nearest definition file takes precedence (e.g., `/projects/foo/bar/hooks.toml` > `/projects/foo/hooks.toml` > `/projects/hooks.toml`)
- Hook scripts run in the directory of their definition file, NOT from git root
- All configuration must be in TOML format

### Target Hooks
- Initially focusing on `pre-commit` and `pre-push` hooks
- Extensible to other git hooks

## Development Requirements

### Code Quality Standards
- **Everything must be tested** - no exceptions
- Extremely strict linting requirements
- Cross-platform support: macOS (primary) and Linux

### Deployment Strategy
- Code should live in separate repository for reusability
- Quick installation method (potentially `curl | bash` style)
- Open source eventual goal

## Implementation Notes

- Language agnostic - choose appropriate technology for the task
- Focus on monorepo structure support
- Hierarchical hook resolution is core functionality
- Configuration-driven approach with TOML files