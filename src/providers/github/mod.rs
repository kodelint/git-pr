// Declare the `github` module as public so it can be accessed from outside this module.
// This typically contains core GitHub-related functionality such as the main provider implementation,
// API interaction methods, or high-level orchestration code.
pub mod github;

// Declare the `methods` module with `pub(crate)` visibility.
// This means the module is public *within the current crate only* but not exposed outside.
// Usually used for helper functions or internal logic that should not be part of the public API.
// Keeping this restricted helps encapsulate implementation details and maintain a clean API surface.
pub(crate) mod methods;

// Declare the `models` module as public.
// This module likely contains data structures, such as API response models, domain models, and data transfer objects (DTOs).
// Making it public means these models can be used throughout the crate and by external users if the crate is published.
pub mod models;

// Re-export selected items from the `github` module at this level.
// This makes `get_remote_url`, `pull_pr`, and `show_diff` available directly from this module,
// allowing users to call these functions without explicitly referencing `github::`.
// This technique improves ergonomics and provides a curated public API.
//
// For example:
// Instead of `crate::providers::github::get_remote_url()`,
// users can simply write `crate::providers::get_remote_url()`
//
// This helps organize code so the internal module structure can change without affecting external users.
pub use self::github::{get_remote_url, pull_pr, show_diff};
