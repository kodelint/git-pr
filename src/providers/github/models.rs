use chrono::{DateTime, Utc};
// For handling date/time, specifically with UTC timezone
use reqwest::blocking::Client;
// HTTP client to make synchronous requests (blocking)
use serde::Deserialize;
// For deserializing JSON responses into Rust structs
use tabled::Tabled;
// Derive macro to allow easy table formatting for display

/// The core struct representing the GitHub provider implementation.
///
/// Holds key information needed to interact with GitHub's API:
/// - `remote_url`: The GitHub repository remote URL (e.g., https://github.com/user/repo.git)
/// - `client`: HTTP client instance to perform API requests
/// - `token`: Authentication token for GitHub API access (typically a personal access token)
///
/// All fields are `pub(crate)` to restrict direct access outside the current crate,
/// providing encapsulation while allowing internal use.
pub struct GitHubProvider {
    pub(crate) remote_url: String,
    pub(crate) client: Client,
    pub(crate) token: String,
}

/// Struct representing a full GitHub Pull Request response from the API.
///
/// Used to deserialize detailed PR data returned by GitHub's REST API.
/// Contains important PR metadata needed to display or manipulate PRs.
///
/// Fields:
/// - `number`: PR number identifier
/// - `title`: The title/summary of the PR
/// - `user`: The user who created the PR (nested struct)
/// - `created_at`: The creation date/time of the PR in UTC
/// - `body`: Optional detailed description of the PR
/// - `labels`: Labels/tags attached to the PR (e.g. "bug", "enhancement")
/// - `commits`: Number of commits in the PR
/// - `changed_files`: Number of files changed by the PR
#[derive(Deserialize)]
pub(crate) struct GitHubPR {
    pub number: u32,
    pub title: String,
    pub user: GitHubUser,
    pub created_at: DateTime<Utc>,
    pub body: Option<String>,
    pub labels: Vec<Label>,
    pub commits: u32,
    pub changed_files: u32,
}

/// A simplified GitHub PR struct used for lightweight API calls,
/// containing only basic metadata.
///
/// Useful for initial listing of PRs where you don't need full details.
/// This helps minimize bandwidth and processing time.
///
/// Fields:
/// - `number`: PR number
/// - `title`: PR title
/// - `user`: PR author info
/// - `created_at`: PR creation timestamp
#[allow(dead_code)]
#[derive(Deserialize)]
pub(crate) struct BasicGitHubPR {
    pub number: u32,
    pub title: String,
    pub user: GitHubUser,
    pub created_at: DateTime<Utc>,
}

/// Represents a GitHub user (author of PR, commenter, etc.)
///
/// Contains the login/username of the user.
///
/// This struct is nested inside other structs for deserialization.
#[derive(Deserialize)]
pub(crate) struct GitHubUser {
    pub login: String,
}

/// Represents a label assigned to a GitHub PR.
///
/// Labels are tags like "bug", "feature", or "urgent".
///
/// This struct is used within the GitHubPR struct.
#[derive(Deserialize)]
pub(crate) struct Label {
    pub name: String,
}

/// A display-friendly struct for summarizing PR info in tables.
///
/// Uses the `Tabled` derive macro for easy conversion into formatted tables.
/// This struct is NOT for deserialization, but for showing info in CLI output.
///
/// Fields are all strings, since they're formatted for display.
///
/// Fields and their table header names:
/// - `number`: PR number (e.g. "#123")
/// - `title`: Title of the PR
/// - `author`: Author username
/// - `age`: Age of PR (e.g. "3d" or "today")
/// - `commits`: Total number of commits as string
/// - `files`: Number of changed files as string
/// - `labels`: Comma-separated list of label names
/// - `description`: Wrapped PR description text
#[derive(Tabled)]
pub(crate) struct DisplayPR {
    #[tabled(rename = "Number")]
    pub number: String,
    #[tabled(rename = "Title")]
    pub title: String,
    #[tabled(rename = "Author")]
    pub author: String,
    #[tabled(rename = "Age")]
    pub age: String,
    #[tabled(rename = "Total Commits")]
    pub commits: String,
    #[tabled(rename = "Number of Changed Files")]
    pub files: String,
    #[tabled(rename = "Labels")]
    pub labels: String,
    #[tabled(rename = "Description")]
    pub description: String,
}

/// Represents a detailed row of PR information for displaying commit-level details.
///
/// Used when showing a PR with its commits and changed files, usually in a CLI table.
///
/// Fields include:
/// - `pr_number`: PR number, shown only in the first row for visual grouping
/// - `title`: PR title, shown only in first row
/// - `status`: PR state (open/closed), first row only
/// - `age`: PR age (days), first row only
/// - `github_username`: PR author, first row only
/// - `commit_sha`: Short SHA of the commit for the row
/// - `changed_files`: Files changed in this commit
#[derive(Tabled)]
pub(crate) struct PRDetailsRow {
    #[tabled(rename = "PR Number")]
    pub pr_number: String,
    #[tabled(rename = "Title")]
    pub title: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Age")]
    pub age: String,
    #[tabled(rename = "Authors")]
    pub github_username: String,
    #[tabled(rename = "Commit SHA")]
    pub commit_sha: String,
    #[tabled(rename = "Changed Files")]
    pub changed_files: String,
}
