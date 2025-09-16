//! Git worktree support and utilities

/// Strategy for installing hooks in a worktree environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorktreeHookStrategy {
    /// Install hooks to shared location (default behavior, backward compatible)
    #[default]
    Shared,
    /// Install hooks to worktree-specific location
    PerWorktree,
    /// Auto-detect strategy based on existing configuration
    Detect,
}

impl std::str::FromStr for WorktreeHookStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "shared" => Ok(Self::Shared),
            "per-worktree" | "per_worktree" | "perworktree" => Ok(Self::PerWorktree),
            "detect" | "auto" => Ok(Self::Detect),
            _ => Err(format!("invalid worktree strategy: {s}")),
        }
    }
}

impl WorktreeHookStrategy {
    /// Get the string representation
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Shared => "shared",
            Self::PerWorktree => "per-worktree",
            Self::Detect => "detect",
        }
    }
}

impl std::fmt::Display for WorktreeHookStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", (*self).as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_from_str() {
        let parse = |s: &str| s.parse::<WorktreeHookStrategy>().ok();
        assert_eq!(parse("shared"), Some(WorktreeHookStrategy::Shared));
        assert_eq!(
            parse("per-worktree"),
            Some(WorktreeHookStrategy::PerWorktree)
        );
        assert_eq!(
            parse("per_worktree"),
            Some(WorktreeHookStrategy::PerWorktree)
        );
        assert_eq!(
            parse("perworktree"),
            Some(WorktreeHookStrategy::PerWorktree)
        );
        assert_eq!(parse("detect"), Some(WorktreeHookStrategy::Detect));
        assert_eq!(parse("auto"), Some(WorktreeHookStrategy::Detect));
        assert_eq!(parse("invalid"), None);
    }

    #[test]
    fn test_strategy_as_str() {
        assert_eq!(WorktreeHookStrategy::Shared.as_str(), "shared");
        assert_eq!(WorktreeHookStrategy::PerWorktree.as_str(), "per-worktree");
        assert_eq!(WorktreeHookStrategy::Detect.as_str(), "detect");
    }

    #[test]
    fn test_strategy_display() {
        assert_eq!(format!("{}", WorktreeHookStrategy::Shared), "shared");
        assert_eq!(
            format!("{}", WorktreeHookStrategy::PerWorktree),
            "per-worktree"
        );
        assert_eq!(format!("{}", WorktreeHookStrategy::Detect), "detect");
    }

    #[test]
    fn test_default_strategy() {
        assert_eq!(
            WorktreeHookStrategy::default(),
            WorktreeHookStrategy::Shared
        );
    }
}
