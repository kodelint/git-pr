// Bring the `env` module from the Rust standard library into scope.
// This module provides functions for accessing environment variables.
use std::env;
// `Command` allows us to spawn system processes like invoking `git`.
use std::process::Command;

/// Determines whether debug logging or verbose diagnostics should be enabled,
/// based on the presence and value of the `DEBUG` environment variable.
///
/// This function is a lightweight toggle mechanism that allows you to
/// enable additional logging or diagnostics in your application without changing code.
/// It's particularly useful for CLI tools, CI environments, or debugging production issues.
///
/// # Behavior:
/// - Reads the environment variable `DEBUG`
/// - Accepts the values "1", "true", or "TRUE" to enable debug mode
/// - Any other value, or if the variable is unset, disables debug mode
///
/// # Example:
/// ```bash
/// DEBUG=1 cargo run      # Debug mode ON
/// DEBUG=true cargo run   # Debug mode ON
/// DEBUG=0 cargo run      # Debug mode OFF
/// unset DEBUG && cargo run  # Debug mode OFF
/// ```
///
/// # Returns
/// `true` if debug mode is enabled, `false` otherwise.
pub fn is_debug_enabled() -> bool {
    // Attempt to read the value of the `DEBUG` environment variable.
    // `env::var()` returns a `Result<String, VarError>`.
    // `.as_deref()` converts `Result<String, _>` to `Result<&str, _>` for easier matching.
    matches!(
        env::var("DEBUG").as_deref(),
        // Match against known truthy values.
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

/// A macro for printing debug logs only when debugging is enabled via `DEBUG` env var.
///
/// It wraps `eprintln!` so logs go to stderr, and will only emit output if `is_debug_enabled()` is `true`.
/// This allows lightweight insertion of debug logs without affecting performance in production.
///
/// # Usage:
/// ```rust
/// debug_log!("Loading config for user: {}", user_id);
/// ```
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if $crate::utils::is_debug_enabled() {
            eprintln!($($arg)*);
        }
    };
}

/// Attempts to retrieve the `origin` remote URL from the local Git repository.
///
/// This function invokes the shell command `git remote get-url origin` and parses the output.
/// It can be used to determine where the repo was cloned from, useful for identifying provider (e.g., GitHub).
///
/// # Returns:
/// - `Some(String)` containing the remote URL if successful.
/// - `None` if Git fails or the command exits with a non-zero code.
pub fn get_remote_url() -> Option<String> {
    // Emit a debug message before executing the Git command, if debugging is enabled.
    debug_log!("[DEBUG] Getting remote origin URL...");

    // Use `git remote get-url origin` to retrieve the remote origin.
    // This is the canonical way to get the upstream URL in Git.
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .expect("Failed to get remote URL"); // Panic if the command itself fails to launch

    // Log raw output of Git command if debugging.
    debug_log!(
        "[DEBUG] Raw output: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    if output.status.success() {
        // Convert raw bytes to UTF-8 string, trim whitespace, and return.
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        debug_log!("[DEBUG] Remote URL: {}", url);
        Some(url)
    } else {
        // If Git command failed, optionally print an error if debugging is enabled.
        debug_log!(
            "[DEBUG] Failed to get remote URL (exit code: {})",
            output.status
        );
        None
    }
}
