use crate::debug_log;
use crate::providers::github::methods::*;
use crate::providers::github::models::*;
use crate::utils::is_debug_enabled;
use chrono::{DateTime, Utc};
use colored::Colorize;
use owo_colors::OwoColorize;
use reqwest::blocking::Client;
use serde_json::json;
use std::env;
use std::error::Error;
use std::process::Command;
use tabled::{settings::Style, Table};
use textwrap::{fill, Options};

impl GitHubProvider {
    /// Creates a new GitHubProvider instance by reading the GitHub token from the environment.
    /// The token must be set in `GITHUB_TOKEN` for authentication with the GitHub API.
    pub fn new(remote_url: String) -> Result<Self, Box<dyn Error>> {
        debug_log!("[DEBUG] Creating GitHubProvider instance");
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
        debug_log!("[DEBUG] Inferring repo details from remote URL");
        let url = self.remote_url.trim_end_matches(".git");

        if url.contains("github.com") {
            let parts: Vec<&str> = if url.starts_with("http") {
                url.split('/').collect()
            } else {
                url.split(':').last()?.split('/').collect()
            };

            debug_log!("[DEBUG] Split URL parts: {:?}", parts);

            if parts.len() >= 2 {
                return Some((
                    parts[parts.len() - 2].to_string(),
                    parts[parts.len() - 1].to_string(),
                ));
            }
        }
        debug_log!("[DEBUG] Failed to infer repo details");
        None
    }
}

impl SourceControlProvider for GitHubProvider {
    /// Submits a "REVIEW*" review on a specific PR using GitHub's REST API.
    /// REVIEW:
    /// - APPROVE `--approve`
    /// - REJECT `--reject`
    /// - COMMENT-ONLY `--comment-only`
    /// This method fetches the PR's current head commit SHA and includes it in the request.
    fn submit_review(
        &self,
        pr_number: &str,
        message: &str,
        event: &str,
    ) -> Result<(), Box<dyn Error>> {
        debug_log!("[DEBUG] Submitting review for PR #{}", pr_number);
        // Parse owner/repo from the remote URL
        let (owner, repo) = self
            .infer_repo_details()
            .ok_or("Could not parse owner/repo")?;
        // Fetch the PR JSON to get the commit SHA required for the review
        let pr_url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo, pr_number
        );

        debug_log!("[DEBUG] Fetching PR for commit_id from: {}", pr_url);

        let pr_response = self
            .client
            .get(&pr_url)
            .bearer_auth(&self.token)
            .header("User-Agent", "git-pr")
            .send()?;

        let pr_json: serde_json::Value = pr_response.json()?;

        let commit_id = pr_json["head"]["sha"]
            .as_str()
            .ok_or("Could not extract commit_id")?;

        debug_log!("[DEBUG] commit_id for PR #{}: {}", pr_number, commit_id);

        let review_url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/reviews",
            owner, repo, pr_number
        );

        let body = json!({
            "body": message,
            "event": event,
            "commit_id": commit_id
        });

        debug_log!("[DEBUG] Submitting review to: {}", review_url);
        debug_log!("[DEBUG] Payload: {}", body);

        let response = self
            .client
            .post(&review_url)
            .bearer_auth(&self.token)
            .header("User-Agent", "git-pr")
            .json(&body)
            .send()?;

        debug_log!("[DEBUG] Response status: {}", response.status());

        if response.status().is_success() {
            println!("✅ Review submitted successfully for PR #{}", pr_number);
            Ok(())
        } else {
            Err(format!("Failed to submit review: {}", response.text()?).into())
        }
    }

    /// Lists all open pull requests for the current repository.
    /// This function:
    /// - Parses the remote to determine the owner/repo
    /// - Fetches open PRs from the GitHub API
    /// - For each PR, fetches detailed info like commits, labels, etc.
    /// - Displays the data in a well-formatted table using `tabled`
    fn list_pull_requests(&self) -> Result<(), Box<dyn Error>> {
        debug_log!("[DEBUG] Listing pull requests");
        // Infer owner and repo from git remote. This returns (user, repo_name)
        let (owner, repo) = self
            .infer_repo_details()
            .ok_or("Could not parse owner/repo")?;

        // Construct the API endpoint to list open PRs (up to 50)
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls?state=open&per_page=50",
            owner, repo
        );

        debug_log!("[DEBUG] Fetching PRs from URL: {}", url);

        // Make the HTTP GET request to fetch the list of PRs
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.token) // Authenticate with GitHub token
            .header("User-Agent", "git-pr") // Required GitHub header
            .send()?; // Execute the request

        // Extract the HTTP status code and raw response body
        let status = resp.status();
        let text = resp.text()?;

        // If DEBUG is enabled, print status and body for inspection
        debug_log!("[DEBUG] Response status: {}", status);
        debug_log!("[DEBUG] Response body: {}", text);

        // If GitHub returned a non-200 response, treat as an error
        if !status.is_success() {
            return Err(format!("Failed to list PRs: {}", text).into());
        }

        // Deserialize the basic PR list into a lightweight struct
        // This does NOT include fields like commits or file count
        let basic_prs: Vec<BasicGitHubPR> = serde_json::from_str(&text)?;

        // Early exit if no PRs found
        if basic_prs.is_empty() {
            println!("ℹ️  No open pull requests found.");
            return Ok(());
        }

        debug_log!("[DEBUG] {} PRs found", basic_prs.len());

        // We'll store (GitHubPR, age_days) so we can sort later
        let mut detailed_prs = Vec::new();

        // Loop through each basic PR and fetch its full details
        for basic_pr in basic_prs {
            // Fetching PR details in DEBUG
            debug_log!("[DEBUG] Fetching details for PR #{}", basic_pr.number);

            let detail_url = format!(
                "https://api.github.com/repos/{}/{}/pulls/{}",
                owner, repo, basic_pr.number
            );

            let detail_resp = self
                .client
                .get(&detail_url)
                .bearer_auth(&self.token)
                .header("User-Agent", "git-pr")
                .send()?;

            let detail_status = detail_resp.status();
            let detail_text = detail_resp.text()?; // Will be parsed as JSON

            if !detail_status.is_success() {
                eprintln!(
                    "⚠️  Failed to fetch details for PR #{}: {}",
                    basic_pr.number, detail_text
                );
                continue;
            }

            let pr: GitHubPR = serde_json::from_str(&detail_text)?;
            let age_days = (Utc::now() - pr.created_at).num_days();

            // Store PR with age_days for later sorting
            detailed_prs.push((pr, age_days));
        }

        // Sort PRs by age_days ASCENDING (oldest first). Use `rev()` to make it newest first.
        detailed_prs.sort_by_key(|(_, age_days)| *age_days);

        debug_log!("[DEBUG] Sorted PRs by age");

        // Build table rows after sorting
        let display_rows: Vec<DisplayPR> = detailed_prs
            .into_iter()
            .map(|(pr, age_days)| {
                debug_log!("[DEBUG] Mapping PR #{} to table row", pr.number);
                let age = if age_days == 0 {
                    "today".to_string()
                } else {
                    format!("{}d", age_days)
                };

                let labels = if pr.labels.is_empty() {
                    "-".to_string()
                } else {
                    pr.labels
                        .iter()
                        .map(|l| l.name.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                let description_raw = pr.body.as_deref().unwrap_or("-");
                let wrap_opts = Options::new(60).break_words(false);
                let description_wrapped = fill(description_raw, wrap_opts);

                DisplayPR {
                    number: format!("#{}", pr.number),
                    title: pr.title.clone(),
                    author: pr.user.login.clone(),
                    age,
                    commits: pr.commits.to_string(),
                    files: pr.changed_files.to_string(),
                    labels,
                    description: description_wrapped,
                }
            })
            .collect();

        // Create and print the final table
        let mut table = Table::new(display_rows);
        table.with(Style::rounded());
        println!("{table}");

        Ok(())
    }

    fn close_pull_request(&self, pr_number: &str) -> Result<(), Box<dyn Error>> {
        debug_log!("[DEBUG] Closing PR #{}", pr_number);
        // Extract repo details from the remote URL
        let (owner, repo) = self
            .infer_repo_details()
            .ok_or("Could not parse owner/repo")?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo, pr_number
        );

        let body = json!({ "state": "closed" });

        debug_log!("📬 [DEBUG] Request Sent: {} to URL: {}", body, url);

        let response = self
            .client
            .patch(&url)
            .bearer_auth(&self.token)
            .header("User-Agent", "git-pr")
            .json(&body)
            .send()?;

        debug_log!(
            "📬 [DEBUG] Response Received: {} from URL: {}",
            response.status(),
            url
        );

        if response.status().is_success() {
            println!("✅ Successfully closed PR #{}", pr_number);
            Ok(())
        } else {
            Err(format!("Failed to close PR: {}", response.text()?).into())
        }
    }

    fn show_pull_request_details(&self, pr_number: &str) -> Result<(), Box<dyn std::error::Error>> {
        debug_log!("[DEBUG] Showing Details for PR #{}", pr_number);
        let (owner, repo) = self
            .infer_repo_details()
            .ok_or("Could not parse owner/repo")?;

        // Fetch PR metadata
        let pr_url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo, pr_number
        );

        debug_log!("[DEBUG] Fetching PR metadata from: {}", pr_url);

        let pr_resp = self
            .client
            .get(&pr_url)
            .bearer_auth(&self.token)
            .header("User-Agent", "git-pr")
            .send()?;

        if !pr_resp.status().is_success() {
            return Err(format!("Failed to fetch PR details: {}", pr_resp.text()?).into());
        }

        let pr_json: serde_json::Value = pr_resp.json()?;
        let title = pr_json["title"].as_str().unwrap_or("-");
        let status = pr_json["state"].as_str().unwrap_or("-");
        let user = pr_json["user"]["login"].as_str().unwrap_or("-");
        let created_at = pr_json["created_at"].as_str().unwrap_or("-");
        let created_date = DateTime::parse_from_rfc3339(created_at)?.with_timezone(&Utc);
        let age_days = (Utc::now() - created_date).num_days();
        let age = if age_days == 0 {
            "today".to_string()
        } else {
            format!("{}d", age_days)
        };

        debug_log!(
            "[DEBUG] PR #{}: title={}, status={}, author={}, age={}d",
            pr_number,
            title,
            status,
            user,
            age_days
        );

        // Fetch commits
        let commits_url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/commits",
            owner, repo, pr_number
        );
        let commits_resp = self
            .client
            .get(&commits_url)
            .bearer_auth(&self.token)
            .header("User-Agent", "git-pr")
            .send()?;

        if !commits_resp.status().is_success() {
            return Err(format!("Failed to fetch commits: {}", commits_resp.text()?).into());
        }

        let commits: Vec<serde_json::Value> = commits_resp.json()?;

        let mut rows = Vec::new();

        // First row: Full PR metadata, plus first commit + files
        for (i, commit) in commits.iter().enumerate() {
            let sha = commit["sha"].as_str().unwrap_or("-");
            let short_sha = &sha[..7.min(sha.len())];

            // Fetch files for the commit
            let commit_url = format!(
                "https://api.github.com/repos/{}/{}/commits/{}",
                owner, repo, sha
            );

            debug_log!("[DEBUG] Fetching files for commit {}", short_sha);

            let commit_resp = self
                .client
                .get(&commit_url)
                .bearer_auth(&self.token)
                .header("User-Agent", "git-pr")
                .send()?;

            if !commit_resp.status().is_success() {
                eprintln!(
                    "⚠️  Failed to fetch commit {}: {}",
                    sha,
                    commit_resp.text()?
                );
                continue;
            }

            let commit_json: serde_json::Value = commit_resp.json()?;
            let files = commit_json["files"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|f| f["filename"].as_str())
                .collect::<Vec<_>>()
                .join(", ");

            let row = PRDetailsRow {
                pr_number: if i == 0 {
                    format!("#{}", pr_number)
                } else {
                    "".to_string()
                },
                title: if i == 0 {
                    title.to_string()
                } else {
                    "".to_string()
                },
                status: if i == 0 {
                    status.to_string()
                } else {
                    "".to_string()
                },
                age: if i == 0 { age.clone() } else { "".to_string() },
                github_username: if i == 0 {
                    user.to_string()
                } else {
                    "".to_string()
                },
                commit_sha: short_sha.to_string(),
                changed_files: files,
            };

            rows.push(row);
        }

        let mut table = Table::new(rows);
        table.with(Style::rounded());
        println!("{table}");
        Ok(())
    }
}

pub fn get_remote_url() -> Option<String> {
    if is_debug_enabled() {
        eprintln!("🐙 [DEBUG] Getting remote origin URL...");
    }

    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .expect("Failed to get remote URL");

    debug_log!(
        "📤 [DEBUG] Raw output: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    if output.status.success() {
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if is_debug_enabled() {
            eprintln!("✅ [DEBUG] Remote URL: {}", url);
        }
        Some(url)
    } else {
        if is_debug_enabled() {
            eprintln!(
                "❌ [DEBUG] Failed to get remote URL (exit code: {})",
                output.status
            );
        }
        None
    }
}

/// Pulls a PR from the remote and checks out a local tracking branch.
///
/// Uses the format `pull/{pr_number}/head:pr-request-{pr_number}`
/// to create a local branch for inspection or testing.
///
pub fn pull_pr(pr_number: &str) {
    let fetch_ref = format!("pull/{}/head:pr-request-{}", pr_number, pr_number);

    debug_log!("📡 [DEBUG] Fetching ref: {}", fetch_ref);

    let fetch = Command::new("git")
        .args(["fetch", "origin", &fetch_ref])
        .status()
        .expect("Failed to fetch PR");

    if fetch.success() {
        debug_log!(
            "[DEBUG] Fetch succeeded, checking out pr-request-{}",
            pr_number
        );
        let checkout = Command::new("git")
            .args(["checkout", &format!("pr-request-{}", pr_number)])
            .status()
            .expect("Failed to checkout PR branch");

        if checkout.success() {
            println!(
                "{}",
                format!("✅ Switched to branch pr-request-{}", pr_number).green()
            );
        } else {
            eprintln!("{}", "❌ Failed to checkout PR branch.".red());
        }
    } else {
        eprintln!("{}", "❌ Failed to fetch PR.".red());
    }
}

/// Displays a diff of the PR branch vs `origin/main`.
/// Assumes the PR has already been fetched and checked out via `pull_pr()`.
///
pub fn show_diff(pr_number: &str) {
    let branch = format!("pr-request-{}", pr_number);
    let diff_range = format!("origin/main...{}", branch);

    debug_log!("[DEBUG] Running: git diff {}", diff_range);

    let diff = Command::new("git")
        .args(["diff", &diff_range])
        .status()
        .expect("Failed to show diff");

    debug_log!("[DEBUG] Preparing Git Diff {}", diff_range);

    if !diff.success() {
        eprintln!("{}", "❌ Failed to display diff.".red());
    }
}
