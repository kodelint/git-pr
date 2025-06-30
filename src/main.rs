// CLI argument parsing via clap
use clap::{Parser, Subcommand};
// For colorful terminal output (errors, info, etc.)
use colored::*;
// To run shell commands like `git fetch` or `git diff`
use std::process::Command;

// Bring in custom provider logic (like GitHub)
mod providers {
    pub mod factory;
    pub mod github;
}

use providers::factory::get_provider;

/// CLI definition using Clap's derive macros.
///
/// This struct maps the overall CLI interface (`git-pr <COMMAND>`)
#[derive(Parser)]
#[command(name = "git-pr")]
#[command(about = "A Git plugin to interact with pull requests", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Enumeration of subcommands supported by `git-pr`.
///
/// Each variant corresponds to an operation you can perform:
/// Pulling, viewing diffs, submitting reviews, or listing PRs.
#[derive(Subcommand)]
enum Commands {
    /// Pull and checkout a PR branch locally
    Pull {
        /// Pull Request number (e.g., 42)
        pr_number: String,
    },
    /// Show the diff of a PR compared to main
    ShowDiff {
        /// Pull Request number (e.g., 42)
        pr_number: String,
    },
    /// Submit an approval review for a PR
    SubmitReview {
        /// Pull Request number (e.g., 42)
        pr_number: String,

        /// Optional review message (defaults to LGTM)
        #[arg(short, long, default_value = "Looks good to me.")]
        message: String,
    },
    /// List all currently open pull requests for the repository
    List,
}

fn main() {
    // Parse CLI arguments using Clap
    let cli = Cli::parse();

    // Try to retrieve the Git remote origin URL for the repo
    let remote_url = match get_remote_url() {
        Some(url) => url,
        None => {
            eprintln!("{}", "❌ Could not determine remote origin URL.".red());
            std::process::exit(1);
        }
    };

    // Get the appropriate SourceControlProvider (currently GitHub only)
    let provider = match get_provider(&remote_url) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{} {}", "❌ Provider error:".red(), e);
            std::process::exit(1);
        }
    };

    // Dispatch based on which subcommand was used
    match cli.command {
        // Show a list of open PRs
        Commands::List => {
            if let Err(e) = provider.list_pull_requests() {
                eprintln!("{} {}", "❌ Error listing PRs:".red(), e);
                std::process::exit(1);
            }
        }

        // Fetch and checkout a specific PR by number
        Commands::Pull { pr_number } => {
            println!("{}", format!("📥 Pulling PR #{}...", pr_number).green());
            pull_pr(&pr_number);
        }

        // Show the diff of a PR vs main
        Commands::ShowDiff { pr_number } => {
            println!(
                "{}",
                format!("🔍 Showing diff for PR #{}...", pr_number).green()
            );
            show_diff(&pr_number);
        }

        // Submit a code review for the PR (currently approves only)
        Commands::SubmitReview { pr_number, message } => {
            println!(
                "{}",
                format!("📝 Submitting review for PR #{}...", pr_number).green()
            );
            if let Err(e) = provider.submit_review(&pr_number, &message) {
                eprintln!("{} {}", "❌ Error submitting review:".red(), e);
                std::process::exit(1);
            }
        }
    }
}

/// Attempts to fetch the GitHub remote URL for the `origin` remote.
///
/// This will usually return something like:
/// - https://github.com/user/repo.git
/// - git@github.com:user/repo.git
fn get_remote_url() -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .expect("Failed to get remote URL");

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Pulls a PR from the remote and checks out a local tracking branch.
///
/// Uses the format `pull/{pr_number}/head:pr-request-{pr_number}`
/// to create a local branch for inspection or testing.
fn pull_pr(pr_number: &str) {
    let fetch_ref = format!("pull/{}/head:pr-request-{}", pr_number, pr_number);

    let fetch = Command::new("git")
        .args(["fetch", "origin", &fetch_ref])
        .status()
        .expect("Failed to fetch PR");

    if fetch.success() {
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
///
/// Assumes the PR has already been fetched and checked out via `pull_pr()`.
fn show_diff(pr_number: &str) {
    let branch = format!("pr-request-{}", pr_number);
    let diff = Command::new("git")
        .args(["diff", &format!("origin/main...{}", branch)])
        .status()
        .expect("Failed to show diff");

    if !diff.success() {
        eprintln!("{}", "❌ Failed to display diff.".red());
    }
}
