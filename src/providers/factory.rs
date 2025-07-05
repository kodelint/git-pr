// Import the GitHub-specific provider and the generic SourceControlProvider trait.
// This allows us to use GitHubProvider as a polymorphic implementation of the trait.
use crate::providers::github::{GitHubProvider, SourceControlProvider};
use std::error::Error;

/// Returns a boxed trait object implementing `SourceControlProvider` based on the remote URL.
///
/// This function acts as a simple factory to choose the appropriate source control backend
/// (currently only GitHub is supported).
///
/// # Arguments
///
/// * `remote_url` - The Git remote origin URL (e.g., from `git remote get-url origin`)
///
/// # Returns
///
/// * `Ok(Box<dyn SourceControlProvider>)` if a provider is found and initialized successfully.
/// * `Err(...)` if no supported provider matches the URL.
///
/// # Example
///
/// ```
/// let provider = get_provider("https://github.com/owner/repo.git")?;
/// provider.list_pull_requests()?;
/// ``
pub fn get_provider(remote_url: &str) -> Result<Box<dyn SourceControlProvider>, Box<dyn Error>> {
    // Currently, only GitHub is supported.
    // Check if the URL contains "github.com" as a simple heuristic.
    if remote_url.contains("github.com") {
        // Return it as a boxed trait object so it can be used generically.
        Ok(Box::new(GitHubProvider::new(remote_url.to_string())?))
    } else {
        // Return an error for any unsupported remote host.
        Err("Unsupported provider".into())
    }
}
