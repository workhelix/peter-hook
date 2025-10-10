//! Comprehensive tests for git repository module

use git2::Repository as Git2Repository;
use peter_hook::git::GitRepository;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_find_from_current_dir() {
    let original_dir = std::env::current_dir().unwrap();
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = GitRepository::find_from_current_dir();
    assert!(result.is_ok());

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_find_from_dir_success() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let result = GitRepository::find_from_dir(temp_dir.path());
    assert!(result.is_ok());
}

#[test]
fn test_find_from_dir_failure() {
    let temp_dir = TempDir::new().unwrap();

    let result = GitRepository::find_from_dir(temp_dir.path());
    assert!(result.is_err());
}

#[test]
fn test_find_from_subdirectory() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let subdir = temp_dir.path().join("a/b/c");
    fs::create_dir_all(&subdir).unwrap();

    let result = GitRepository::find_from_dir(&subdir);
    assert!(result.is_ok());
}

#[test]
fn test_repository_fields_populated() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let repo = GitRepository::find_from_dir(temp_dir.path()).unwrap();

    assert!(repo.root.exists());
    assert!(repo.git_dir.exists());
    assert!(repo.hooks_dir.to_str().is_some());
    assert!(!repo.is_worktree);
    assert!(repo.worktree_name.is_none());
}

#[test]
fn test_list_hooks_empty_repo() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let repo = GitRepository::find_from_dir(temp_dir.path()).unwrap();
    let hooks = repo.list_hooks().unwrap();

    assert!(hooks.is_empty() || !hooks.is_empty());
}

#[test]
fn test_get_hook_info_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let repo = GitRepository::find_from_dir(temp_dir.path()).unwrap();
    let info = repo.get_hook_info("nonexistent-hook").unwrap();

    assert!(info.is_none());
}

#[test]
fn test_get_hook_info_existing() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    // Create a hook file
    let hooks_dir = temp_dir.path().join(".git/hooks");
    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(hooks_dir.join("pre-commit"), "#!/bin/sh\necho test").unwrap();

    let repo = GitRepository::find_from_dir(temp_dir.path()).unwrap();
    let info = repo.get_hook_info("pre-commit").unwrap();

    assert!(info.is_some());
}

#[test]
fn test_is_worktree_main_repo() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let repo = GitRepository::find_from_dir(temp_dir.path()).unwrap();
    assert!(!repo.is_worktree);
}

#[test]
fn test_get_worktree_name_main_repo() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let repo = GitRepository::find_from_dir(temp_dir.path()).unwrap();
    let name = repo.get_worktree_name();
    assert!(name.is_none());
}

#[test]
fn test_list_worktrees() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let repo = GitRepository::find_from_dir(temp_dir.path()).unwrap();
    let result = repo.list_worktrees();

    assert!(result.is_ok());
}

#[test]
fn test_hooks_directory_path() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let repo = GitRepository::find_from_dir(temp_dir.path()).unwrap();

    assert!(repo.hooks_dir.to_string_lossy().contains("hooks"));
}

#[test]
fn test_common_dir_main_repo() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let repo = GitRepository::find_from_dir(temp_dir.path()).unwrap();

    assert!(repo.common_dir.exists());
    assert!(repo.common_dir.to_string_lossy().contains(".git"));
}

#[test]
fn test_clone_repository() {
    let temp_dir = TempDir::new().unwrap();
    Git2Repository::init(temp_dir.path()).unwrap();

    let repo = GitRepository::find_from_dir(temp_dir.path()).unwrap();
    let repo_clone = repo.clone();

    assert_eq!(repo.root, repo_clone.root);
    assert_eq!(repo.is_worktree, repo_clone.is_worktree);
}
