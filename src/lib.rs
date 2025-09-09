//! Git Hook Manager - A hierarchical git hooks manager for monorepos
//! 
//! This crate provides a system for managing git hooks in monorepo environments
//! with support for hierarchical configuration and hook composition.

/// Configuration parsing and management
pub mod config;
/// Hook resolution and execution system
pub mod hooks;
/// Command-line interface
pub mod cli;

pub use config::*;
pub use hooks::*;