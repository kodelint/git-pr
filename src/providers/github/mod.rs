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
