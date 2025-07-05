use std::env;

/// Checks if DEBUG mode is enabled via the `DEBUG` environment variable.
/// Accepts values: `1`, `true`, `TRUE`
pub fn is_debug_enabled() -> bool {
    matches!(
        env::var("DEBUG").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}
