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