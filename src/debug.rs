//! Global debug state management

use std::sync::atomic::{AtomicBool, Ordering};

/// Global debug state
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Enable debug mode
pub fn enable() {
    DEBUG_ENABLED.store(true, Ordering::Relaxed);
}

/// Check if debug mode is enabled
pub fn is_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::Relaxed)
}

/// Disable debug mode (for testing)
#[cfg(test)]
pub fn disable() {
    DEBUG_ENABLED.store(false, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_initially_disabled() {
        // Reset state
        disable();
        assert!(!is_enabled(), "Debug should be disabled by default");
    }

    #[test]
    fn test_debug_enable() {
        disable();
        assert!(!is_enabled());

        enable();
        assert!(is_enabled(), "Debug should be enabled after enable()");

        // Clean up
        disable();
    }

    #[test]
    fn test_debug_enable_disable_toggle() {
        disable();
        assert!(!is_enabled());

        enable();
        assert!(is_enabled());

        disable();
        assert!(!is_enabled());

        enable();
        assert!(is_enabled());

        // Clean up
        disable();
    }

    #[test]
    fn test_debug_multiple_enables() {
        disable();

        enable();
        enable();
        enable();

        assert!(
            is_enabled(),
            "Should remain enabled after multiple enable() calls"
        );

        // Clean up
        disable();
    }
}
