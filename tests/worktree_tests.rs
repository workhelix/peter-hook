#![allow(clippy::all, clippy::pedantic, clippy::nursery)]
//! Integration tests for worktree functionality

use git2::Repository as Git2Repository;
use peter_hook::{
    config::TemplateResolver,
    git::{GitHookInstaller, GitRepository, WorktreeHookStrategy},
    hooks::resolver::WorktreeContext,
};
use tempfile::TempDir;

#[test]
fn test_worktree_template_variables() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path();
    let working_dir = temp_dir.path();

    // Create a mock worktree context
    let worktree_context = WorktreeContext {
        is_worktree: true,
        worktree_name: Some("feature-branch".to_string()),
        repo_root: temp_dir.path().to_path_buf(),
        common_dir: temp_dir.path().join(".git"),
        working_dir: temp_dir.path().to_path_buf(),
    };

    let resolver =
        TemplateResolver::with_worktree_context(config_dir, working_dir, &worktree_context);

    // Test worktree-specific template variables
    let variables = resolver.get_available_variables();

    assert_eq!(variables.get("IS_WORKTREE").unwrap(), "true");
    assert_eq!(variables.get("WORKTREE_NAME").unwrap(), "feature-branch");
    assert!(variables.contains_key("COMMON_DIR"));
    assert!(variables.contains_key("REPO_ROOT"));

    // Test template resolution in strings
    let test_command = "echo 'Working in worktree: {WORKTREE_NAME}, is_worktree: {IS_WORKTREE}'";
    let resolved_text = resolver.resolve_string(test_command).unwrap();

    assert!(resolved_text.contains("feature-branch"));
    assert!(resolved_text.contains("true"));
}

#[test]
fn test_main_repository_template_variables() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path();
    let working_dir = temp_dir.path();

    // Create a mock main repository context
    let worktree_context = WorktreeContext {
        is_worktree: false,
        worktree_name: None,
        repo_root: temp_dir.path().to_path_buf(),
        common_dir: temp_dir.path().join(".git"),
        working_dir: temp_dir.path().to_path_buf(),
    };

    let resolver =
        TemplateResolver::with_worktree_context(config_dir, working_dir, &worktree_context);

    // Test main repository template variables
    let variables = resolver.get_available_variables();

    assert_eq!(variables.get("IS_WORKTREE").unwrap(), "false");
    assert!(!variables.contains_key("WORKTREE_NAME"));
    assert!(variables.contains_key("COMMON_DIR"));
    assert!(variables.contains_key("REPO_ROOT"));

    // Test template resolution
    let test_command = "echo 'In main repo: {IS_WORKTREE}'";
    let resolved_text = resolver.resolve_string(test_command).unwrap();

    assert!(resolved_text.contains("false"));
}

#[test]
fn test_git_repository_worktree_detection() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();

    // Initialize a real git repository
    let _repo = Git2Repository::init(temp_dir.path()).unwrap();

    // Test finding the repository
    let git_repo = GitRepository::find_from_dir(temp_dir.path()).unwrap();

    // Should detect as main repository (not worktree)
    assert!(!git_repo.is_worktree);
    assert!(git_repo.worktree_name.is_none());
    // Use canonicalized paths to handle macOS /private/var vs /var differences
    let expected_root = temp_dir.path().canonicalize().unwrap();
    let actual_root = git_repo.root.canonicalize().unwrap();
    assert_eq!(actual_root, expected_root);

    let expected_common = temp_dir.path().join(".git").canonicalize().unwrap();
    let actual_common = git_repo.common_dir.canonicalize().unwrap();
    assert_eq!(actual_common, expected_common);
}

#[test]
fn test_worktree_hook_strategy_parsing() {
    // Test strategy parsing
    assert_eq!(
        "shared".parse::<WorktreeHookStrategy>().ok(),
        Some(WorktreeHookStrategy::Shared)
    );
    assert_eq!(
        "per-worktree".parse::<WorktreeHookStrategy>().ok(),
        Some(WorktreeHookStrategy::PerWorktree)
    );
    assert_eq!(
        "per_worktree".parse::<WorktreeHookStrategy>().ok(),
        Some(WorktreeHookStrategy::PerWorktree)
    );
    assert_eq!(
        "detect".parse::<WorktreeHookStrategy>().ok(),
        Some(WorktreeHookStrategy::Detect)
    );
    assert_eq!("invalid".parse::<WorktreeHookStrategy>().ok(), None);

    // Test string representation
    assert_eq!(WorktreeHookStrategy::Shared.as_str(), "shared");
    assert_eq!(WorktreeHookStrategy::PerWorktree.as_str(), "per-worktree");
    assert_eq!(WorktreeHookStrategy::Detect.as_str(), "detect");

    // Test default
    assert_eq!(
        WorktreeHookStrategy::default(),
        WorktreeHookStrategy::Shared
    );
}

#[test]
fn test_git_hook_installer_with_strategy() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize a git repository
    let _ = Git2Repository::init(temp_dir.path()).unwrap();

    // Create a GitRepository instance
    let repo = GitRepository::find_from_dir(temp_dir.path()).unwrap();

    // Test creating installer with different strategies
    let installer_shared = GitHookInstaller::with_repository_binary_and_strategy(
        repo.clone(),
        "test-binary".to_string(),
        WorktreeHookStrategy::Shared,
    );

    let installer_per_worktree = GitHookInstaller::with_repository_binary_and_strategy(
        repo.clone(),
        "test-binary".to_string(),
        WorktreeHookStrategy::PerWorktree,
    );

    let installer_detect = GitHookInstaller::with_repository_binary_and_strategy(
        repo,
        "test-binary".to_string(),
        WorktreeHookStrategy::Detect,
    );

    // These should create successfully without panicking
    drop(installer_shared);
    drop(installer_per_worktree);
    drop(installer_detect);
}
