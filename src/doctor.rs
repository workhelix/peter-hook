//! Health check and diagnostics module.

use crate::{
    git::GitRepository,
    hooks::HookResolver,
    HookConfig,
};

/// Run doctor command to check health and configuration.
///
/// Returns exit code: 0 if healthy, 1 if issues found.
#[must_use]
pub fn run_doctor() -> i32 {
    println!("🏥 peter-hook health check");
    println!("==========================");
    println!();

    let mut has_errors = false;
    let mut has_warnings = false;

    check_git_repository(&mut has_errors, &mut has_warnings);
    println!();

    check_configuration(&mut has_errors, &mut has_warnings);
    println!();

    check_updates(&mut has_warnings);
    println!();

    // Summary
    if has_errors {
        println!("❌ Issues found - see above for details");
        1
    } else if has_warnings {
        println!("⚠️  Warnings found - configuration may need attention");
        0 // Warnings don't cause failure
    } else {
        println!("✨ Everything looks healthy!");
        0
    }
}

fn check_git_repository(has_errors: &mut bool, has_warnings: &mut bool) {
    println!("Git Repository:");
    match GitRepository::find_from_current_dir() {
        Ok(repo) => {
            println!("  ✅ Git repository found");

            // Check hooks
            match repo.list_hooks() {
                Ok(hooks) => {
                    if hooks.is_empty() {
                        println!("  ⚠️  No git hooks installed");
                        *has_warnings = true;
                    } else {
                        println!("  ✅ {} git hook(s) found", hooks.len());

                        // Check if managed by peter-hook
                        let mut managed_count = 0;
                        for hook_name in &hooks {
                            if let Ok(Some(info)) = repo.get_hook_info(hook_name) {
                                if info.is_managed {
                                    managed_count += 1;
                                }
                            }
                        }

                        if managed_count == 0 {
                            println!("  ⚠️  No hooks managed by peter-hook");
                            println!("  ℹ️  Run 'peter-hook install' to install hooks");
                            *has_warnings = true;
                        } else {
                            println!("  ✅ {managed_count} hook(s) managed by peter-hook");
                        }
                    }
                }
                Err(e) => {
                    println!("  ❌ Failed to list git hooks: {e}");
                    *has_errors = true;
                }
            }
        }
        Err(e) => {
            println!("  ❌ Not in a git repository: {e}");
            *has_errors = true;
        }
    }
}

fn check_configuration(has_errors: &mut bool, has_warnings: &mut bool) {
    println!("Configuration:");
    let resolver = HookResolver::new(std::env::current_dir().unwrap_or_default());

    match resolver.find_config_file() {
        Ok(Some(config_path)) => {
            println!("  ✅ Config file: {}", config_path.display());

            // Try to parse it
            match HookConfig::from_file(&config_path) {
                Ok(config) => {
                    println!("  ✅ Config is valid");

                    let hook_names = config.get_hook_names();
                    if hook_names.is_empty() {
                        println!("  ⚠️  No hooks or groups defined");
                        *has_warnings = true;
                    } else {
                        println!("  ✅ Found {} hook(s)/group(s)", hook_names.len());
                    }
                }
                Err(e) => {
                    println!("  ❌ Config is invalid: {e}");
                    *has_errors = true;
                }
            }
        }
        Ok(None) => {
            println!("  ⚠️  No hooks.toml file found");
            println!("  ℹ️  Create a hooks.toml file to configure peter-hook");
            *has_warnings = true;
        }
        Err(e) => {
            println!("  ❌ Failed to find config: {e}");
            *has_errors = true;
        }
    }
}

fn check_updates(has_warnings: &mut bool) {
    println!("Updates:");
    match check_for_updates() {
        Ok(Some(latest)) => {
            let current = env!("CARGO_PKG_VERSION");
            println!("  ⚠️  Update available: v{latest} (current: v{current})");
            println!("  💡 Run 'peter-hook update' to install the latest version");
            *has_warnings = true;
        }
        Ok(None) => {
            println!(
                "  ✅ Running latest version (v{})",
                env!("CARGO_PKG_VERSION")
            );
        }
        Err(e) => {
            println!("  ⚠️  Failed to check for updates: {e}");
            *has_warnings = true;
        }
    }
}

/// Check for available updates from GitHub releases.
///
/// Returns Ok(Some(version)) if update available, Ok(None) if up to date, or Err on network failure.
///
/// # Errors
///
/// Returns an error if the network request fails or the response cannot be parsed.
pub fn check_for_updates() -> Result<Option<String>, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("peter-hook-doctor")
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let url = "https://api.github.com/repos/workhelix/peter-hook/releases/latest";
    let response: serde_json::Value = client
        .get(url)
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let tag_name = response["tag_name"]
        .as_str()
        .ok_or_else(|| "No tag_name in response".to_string())?;

    let latest = tag_name
        .trim_start_matches("peter-hook-v")
        .trim_start_matches('v');
    let current = env!("CARGO_PKG_VERSION");

    if latest == current {
        Ok(None)
    } else {
        Ok(Some(latest.to_string()))
    }
}
