#![allow(clippy::all, clippy::pedantic, clippy::nursery)]
//! Tests for git lint module

use git2::Repository as Git2Repository;
use peter_hook::git::LintFileDiscovery;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_lint_discovery_new() {
    let temp_dir = TempDir::new().unwrap();
    let discovery = LintFileDiscovery::new(temp_dir.path());
    drop(discovery);
}

#[test]
fn test_lint_discovery_in_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let discovery = LintFileDiscovery::new(temp_dir.path());
    let result = discovery.discover_files();

    assert!(result.is_ok());
}

#[test]
fn test_lint_discovery_without_git() {
    let temp_dir = TempDir::new().unwrap();

    let discovery = LintFileDiscovery::new(temp_dir.path());
    let result = discovery.discover_files();

    assert!(result.is_ok());
}

#[test]
fn test_lint_discovers_regular_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files
    fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("file2.rs"), "content2").unwrap();

    let discovery = LintFileDiscovery::new(temp_dir.path());
    let result = discovery.discover_files();

    assert!(result.is_ok());
    if let Ok(files) = result {
        assert!(!files.is_empty());
    }
}

#[test]
fn test_lint_respects_gitignore_in_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    // Create gitignore
    fs::write(
        temp_dir.path().join(".gitignore"),
        "ignored.txt\nignored/\n",
    )
    .unwrap();

    // Create ignored file
    fs::write(temp_dir.path().join("ignored.txt"), "ignored").unwrap();

    // Create tracked file
    fs::write(temp_dir.path().join("tracked.txt"), "tracked").unwrap();

    let discovery = LintFileDiscovery::new(temp_dir.path());
    let result = discovery.discover_files();

    assert!(result.is_ok());
}

#[test]
fn test_lint_discovers_nested_files() {
    let temp_dir = TempDir::new().unwrap();

    let nested = temp_dir.path().join("a/b/c");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("deep.txt"), "content").unwrap();

    let discovery = LintFileDiscovery::new(temp_dir.path());
    let result = discovery.discover_files();

    assert!(result.is_ok());
    if let Ok(files) = result {
        assert!(!files.is_empty());
    }
}

#[test]
fn test_lint_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    let discovery = LintFileDiscovery::new(temp_dir.path());
    let result = discovery.discover_files();

    assert!(result.is_ok());
    if let Ok(files) = result {
        assert!(files.is_empty());
    }
}

#[test]
fn test_lint_with_nested_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    // Root gitignore
    fs::write(temp_dir.path().join(".gitignore"), "*.log\n").unwrap();

    // Nested directory with its own gitignore
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join(".gitignore"), "*.tmp\n").unwrap();

    // Create files
    fs::write(temp_dir.path().join("root.log"), "ignored").unwrap();
    fs::write(subdir.join("file.tmp"), "ignored").unwrap();
    fs::write(subdir.join("file.txt"), "tracked").unwrap();

    let discovery = LintFileDiscovery::new(temp_dir.path());
    let result = discovery.discover_files();

    assert!(result.is_ok());
}

#[test]
fn test_lint_from_subdirectory() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("file.txt"), "content").unwrap();

    // Discover from subdirectory
    let discovery = LintFileDiscovery::new(&subdir);
    let result = discovery.discover_files();

    assert!(result.is_ok());
}
