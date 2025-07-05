// Import the standard library's error trait for use in returning error types.
use std::error::Error;

/// A trait defining a common interface for interacting with source control providers.
///
/// This trait abstracts operations that a source control provider (like GitHub, GitLab, Bitbucket)
/// should support for managing pull requests (PRs). It enables writing generic code that can
/// work with multiple providers through a single interface.
///
/// The trait's methods typically return `Result` types to handle success or error scenarios gracefully.
///
/// # Design notes:
/// - The trait uses `&self` (shared reference) implying these operations don't require mutable state,
///   or the implementation manages interior mutability internally.
/// - The error type is boxed `dyn Error` to allow flexibility in error handling and avoid
///   binding to a specific error enum, simplifying interoperability.
/// - The trait is focused on PR workflows: submitting reviews, listing PRs, closing PRs, and showing details.
pub trait SourceControlProvider {
    /// Submits a review on a pull request.
    ///
    /// # Parameters
    /// - `pr_number`: The pull request identifier as a string slice, typically a numeric ID.
    /// - `message`: The review message or comment body to submit.
    /// - `event`: The review event type, e.g., "APPROVE", "REQUEST_CHANGES", "COMMENT".
    ///
    /// # Returns
    /// - `Ok(())` if the review submission was successful.
    /// - `Err` containing a boxed error if something went wrong (network error, invalid PR, etc).
    ///
    /// # Usage
    /// This method encapsulates the entire review submission flow, including authentication,
    /// sending the review comment, and handling response errors.
    fn submit_review(
        &self,
        pr_number: &str,
        message: &str,
        event: &str,
    ) -> Result<(), Box<dyn Error>>;

    /// Lists all open pull requests for the current repository.
    ///
    /// # Returns
    /// - `Ok(())` on success (typically, this method would internally handle displaying
    ///   or returning the PR list; the signature can be adjusted for your use case).
    /// - `Err` on failure (e.g., API unreachable, authentication failed).
    ///
    /// # Notes
    /// This method abstracts the retrieval and possibly display of open PRs, hiding API details.
    fn list_pull_requests(&self) -> Result<(), Box<dyn Error>>;

    /// Closes the specified pull request.
    ///
    /// # Parameters
    /// - `pr_number`: The identifier of the PR to close.
    ///
    /// # Returns
    /// - `Ok(())` if the PR was closed successfully.
    /// - `Err` if closing the PR failed.
    ///
    /// # Context
    /// This can be used to implement rejecting a PR as part of a review workflow.
    fn close_pull_request(&self, pr_number: &str) -> Result<(), Box<dyn Error>>;

    /// Displays detailed information about a specific pull request.
    ///
    /// # Parameters
    /// - `pr_number`: The identifier of the PR to display.
    ///
    /// # Returns
    /// - `Ok(())` after successfully displaying the PR details.
    /// - `Err` if fetching or displaying details fails.
    ///
    /// # Usage
    /// Useful for showing metadata like PR title, author, status, commits, files changed, etc.
    fn show_pull_request_details(&self, pr_number: &str) -> Result<(), Box<dyn Error>>;
}
