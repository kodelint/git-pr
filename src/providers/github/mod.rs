use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;
use std::env;
use std::error::Error;

/// A trait that represents common behaviors for interacting with a source control provider.
pub trait SourceControlProvider {
    /// Submits a review for a given pull request number with a message.
    fn submit_review(&self, pr_number: &str, message: &str) -> Result<(), Box<dyn Error>>;

    /// Lists open pull requests for the current repository.
    fn list_pull_requests(&self) -> Result<(), Box<dyn Error>>;
}

/// A GitHub-specific implementation of the SourceControlProvider trait.
/// This struct holds state like the remote repo URL, an HTTP client, and the GitHub token.
pub struct GitHubProvider {
    pub remote_url: String,
    pub client: Client,
    pub token: String,
}

impl GitHubProvider {
    /// Creates a new GitHubProvider instance by reading the GitHub token from the environment.
    /// The token must be set in `GITHUB_TOKEN` for authentication with the GitHub API.
    pub fn new(remote_url: String) -> Result<Self, Box<dyn Error>> {
        let token = env::var("GITHUB_TOKEN")?;
        Ok(GitHubProvider {
            remote_url,
            client: Client::new(),
            token,
        })
    }

    /// Extracts the GitHub `owner` and `repo` name from the remote URL.
    /// Handles both HTTPS and SSH GitHub URLs.
    fn infer_repo_details(&self) -> Option<(String, String)> {
        let url = self.remote_url.trim_end_matches(".git");

        if url.contains("github.com") {
            let parts: Vec<&str> = if url.starts_with("http") {
                url.split('/').collect()
            } else {
                // For SSH URLs like git@github.com:owner/repo.git
                url.split(':').last()?.split('/').collect()
            };

            if parts.len() >= 2 {
                return Some((
                    parts[parts.len() - 2].to_string(),
                    parts[parts.len() - 1].to_string(),
                ));
            }
        }

        None
    }
}

impl SourceControlProvider for GitHubProvider {
    /// Submits an "APPROVE" review on a specific PR using GitHub's REST API.
    /// This method fetches the PR's current head commit SHA and includes it in the request.
    fn submit_review(&self, pr_number: &str, message: &str) -> Result<(), Box<dyn Error>> {
        // Parse owner/repo from the remote URL
        let (owner, repo) = self
            .infer_repo_details()
            .ok_or("Could not parse owner/repo")?;

        // Fetch the PR JSON to get the commit SHA required for the review
        let pr_url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo, pr_number
        );

        if is_debug_enabled() {
            eprintln!("🔍 [DEBUG] Fetching PR for commit_id from: {}", pr_url);
        }

        let pr_response = self
            .client
            .get(&pr_url)
            .bearer_auth(&self.token)
            .header("User-Agent", "git-pr")
            .send()?;

        let pr_json: serde_json::Value = pr_response.json()?;

        // Extract the head commit SHA
        let commit_id = pr_json["head"]["sha"]
            .as_str()
            .ok_or("❌ Could not extract commit_id from PR JSON")?;

        if is_debug_enabled() {
            eprintln!("🧪 [DEBUG] commit_id for PR #{}: {}", pr_number, commit_id);
        }

        // Build the review submission URL and request body
        let review_url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/reviews",
            owner, repo, pr_number
        );

        let body = json!({
            "body": message,
            "event": "APPROVE",
            "commit_id": commit_id
        });

        if is_debug_enabled() {
            eprintln!("🚀 [DEBUG] Submitting review to: {}", review_url);
            eprintln!("📦 [DEBUG] Payload: {}", body);
        }

        let response = self
            .client
            .post(&review_url)
            .bearer_auth(&self.token)
            .header("User-Agent", "git-pr")
            .json(&body)
            .send()?;

        let status = response.status();
        let text = response.text().unwrap_or_default();

        if is_debug_enabled() {
            eprintln!("📬 [DEBUG] Response status: {}", status);
            eprintln!("📨 [DEBUG] Response body: {}", text);
        }

        // Check success, otherwise print a helpful error message
        if status.is_success() {
            println!("✅ Review submitted successfully for PR #{}", pr_number);
        } else {
            // Parse JSON error response from GitHub if possible
            let message = match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(json) => {
                    let msg = json["message"].as_str().unwrap_or("Unknown error");
                    let details = json["errors"]
                        .as_array()
                        .map(|errs| {
                            errs.iter()
                                .filter_map(|e| e.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();

                    format!(
                        "{}{}",
                        msg,
                        if !details.is_empty() {
                            format!(": {}", details)
                        } else {
                            "".into()
                        }
                    )
                }
                Err(_) => text,
            };

            eprintln!("❌ Failed to submit review: {}", message);
        }

        Ok(())
    }

    /// Lists all open pull requests for the current repository using the GitHub API.
    fn list_pull_requests(&self) -> Result<(), Box<dyn Error>> {
        // Extract repo details from the remote URL
        let (owner, repo) = self
            .infer_repo_details()
            .ok_or("❌ Could not parse owner/repo from remote URL")?;
        let url = format!("https://api.github.com/repos/{}/{}/pulls", owner, repo);

        if is_debug_enabled() {
            eprintln!("📡 [DEBUG] GET {}", url);
        }

        // Send the GET request to GitHub API
        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .header("User-Agent", "git-pr")
            .send()?;

        let status = response.status();
        let body_text = response.text().unwrap_or_default();

        if is_debug_enabled() {
            eprintln!("📬 [DEBUG] Response status: {}", status);
            eprintln!("📨 [DEBUG] Response body: {}", body_text);
        }

        if status.is_success() {
            // Deserialize and display the PR list
            let prs: Vec<GitHubPR> = serde_json::from_str(&body_text)?;
            if prs.is_empty() {
                println!("ℹ️  No open pull requests found.");
            } else {
                println!("📋 Open Pull Requests:");
                for pr in prs {
                    println!("#{}: {}", pr.number, pr.title);
                }
            }
        } else {
            // Handle API errors gracefully and print helpful messages
            let message = match serde_json::from_str::<serde_json::Value>(&body_text) {
                Ok(json) => {
                    let msg = json["message"].as_str().unwrap_or("Unknown error");
                    let details = json["errors"]
                        .as_array()
                        .map(|errs| {
                            errs.iter()
                                .filter_map(|e| e.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();

                    format!(
                        "{}{}",
                        msg,
                        if !details.is_empty() {
                            format!(": {}", details)
                        } else {
                            "".into()
                        }
                    )
                }
                Err(_) => body_text,
            };

            eprintln!("❌ Failed to list PRs: {}", message);
        }

        Ok(())
    }
}

/// Deserialization struct for GitHub PR API response
#[derive(Deserialize)]
struct GitHubPR {
    number: u32,
    title: String,
}

/// Checks if DEBUG mode is enabled via the `DEBUG` environment variable.
/// Accepts values: `1`, `true`, `TRUE`
fn is_debug_enabled() -> bool {
    matches!(
        env::var("DEBUG").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}
