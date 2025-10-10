#![allow(clippy::all, clippy::pedantic, clippy::nursery)]
//! Integration tests for list-worktrees command

use git2::Repository as Git2Repository;
use std::process::Command;
use tempfile::TempDir;

fn bin_path() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("peter-hook")
}

#[test]
fn test_list_worktrees_main_repo() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let output = Command::new(bin_path())
        .current_dir(temp_dir.path())
        .arg("list-worktrees")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!combined.is_empty(), "Should produce output");
}

#[test]
fn test_list_worktrees_outside_repo_fails() {
    let temp_dir = TempDir::new().unwrap();

    let output = Command::new(bin_path())
        .current_dir(temp_dir.path())
        .arg("list-worktrees")
        .output()
        .expect("Failed to execute");

    assert!(!output.status.success());
}

#[test]
fn test_list_worktrees_help() {
    let output = Command::new(bin_path())
        .arg("list-worktrees")
        .arg("--help")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("worktree") || stdout.contains("List"));
}

#[test]
fn test_list_worktrees_exit_code() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let output = Command::new(bin_path())
        .current_dir(temp_dir.path())
        .arg("list-worktrees")
        .output()
        .expect("Failed to execute");

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_list_worktrees_with_debug() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let output = Command::new(bin_path())
        .current_dir(temp_dir.path())
        .arg("--debug")
        .arg("list-worktrees")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
}

#[test]
fn test_list_worktrees_from_subdirectory() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let subdir = temp_dir.path().join("sub");
    std::fs::create_dir(&subdir).unwrap();

    let output = Command::new(bin_path())
        .current_dir(&subdir)
        .arg("list-worktrees")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
}

#[test]
fn test_list_worktrees_output_format() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let output = Command::new(bin_path())
        .current_dir(temp_dir.path())
        .arg("list-worktrees")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    // Just verify output is produced
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!combined.trim().is_empty());
}
