//! Self-update module.

use sha2::{Digest, Sha256};
use std::path::Path;

/// Run update command to install latest or specified version.
///
/// Returns exit code: 0 if successful, 1 on error, 2 if already up-to-date.
///
/// # Panics
///
/// May panic if stdout flush fails or stdin read fails during user confirmation prompt.
#[must_use]
#[allow(clippy::unused_async)]
pub fn run_update(version: Option<&str>, force: bool, install_dir: Option<&Path>) -> i32 {
    let current_version = env!("CARGO_PKG_VERSION");

    println!("🔄 Checking for updates...");

    // Get target version
    let target_version = if let Some(v) = version {
        v.to_string()
    } else {
        match get_latest_version() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("❌ Failed to check for updates: {e}");
                return 1;
            }
        }
    };

    // Check if already up-to-date
    if target_version == current_version && !force {
        println!("✅ Already running latest version (v{current_version})");
        return 2;
    }

    println!("✨ Update available: v{target_version} (current: v{current_version})");

    // Detect current binary location
    let install_path = if let Some(dir) = install_dir {
        dir.join("peter-hook")
    } else {
        match std::env::current_exe() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("❌ Failed to determine binary location: {e}");
                return 1;
            }
        }
    };

    println!("📍 Install location: {}", install_path.display());
    println!();

    // Confirm unless forced
    if !force {
        use std::io::{self, Write};
        print!("Continue with update? [y/N]: ");
        io::stdout().flush().unwrap();

        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();

        if !matches!(response.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Update cancelled.");
            return 0;
        }
    }

    // Perform update
    match perform_update(&target_version, &install_path) {
        Ok(()) => {
            println!("✅ Successfully updated to v{target_version}");
            println!();
            println!("Run 'peter-hook --version' to verify the installation.");
            0
        }
        Err(e) => {
            eprintln!("❌ Update failed: {e}");
            1
        }
    }
}

/// Get the latest version from GitHub releases.
///
/// Returns the version string (without 'v' prefix) or an error if the network request fails.
///
/// # Errors
///
/// Returns an error if the HTTP request fails, the response cannot be parsed as JSON,
/// or the `tag_name` field is missing from the response.
pub fn get_latest_version() -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("peter-hook-updater")
        .timeout(std::time::Duration::from_secs(10))
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

    let version = tag_name
        .trim_start_matches("peter-hook-v")
        .trim_start_matches('v');
    Ok(version.to_string())
}

fn perform_update(version: &str, install_path: &Path) -> Result<(), String> {
    // Detect platform
    let platform = get_platform_string();
    let archive_ext = if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    };

    let filename = format!("peter-hook-{platform}.{archive_ext}");
    let download_url = format!(
        "https://github.com/workhelix/peter-hook/releases/download/peter-hook-v{version}/{filename}"
    );

    println!("📥 Downloading {filename}...");

    // Download file
    let client = reqwest::blocking::Client::builder()
        .user_agent("peter-hook-updater")
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&download_url)
        .send()
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }

    let bytes = response.bytes().map_err(|e| e.to_string())?;

    // Download checksum
    let checksum_url = format!("{download_url}.sha256");
    let checksum_response = client
        .get(&checksum_url)
        .send()
        .map_err(|e| e.to_string())?;

    if checksum_response.status().is_success() {
        println!("🔐 Verifying checksum...");
        let expected_checksum = checksum_response.text().map_err(|e| e.to_string())?;
        let expected_hash = expected_checksum
            .split_whitespace()
            .next()
            .ok_or_else(|| "Invalid checksum format".to_string())?;

        // Calculate actual checksum
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let actual_hash = hex::encode(hasher.finalize());

        if actual_hash != expected_hash {
            return Err(format!(
                "Checksum verification failed!\nExpected: {expected_hash}\nActual:   {actual_hash}"
            ));
        }

        println!("✅ Checksum verified");
    } else {
        eprintln!("⚠️  Checksum file not available, skipping verification");
    }

    // Extract and install
    println!("📦 Installing...");

    // Create temp directory
    let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;

    // Extract archive
    if cfg!(target_os = "windows") {
        // Extract zip (would need zip crate)
        return Err("Windows update not yet implemented".to_string());
    }
    // Extract tar.gz
    let tar_gz = flate2::read::GzDecoder::new(&bytes[..]);
    let mut archive = tar::Archive::new(tar_gz);
    archive.unpack(temp_dir.path()).map_err(|e| e.to_string())?;

    // Find binary in temp dir
    let binary_name = if cfg!(target_os = "windows") {
        "peter-hook.exe"
    } else {
        "peter-hook"
    };

    let temp_binary = temp_dir.path().join(binary_name);
    if !temp_binary.exists() {
        return Err(format!("Binary not found in archive: {binary_name}"));
    }

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&temp_binary)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&temp_binary, perms).map_err(|e| e.to_string())?;
    }

    // Replace binary
    std::fs::copy(&temp_binary, install_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            format!(
                "Permission denied. Try running with sudo or use --install-dir to specify a \
                 writable location:\n  {e}"
            )
        } else {
            e.to_string()
        }
    })?;

    Ok(())
}

/// Get the platform string for the current OS and architecture.
///
/// Returns a target triple string like "x86_64-apple-darwin" or "aarch64-unknown-linux-gnu".
#[must_use]
pub fn get_platform_string() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        _ => "unknown",
    }
}
