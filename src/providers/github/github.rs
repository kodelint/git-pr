use crate::debug_log;
use crate::providers::github::methods::*;
use crate::providers::github::models::*;
use crate::utils::get_remote_url;
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
    /// Submits a code review for a specific pull request on GitHub.
    ///
    /// This function supports submitting one of three review types:
    /// - APPROVE: Approves the changes (`--approve` flag)
    /// - REQUEST_CHANGES: Rejects the changes and asks for changes (`--reject` flag)
    /// - COMMENT: Only adds a comment without approving or rejecting (`--comment-only` flag)
    ///
    /// The method uses GitHub's REST API and requires the head commit SHA of the PR,
    /// which must be included in the review payload.
    fn submit_review(
        &self,
        pr_number: &str, // The pull request number, as a string (e.g. "42")
        message: &str,   // The review message to be attached to the review
        event: &str,     // The type of review: APPROVE, REQUEST_CHANGES, or COMMENT
    ) -> Result<(), Box<dyn Error>> {
        // Log debug message that a review is being initiated
        debug_log!("[DEBUG] Submitting review for PR #{}", pr_number);

        // Infer the repository owner and name from the remote URL
        let (owner, repo) = self
            .infer_repo_details()
            .ok_or("Could not parse owner/repo")?; // Return error if parsing fails

        // Build the URL to fetch the pull request details (needed to get the commit SHA)
        let pr_url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo, pr_number
        );

        debug_log!("[DEBUG] Fetching PR for commit_id from: {}", pr_url);

        // Make a GET request to fetch the PR data
        let pr_response = self
            .client
            .get(&pr_url)
            .bearer_auth(&self.token) // Use GitHub token for authentication
            .header("User-Agent", "git-pr") // Required by GitHub's API
            .send()?; // Send request and propagate errors

        // Parse the response body as JSON
        let pr_json: serde_json::Value = pr_response.json()?;

        // Extract the head commit SHA from the PR JSON
        let commit_id = pr_json["head"]["sha"]
            .as_str()
            .ok_or("Could not extract commit_id")?; // Error if SHA is missing

        debug_log!("[DEBUG] commit_id for PR #{}: {}", pr_number, commit_id);

        // Construct the URL to submit the review to GitHub's review API
        let review_url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/reviews",
            owner, repo, pr_number
        );

        // Create the JSON payload for the review submission
        let body = json!({
            "body": message,       // The review message text
            "event": event,       // Review type (APPROVE, REQUEST_CHANGES, COMMENT)
            "commit_id": commit_id // Required commit SHA for the review
        });

        debug_log!("[DEBUG] Submitting review to: {}", review_url);
        debug_log!("[DEBUG] Payload: {}", body);

        // Send the POST request to submit the review
        let response = self
            .client
            .post(&review_url)
            .bearer_auth(&self.token) // Again use the GitHub token
            .header("User-Agent", "git-pr") // Required user-agent
            .json(&body) // Attach the JSON payload
            .send()?; // Send and propagate any errors

        // Log the HTTP status for debug
        debug_log!("[DEBUG] Response status: {}", response.status());

        // Check if the submission was successful
        if response.status().is_success() {
            println!("✅ Review submitted successfully for PR #{}", pr_number);
            Ok(()) // Return success
        } else {
            // Try to extract and include the error response text for clarity
            Err(format!("Failed to submit review: {}", response.text()?).into())
        }
    }

    /// Displays a diff of the PR branch versus the `origin/main` branch.
    ///
    /// This function assumes that the pull request has already been fetched
    /// and checked out locally using a consistent naming convention.
    /// It constructs the branch name and diff range, runs a `git diff`,
    /// and reports failure if the command doesn't succeed.
    fn show_diff(&self, pr_number: &str) {
        // Construct the expected local branch name for the PR.
        // This assumes a naming scheme like: `pr-request-<PR_NUMBER>`
        let branch = format!("pr-request-{}", pr_number);

        // Define the diff range using Git's triple-dot syntax.
        // This compares changes from the merge base between origin/main and the PR branch.
        let diff_range = format!("origin/main...{}", branch);

        // Print a debug message showing the exact diff command being run.
        debug_log!("[DEBUG] Running: git diff {}", diff_range);

        // Execute the `git diff` command in a subprocess using std::process::Command.
        // This will output the differences between the two branches.
        let diff = Command::new("git")
            .args(["diff", &diff_range])
            .status() // This returns a `Result<ExitStatus>` indicating success or failure.
            .expect("Failed to show diff"); // Panic if the command itself fails to launch.

        // Log that the diff command was invoked, even if it failed.
        debug_log!("[DEBUG] Preparing Git Diff {}", diff_range);

        // Check if the `git diff` command failed based on its exit status.
        // This can happen if the branch doesn't exist or Git errors out.
        if !diff.success() {
            // Print a human-readable error message if the diff fails.
            eprintln!("{}", "❌ Failed to display diff.".red());
        }
    }

    /// Pulls a GitHub pull request (PR) and checks out a corresponding local branch.
    /// This function supports two main scenarios for how PRs are created on GitHub:
    ///
    /// ---
    ///
    /// ## 🔁 Scenario 1: PR From Same Repository (Not Forked)
    ///
    /// - The contributor created a branch in the **same repository** (upstream/original repo).
    /// - The `head.repo.full_name` is the same as the `base.repo.full_name`.
    /// - This function:
    ///   - Fetches the PR's head branch from `origin`.
    ///   - Creates a local branch with the same name (e.g., `feature-x`).
    ///   - Sets it to track `origin/feature-x`.
    /// - ✅ The user can directly push commits to this branch if they have write access to the repo.
    /// - This is the **ideal flow for collaboration** within the same team/org.
    ///
    /// ---
    ///
    /// ## 🍴 Scenario 2: PR From a Forked Repository
    ///
    /// - The contributor forked the upstream repo, pushed changes to the fork, and opened a PR.
    /// - The `head.repo.full_name` is different from `base.repo.full_name`.
    /// - This function:
    ///   - Uses GitHub’s special `refs/pull/<PR_NUMBER>/head` to fetch the PR as a read-only branch.
    ///   - Creates a local branch named `<fork-owner>-pr-<PR_NUMBER>` (e.g., `alice-pr-42`).
    ///   - Checks out this local branch, but does **not** connect it to any remote.
    /// - ⚠️ This branch cannot be pushed back to the original PR directly, since the upstream user
    ///   doesn’t have write access to the fork.
    /// - If you need to make changes, you must create a **new branch** and open a **separate PR**.
    ///
    /// ---
    ///
    /// ## 💡 Tip:
    /// When working with PRs from forks, you can cherry-pick or patch the commits to your own branch,
    /// but cannot push directly to the fork’s branch unless you have permissions.
    ///
    /// ---
    fn pull_pr(&self, pr_number: &str) {
        // Get the origin URL of the current Git repository (e.g., git@github.com:owner/repo.git)
        let remote_url = get_remote_url().unwrap_or_else(|| {
            eprintln!("{}", "❌ Could not determine remote URL.".red());
            std::process::exit(1);
        });

        // Create a GitHub provider instance using the remote URL
        // This gives access to authenticated API operations and utilities
        let github = GitHubProvider::new(remote_url.clone()).unwrap_or_else(|e| {
            eprintln!(
                "{}",
                format!("❌ Failed to create GitHubProvider: {}", e).red()
            );
            std::process::exit(1);
        });

        // Infer GitHub repo owner and repo name from remote URL
        // Example: git@github.com:foo/bar.git → ("foo", "bar")
        let (owner, repo) = github.infer_repo_details().unwrap_or_else(|| {
            eprintln!("{}", "❌ Could not infer owner/repo.".red());
            std::process::exit(1);
        });

        let client = &github.client;
        let token = &github.token;

        // Construct GitHub API URL for fetching pull request metadata
        let pr_url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo, pr_number
        );
        debug_log!("[DEBUG] Fetching PR info from: {}", pr_url);

        // Perform authenticated API GET request to retrieve PR details
        let pr_resp = client
            .get(&pr_url)
            .bearer_auth(token)
            .header("User-Agent", "git-pr")
            .send()
            .expect("Failed to fetch PR info");

        // Abort if the response isn't a success
        if !pr_resp.status().is_success() {
            eprintln!(
                "{}",
                format!("❌ Failed to fetch PR: {}", pr_resp.status()).red()
            );
            std::process::exit(1);
        }

        // Parse JSON response containing PR metadata
        let pr_json: serde_json::Value = pr_resp.json().expect("Failed to parse PR JSON");

        // Extract head branch name from the PR
        let head_branch = pr_json["head"]["ref"].as_str().unwrap_or("");

        // Extract the full name of the head repo (e.g., "user/repo")
        let head_repo = pr_json["head"]["repo"]["full_name"].as_str().unwrap_or("");

        // Extract the GitHub login of the user who owns the head repo
        let head_repo_owner = pr_json["head"]["repo"]["owner"]["login"]
            .as_str()
            .unwrap_or("");

        // Extract the full name of the base repository that the PR targets
        let base_repo = pr_json["base"]["repo"]["full_name"].as_str().unwrap_or("");

        // Determine if the PR is from a fork (head repo != base repo)
        let head_is_fork = head_repo != base_repo;

        debug_log!(
            "[DEBUG] PR head branch: {}, head repo: {}, head owner: {}, base repo: {}, is fork: {}",
            head_branch,
            head_repo,
            head_repo_owner,
            base_repo,
            head_is_fork
        );

        // Get authenticated user's GitHub username (via /user endpoint)
        let user_resp = client
            .get("https://api.github.com/user")
            .bearer_auth(token)
            .header("User-Agent", "git-pr")
            .send()
            .expect("Failed to fetch authenticated user");

        let user_json: serde_json::Value = user_resp.json().expect("Failed to parse user JSON");
        let username = user_json["login"].as_str().unwrap_or("");
        debug_log!("[DEBUG] Authenticated as: {}", username);

        // Handle the case where the PR is from the same repository (not a fork)
        if !head_is_fork {
            debug_log!("[DEBUG] PR is from same repository. Using origin tracking.");

            let local_branch = head_branch.to_string();

            // Fetch the PR branch from origin and create a local branch with same name
            let _ = Command::new("git")
                .args([
                    "fetch",
                    "origin",
                    &format!("{}:{}", head_branch, local_branch),
                ])
                .status();

            // Check out the local branch just created
            let _ = Command::new("git")
                .args(["checkout", &local_branch])
                .status();

            // Set the upstream for the branch to track origin/<branch>
            let _ = Command::new("git")
                .args([
                    "branch",
                    "--set-upstream-to",
                    &format!("origin/{}", head_branch),
                    &local_branch,
                ])
                .status();

            // Inform user of success and push capability
            println!(
                "{}",
                format!(
                    "✅ Switched to branch {} tracking origin/{}",
                    local_branch.green(),
                    head_branch
                )
            );
            return;
        } else {
            // Handle case where PR is from a fork (read-only access to head repo)
            debug_log!("[DEBUG] PR is from fork. Will fetch as read-only checkout.");

            // Create local branch name using format "<username>-pr-<number>"
            let local_branch = format!("{}-pr-{}", head_repo_owner, pr_number);

            // Use GitHub's pull/<ID>/head ref to fetch a temporary read-only copy
            let fetch = Command::new("git")
                .args([
                    "fetch",
                    "origin",
                    &format!("pull/{}/head:{}", pr_number, local_branch),
                ])
                .status()
                .expect("Failed to fetch PR");

            if !fetch.success() {
                eprintln!("{}", "❌ Failed to fetch PR.".red());
                std::process::exit(1);
            }

            // Checkout the read-only branch
            let checkout = Command::new("git")
                .args(["checkout", &local_branch])
                .status()
                .expect("Failed to checkout PR branch");

            if checkout.success() {
                // Let user know that branch is local, detached from the fork
                println!("✅ Switched to branch {}", local_branch.green());
                println!(
                    "This branch is a read-only checkout of PR #{}, since it comes from a fork.",
                    pr_number
                );
            } else {
                eprintln!("{}", "❌ Failed to checkout PR branch.".red());
            }
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

    /// This is only used with `submit-review --reject` option, if `--reject` switch is used with
    /// `submit-review` then PR will be closed as REJECTED. `close_pull_request` helps to close the
    /// pull request (PR) on GitHub by setting its state to "closed" via the GitHub REST API.
    /// This method sends an authenticated PATCH request to the GitHub API to change
    /// the PR's state, effectively closing it.
    ///
    /// # Arguments
    ///
    /// * `pr_number` - The pull request number (as a string) to close.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the PR was successfully closed.
    /// * `Err(...)` if there was a failure during the API request or processing.
    ///
    /// # Example
    ///
    /// ```no_run
    /// close_pull_request("42")?;
    /// ```
    ///
    /// ```no_run
    /// git pr submit-review 10 --message "garbage pr" --reject
    /// ```
    ///
    fn close_pull_request(&self, pr_number: &str) -> Result<(), Box<dyn Error>> {
        // Log debug message indicating the start of the PR close operation.
        debug_log!("[DEBUG] Closing PR #{}", pr_number);

        // Attempt to parse the repository owner and name from the remote Git URL.
        // This is essential for constructing the API endpoint URL.
        //
        // `infer_repo_details()` returns an Option<(owner, repo)>, so we
        // handle the None case by returning an error.
        let (owner, repo) = self
            .infer_repo_details()
            .ok_or("Could not parse owner/repo")?;

        // Construct the GitHub API endpoint URL for the specific pull request.
        //
        // Example URL:
        // https://api.github.com/repos/owner/repo/pulls/42
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo, pr_number
        );

        // Create the JSON payload to send in the PATCH request.
        // Setting `"state": "closed"` instructs GitHub to close the PR.
        //
        // This JSON body will be sent as the request payload.
        let body = json!({ "state": "closed" });

        // Debug log the outgoing request body and URL for troubleshooting.
        debug_log!("📬 [DEBUG] Request Sent: {} to URL: {}", body, url);

        // Send a PATCH request to the GitHub API to update the PR.
        //
        // - Use the authenticated HTTP client stored in `self.client`.
        // - Bearer token authentication with `self.token` ensures the request is authorized.
        // - Set a "User-Agent" header to identify the client, which GitHub requires.
        // - Send the JSON body created above.
        //
        // This call may return an error (e.g., network failure), so we propagate it with `?`.
        let response = self
            .client
            .patch(&url)
            .bearer_auth(&self.token)
            .header("User-Agent", "git-pr")
            .json(&body)
            .send()?;

        // Log the HTTP response status code for debugging purposes.
        debug_log!(
            "📬 [DEBUG] Response Received: {} from URL: {}",
            response.status(),
            url
        );

        // Check if the HTTP response indicates success (status 2xx).
        if response.status().is_success() {
            // Inform the user that the PR was successfully closed.
            println!("✅ Successfully closed PR #{}", pr_number);
            Ok(())
        } else {
            // On failure, read the response body text (error message from GitHub)
            // and convert it into an error returned from this method.
            Err(format!("Failed to close PR: {}", response.text()?).into())
        }
    }

    /// Shows detailed information about a pull request (PR),
    /// including metadata like title, author, status, age,
    /// and lists all commits along with their changed files.
    ///
    /// This method uses the GitHub REST API to fetch all relevant data.
    ///
    /// # Arguments
    ///
    /// * `pr_number` - The number of the pull request to display.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success, after printing the PR details table.
    /// * `Err(...)` if any API request or parsing step fails.
    ///
    fn show_pull_request_details(&self, pr_number: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Log debug info that we're starting to show details for the specified PR
        debug_log!("[DEBUG] Showing Details for PR #{}", pr_number);

        // Infer the GitHub repo owner and repository name from the remote URL
        // This is necessary to build the API URLs for requests.
        let (owner, repo) = self
            .infer_repo_details()
            .ok_or("Could not parse owner/repo")?;

        // Construct the GitHub API endpoint URL to fetch PR metadata.
        // This includes title, author, status, creation date, etc.
        let pr_url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo, pr_number
        );

        // Debug log the API URL for fetching PR metadata
        debug_log!("[DEBUG] Fetching PR metadata from: {}", pr_url);

        // Perform an authenticated GET request to the GitHub API
        // to retrieve the PR metadata as JSON.
        let pr_resp = self
            .client
            .get(&pr_url)
            .bearer_auth(&self.token)
            .header("User-Agent", "git-pr")
            .send()?;

        // Check if the HTTP response was successful (status 2xx).
        // If not, return an error with the response body text.
        if !pr_resp.status().is_success() {
            return Err(format!("Failed to fetch PR details: {}", pr_resp.text()?).into());
        }

        // Parse the JSON response into a serde_json::Value for flexible access.
        let pr_json: serde_json::Value = pr_resp.json()?;

        // Extract useful fields from the JSON:
        // - title: The PR title
        // - state: The PR status (open, closed, merged)
        // - user.login: The username of the PR author
        // - created_at: Timestamp when the PR was created (RFC 3339 format)
        //
        // Use `unwrap_or("-")` to provide a default if fields are missing.
        let title = pr_json["title"].as_str().unwrap_or("-");
        let status = pr_json["state"].as_str().unwrap_or("-");
        let user = pr_json["user"]["login"].as_str().unwrap_or("-");
        let created_at = pr_json["created_at"].as_str().unwrap_or("-");

        // Parse the creation timestamp into a DateTime<Utc> for calculations
        let created_date = DateTime::parse_from_rfc3339(created_at)?.with_timezone(&Utc);

        // Calculate the age of the PR in days, relative to now (UTC)
        let age_days = (Utc::now() - created_date).num_days();

        // Convert the age into a human-readable string:
        // - "today" if less than 1 day old
        // - "<n>d" otherwise (e.g., "5d" for 5 days)
        let age = if age_days == 0 {
            "today".to_string()
        } else {
            format!("{}d", age_days)
        };

        // Debug log all extracted metadata for troubleshooting
        debug_log!(
        "[DEBUG] PR #{}: title={}, status={}, author={}, age={}d",
        pr_number,
        title,
        status,
        user,
        age_days
    );

        // Construct the GitHub API URL to fetch the list of commits on this PR
        let commits_url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/commits",
            owner, repo, pr_number
        );

        // Perform authenticated GET request to retrieve commits as JSON
        let commits_resp = self
            .client
            .get(&commits_url)
            .bearer_auth(&self.token)
            .header("User-Agent", "git-pr")
            .send()?;

        // Return an error if the commits API call fails
        if !commits_resp.status().is_success() {
            return Err(format!("Failed to fetch commits: {}", commits_resp.text()?).into());
        }

        // Parse the commits response JSON into a vector of JSON values (each a commit)
        let commits: Vec<serde_json::Value> = commits_resp.json()?;

        // Vector to hold rows for tabular output
        let mut rows = Vec::new();

        // Iterate over each commit to collect details and changed files
        for (i, commit) in commits.iter().enumerate() {
            // Extract the full commit SHA and create a shortened SHA (first 7 chars)
            let sha = commit["sha"].as_str().unwrap_or("-");
            let short_sha = &sha[..7.min(sha.len())];

            // Construct the GitHub API URL to fetch detailed commit info (including changed files)
            let commit_url = format!(
                "https://api.github.com/repos/{}/{}/commits/{}",
                owner, repo, sha
            );

            // Log the commit we're fetching files for
            debug_log!("[DEBUG] Fetching files for commit {}", short_sha);

            // Fetch detailed commit info JSON via authenticated GET request
            let commit_resp = self
                .client
                .get(&commit_url)
                .bearer_auth(&self.token)
                .header("User-Agent", "git-pr")
                .send()?;

            // If fetching commit details failed, print warning and skip this commit
            if !commit_resp.status().is_success() {
                eprintln!(
                    "⚠️  Failed to fetch commit {}: {}",
                    sha,
                    commit_resp.text()?
                );
                continue;
            }

            // Parse commit JSON to extract list of changed files
            let commit_json: serde_json::Value = commit_resp.json()?;
            let files = commit_json["files"]
                .as_array()
                .unwrap_or(&vec![]) // fallback to empty array if missing
                .iter()
                .filter_map(|f| f["filename"].as_str()) // extract filename strings
                .collect::<Vec<_>>() // collect into Vec<&str>
                .join(", "); // join filenames as comma-separated string

            // Build a PRDetailsRow for this commit.
            // For the first commit row, include PR metadata fields.
            // For subsequent commits, leave PR metadata blank to avoid repetition.
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

            // Append the row to the rows vector
            rows.push(row);
        }

        // Create a table using the collected rows
        let mut table = Table::new(rows);

        // Apply a rounded style to the table for nicer visual appearance
        table.with(Style::rounded());

        // Print the completed table to stdout
        println!("{table}");

        // Return success
        Ok(())
    }
}
