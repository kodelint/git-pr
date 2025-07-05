// Bring the `env` module from the Rust standard library into scope.
// This module provides functions for accessing environment variables.
use std::env;

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
/// # Example
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
    //
    // We use `.as_deref()` to convert `Result<String, _>` to `Result<&str, _>`
    // which lets us match on the string contents directly without allocating.
    matches!(
        env::var("DEBUG").as_deref(),
        // Match against a few common string values considered to mean "true".
        // This is case-sensitive for "true", but includes uppercase variant as well.
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if $crate::utils::is_debug_enabled() {
            eprintln!($($arg)*);
        }
    };
}
