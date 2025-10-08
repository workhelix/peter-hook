//! Lint mode file discovery with .gitignore support

use anyhow::{Context, Result};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

/// Discovers files for lint mode, respecting .gitignore rules
pub struct LintFileDiscovery {
    /// Starting directory for discovery
    start_dir: PathBuf,
    /// Git repository root (for finding .gitignore files)
    repo_root: Option<PathBuf>,
}

impl LintFileDiscovery {
    /// Create a new file discovery instance
    #[must_use]
    pub fn new<P: AsRef<Path>>(start_dir: P) -> Self {
        let start_dir = start_dir.as_ref().to_path_buf();

        // Try to find git root
        let repo_root = Self::find_git_root(&start_dir).ok();

        Self {
            start_dir,
            repo_root,
        }
    }

    /// Find all non-ignored files in the start directory and subdirectories
    ///
    /// # Errors
    ///
    /// Returns an error if file system operations fail
    pub fn discover_files(&self) -> Result<Vec<PathBuf>> {
        // Use git ls-files if we're in a git repo for most efficient discovery
        self.repo_root.as_ref().map_or_else(
            || self.discover_manual(),
            |repo_root| self.discover_with_git(repo_root),
        )
    }

    /// Use git ls-files to efficiently discover non-ignored files
    fn discover_with_git(&self, _repo_root: &Path) -> Result<Vec<PathBuf>> {
        // Run git ls-files from the start directory
        // This respects .gitignore rules hierarchically up to repo root
        let output = Command::new("git")
            .args([
                "ls-files",
                "--cached",           // Tracked files
                "--others",           // Untracked files
                "--exclude-standard", // Respect .gitignore
            ])
            .current_dir(&self.start_dir)
            .output()
            .context("Failed to run git ls-files")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("git ls-files failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut files = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if !line.is_empty() {
                // git ls-files returns paths relative to current directory
                let file_path = self.start_dir.join(line);
                if file_path.exists() && file_path.is_file() {
                    files.push(file_path);
                }
            }
        }

        Ok(files)
    }

    /// Manual file discovery (fallback for non-git directories)
    fn discover_manual(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut visited = HashSet::new();

        Self::walk_directory(&self.start_dir, &mut files, &mut visited)?;

        Ok(files)
    }

    /// Recursively walk directory tree
    fn walk_directory(
        dir: &Path,
        files: &mut Vec<PathBuf>,
        visited: &mut HashSet<PathBuf>,
    ) -> Result<()> {
        // Prevent infinite loops from symlinks
        let canonical = fs::canonicalize(dir)
            .with_context(|| format!("Failed to canonicalize {}", dir.display()))?;

        if !visited.insert(canonical) {
            return Ok(());
        }

        let entries = fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory {}", dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Skip hidden files and common ignore patterns
            if file_name_str.starts_with('.') {
                continue;
            }

            // Skip common build/dependency directories
            if matches!(
                file_name_str.as_ref(),
                "node_modules" | "target" | "build" | "dist" | "__pycache__" | ".venv" | "venv"
            ) {
                continue;
            }

            if path.is_dir() {
                Self::walk_directory(&path, files, visited)?;
            } else if path.is_file() {
                files.push(path);
            }
        }

        Ok(())
    }

    /// Find git repository root by looking for .git directory
    fn find_git_root(start_dir: &Path) -> Result<PathBuf> {
        let mut current = start_dir;

        loop {
            let git_dir = current.join(".git");
            if git_dir.exists() {
                return Ok(current.to_path_buf());
            }

            current = current
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Not in a git repository"))?;
        }
    }

    /// Get the repository root if we're in a git repo
    #[must_use]
    pub fn repo_root(&self) -> Option<&Path> {
        self.repo_root.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_git_repo(temp_dir: &Path) -> PathBuf {
        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(temp_dir)
            .output()
            .unwrap();

        // Configure git for tests
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_dir)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp_dir)
            .output()
            .unwrap();

        temp_dir.to_path_buf()
    }

    #[test]
    fn test_discover_files_in_git_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo_dir = create_test_git_repo(temp_dir.path());

        // Create some files
        fs::write(repo_dir.join("test1.txt"), "content1").unwrap();
        fs::write(repo_dir.join("test2.rs"), "fn main() {}").unwrap();

        let subdir = repo_dir.join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("test3.py"), "print('hello')").unwrap();

        let discovery = LintFileDiscovery::new(&repo_dir);
        let files = discovery.discover_files().unwrap();

        assert!(!files.is_empty());
        assert!(files.iter().any(|f| f.ends_with("test1.txt")));
        assert!(files.iter().any(|f| f.ends_with("test2.rs")));
        assert!(files.iter().any(|f| f.ends_with("test3.py")));
    }

    #[test]
    fn test_respects_gitignore() {
        let temp_dir = TempDir::new().unwrap();
        let repo_dir = create_test_git_repo(temp_dir.path());

        // Create .gitignore
        fs::write(repo_dir.join(".gitignore"), "*.log\nignored/\n").unwrap();

        // Create files
        fs::write(repo_dir.join("included.txt"), "include me").unwrap();
        fs::write(repo_dir.join("excluded.log"), "exclude me").unwrap();

        let ignored_dir = repo_dir.join("ignored");
        fs::create_dir(&ignored_dir).unwrap();
        fs::write(ignored_dir.join("file.txt"), "ignored").unwrap();

        // Add .gitignore to git
        Command::new("git")
            .args(["add", ".gitignore"])
            .current_dir(&repo_dir)
            .output()
            .unwrap();

        let discovery = LintFileDiscovery::new(&repo_dir);
        let files = discovery.discover_files().unwrap();

        // Should include included.txt and .gitignore
        assert!(files.iter().any(|f| f.ends_with("included.txt")));

        // Should NOT include excluded.log or files in ignored/
        assert!(!files.iter().any(|f| f.ends_with("excluded.log")));
        assert!(
            !files
                .iter()
                .any(|f| f.to_string_lossy().contains("ignored/file.txt"))
        );
    }

    #[test]
    fn test_hierarchical_gitignore() {
        let temp_dir = TempDir::new().unwrap();
        let repo_dir = create_test_git_repo(temp_dir.path());

        // Root .gitignore
        fs::write(repo_dir.join(".gitignore"), "*.log\n").unwrap();

        // Subdirectory with its own .gitignore
        let subdir = repo_dir.join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join(".gitignore"), "*.tmp\n").unwrap();

        // Create files
        fs::write(repo_dir.join("root.txt"), "root file").unwrap();
        fs::write(repo_dir.join("root.log"), "excluded by root").unwrap();
        fs::write(subdir.join("sub.txt"), "sub file").unwrap();
        fs::write(subdir.join("sub.tmp"), "excluded by subdir").unwrap();
        fs::write(subdir.join("sub.log"), "excluded by root").unwrap();

        // Add .gitignore files
        Command::new("git")
            .args(["add", ".gitignore"])
            .current_dir(&repo_dir)
            .output()
            .unwrap();

        Command::new("git")
            .args(["add", "subdir/.gitignore"])
            .current_dir(&repo_dir)
            .output()
            .unwrap();

        let discovery = LintFileDiscovery::new(&repo_dir);
        let files = discovery.discover_files().unwrap();

        // Should include .txt files
        assert!(files.iter().any(|f| f.ends_with("root.txt")));
        assert!(files.iter().any(|f| f.ends_with("sub.txt")));

        // Should NOT include .log or .tmp files
        assert!(!files.iter().any(|f| f.ends_with("root.log")));
        assert!(!files.iter().any(|f| f.ends_with("sub.tmp")));
        assert!(!files.iter().any(|f| f.ends_with("sub.log")));
    }
}
