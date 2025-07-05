use std::error::Error;

/// A trait that represents common behaviors for interacting with a source control provider.
pub trait SourceControlProvider {
    /// Submits a review for a given pull request number with a message.
    fn submit_review(
        &self,
        pr_number: &str,
        message: &str,
        event: &str,
    ) -> Result<(), Box<dyn Error>>;

    /// Lists open pull requests for the current repository.
    fn list_pull_requests(&self) -> Result<(), Box<dyn Error>>;

    /// Close the PR if `--reject` is used to with `submit-review`
    fn close_pull_request(&self, pr_number: &str) -> Result<(), Box<dyn Error>>;
}
