#![allow(clippy::all, clippy::pedantic, clippy::nursery)]
//! Comprehensive tests for output formatting module

use peter_hook::output::OutputFormatter;

#[test]
fn test_hook_start_tty() {
    let formatter = OutputFormatter::with_tty(true);
    let result = formatter.hook_start("test-hook");
    assert!(result.contains("test-hook"));
}

#[test]
fn test_hook_start_non_tty() {
    let formatter = OutputFormatter::with_tty(false);
    let result = formatter.hook_start("test-hook");
    assert_eq!(result, "Running: test-hook");
}

#[test]
fn test_hook_result_success_tty() {
    let formatter = OutputFormatter::with_tty(true);
    let result = formatter.hook_result("test-hook", true, 0);
    assert!(result.contains("test-hook"));
    assert!(result.contains('0'));
}

#[test]
fn test_hook_result_failure_tty() {
    let formatter = OutputFormatter::with_tty(true);
    let result = formatter.hook_result("test-hook", false, 1);
    assert!(result.contains("test-hook"));
    assert!(result.contains('1'));
}

#[test]
fn test_hook_result_success_non_tty() {
    let formatter = OutputFormatter::with_tty(false);
    let result = formatter.hook_result("test-hook", true, 0);
    assert_eq!(result, "[PASS] test-hook: exit code 0");
}

#[test]
fn test_hook_result_failure_non_tty() {
    let formatter = OutputFormatter::with_tty(false);
    let result = formatter.hook_result("test-hook", false, 1);
    assert_eq!(result, "[FAIL] test-hook: exit code 1");
}

#[test]
fn test_section_header_tty() {
    let formatter = OutputFormatter::with_tty(true);
    let result = formatter.section_header("Test Section");
    assert!(result.contains("Test Section"));
}

#[test]
fn test_section_header_non_tty() {
    let formatter = OutputFormatter::with_tty(false);
    let result = formatter.section_header("Test Section");
    assert_eq!(result, "=== Test Section ===");
}

#[test]
fn test_overall_result_success_tty() {
    let formatter = OutputFormatter::with_tty(true);
    let result = formatter.overall_result(true);
    assert!(result.contains("success") || result.contains("SUCCESS"));
}

#[test]
fn test_overall_result_failure_tty() {
    let formatter = OutputFormatter::with_tty(true);
    let result = formatter.overall_result(false);
    assert!(result.contains("fail") || result.contains("FAIL"));
}

#[test]
fn test_overall_result_success_non_tty() {
    let formatter = OutputFormatter::with_tty(false);
    let result = formatter.overall_result(true);
    assert_eq!(result, "Overall: SUCCESS");
}

#[test]
fn test_overall_result_failure_non_tty() {
    let formatter = OutputFormatter::with_tty(false);
    let result = formatter.overall_result(false);
    assert_eq!(result, "Overall: FAILURE");
}

#[test]
fn test_restore_symbol() {
    let formatter_tty = OutputFormatter::with_tty(true);
    let formatter_non_tty = OutputFormatter::with_tty(false);

    assert_eq!(formatter_tty.restore(), "🔄");
    assert_eq!(formatter_non_tty.restore(), "[RESTORE]");
}

#[test]
fn test_backup_symbol() {
    let formatter_tty = OutputFormatter::with_tty(true);
    let formatter_non_tty = OutputFormatter::with_tty(false);

    assert_eq!(formatter_tty.backup(), "💾");
    assert_eq!(formatter_non_tty.backup(), "[BACKUP]");
}

#[test]
fn test_skip_symbol() {
    let formatter_tty = OutputFormatter::with_tty(true);
    let formatter_non_tty = OutputFormatter::with_tty(false);

    assert_eq!(formatter_tty.skip(), "⏭️");
    assert_eq!(formatter_non_tty.skip(), "[SKIP]");
}

#[test]
fn test_create_progress_bar_tty_multiple_items() {
    let formatter = OutputFormatter::with_tty(true);
    let pb = formatter.create_progress_bar(5);
    assert!(pb.is_some());
}

#[test]
fn test_create_progress_bar_tty_single_item() {
    let formatter = OutputFormatter::with_tty(true);
    let pb = formatter.create_progress_bar(1);
    assert!(
        pb.is_none(),
        "Should not create progress bar for single item"
    );
}

#[test]
fn test_create_progress_bar_non_tty() {
    let formatter = OutputFormatter::with_tty(false);
    let pb = formatter.create_progress_bar(10);
    assert!(pb.is_none(), "Should not create progress bar for non-TTY");
}

#[test]
fn test_global_formatter_access() {
    let formatter = peter_hook::output::formatter();
    // Should be able to access global formatter
    let _ = formatter.status(true);
}

#[test]
fn test_hook_result_various_exit_codes() {
    let formatter = OutputFormatter::with_tty(false);

    let result0 = formatter.hook_result("hook", true, 0);
    assert!(result0.contains('0'));

    let result1 = formatter.hook_result("hook", false, 1);
    assert!(result1.contains('1'));

    let result127 = formatter.hook_result("hook", false, 127);
    assert!(result127.contains("127"));
}

#[test]
fn test_hook_result_special_characters_in_name() {
    let formatter = OutputFormatter::with_tty(false);
    let result = formatter.hook_result("hook-with-dashes_and_underscores", true, 0);
    assert!(result.contains("hook-with-dashes_and_underscores"));
}

#[test]
fn test_section_header_empty_string() {
    let formatter = OutputFormatter::with_tty(false);
    let result = formatter.section_header("");
    assert_eq!(result, "===  ===");
}

#[test]
fn test_section_header_long_title() {
    let formatter = OutputFormatter::with_tty(false);
    let long_title = "A very long section title that contains many words";
    let result = formatter.section_header(long_title);
    assert!(result.contains(long_title));
}

#[test]
fn test_divider_various_lengths() {
    let formatter = OutputFormatter::with_tty(true);

    let div1 = formatter.divider("A");
    assert!(div1.contains('A'));
    assert_eq!(div1, "A\n=");

    let div_long = formatter.divider("Long Title Here");
    assert!(div_long.contains("Long Title Here"));
    assert_eq!(div_long.matches('=').count(), "Long Title Here".len());
}
