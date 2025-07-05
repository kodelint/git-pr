// This module acts as the root module for defining and accessing source control providers.
// Currently, it only supports GitHub, but is designed to be extensible to support
// additional providers like GitLab, Bitbucket, etc., in the future.

// Import the `SourceControlProvider` trait which defines the behavior any source control
// provider must implement (e.g., listing PRs, submitting reviews, etc.)
use crate::providers::github::methods::SourceControlProvider;

// Import the concrete implementation of the provider for GitHub.
// `GitHubProvider` is a struct that implements the `SourceControlProvider` trait.
use crate::providers::github::models::GitHubProvider;

// The Error trait from Rust's standard library is required to support flexible error handling
// in the return types of provider factories and operations.
use std::error::Error;

// Re-export the GitHub provider module so other parts of the crate can access it.
// This allows submodules like `github::methods` and `github::models` to be accessed
// through the public `providers` namespace.
pub mod github;

/// Attempts to select and construct a source control provider based on the provided remote URL.
///
/// This function acts as a basic factory for determining which provider should be used
/// based on the `remote_url` string. In this case, it checks for the presence of
/// "github.com" and returns a GitHub provider.
///
/// # Arguments
///
/// * `remote_url` - The Git remote origin URL obtained via `git remote get-url origin`.
///   This string is used to determine which source control provider the project is using.
///
/// # Returns
///
/// * `Ok(Box<dyn SourceControlProvider>)` - If a compatible provider is found, it returns
///   a boxed trait object allowing dynamic dispatch of trait methods (e.g., listing PRs).
/// * `Err(...)` - If the remote URL does not match any known provider, an error is returned.
///
/// # Design Note
///
/// - The use of a boxed trait object (`Box<dyn SourceControlProvider>`) allows client code
///   to interact with the provider generically without needing to know its exact type.
/// - This approach simplifies adding new providers by updating only this function,
///   while the rest of the code remains unchanged.
///
/// # Example
///
/// ```rust
/// let remote_url = "https://github.com/user/repo.git";
/// let provider = get_provider(remote_url)?;
/// provider.list_pull_requests()?;
/// ```
pub fn get_provider(remote_url: &str) -> Result<Box<dyn SourceControlProvider>, Box<dyn Error>> {
    // Simple pattern match on the remote URL.
    // This check assumes that any GitHub remote will include "github.com" in the URL.
    // In the future, more sophisticated matching or parsing logic may be used
    // to support other providers like GitLab or Bitbucket.
    if remote_url.contains("github.com") {
        // Instantiate a new GitHub provider with the given URL.
        // `.new()` may return an error, so the `?` operator is used to propagate it.
        Ok(Box::new(GitHubProvider::new(remote_url.to_string())?))
    } else {
        // If the URL does not match any known provider, return a generic error.
        // `.into()` converts the &str into a boxed error.
        Err("Unsupported provider".into())
    }
}
