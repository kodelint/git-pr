// CLI argument parsing via clap
use clap::{Parser, Subcommand};
// For colorful terminal output (errors, info, etc.)
use colored::*;

// Bring in custom provider logic (like GitHub)
mod providers;
// Module for General Utility functions
mod utils;
use providers::get_provider;

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
        pr_number: String,
    },

    // Show details for particular PR, takes PR Number as argument
    ShowDetails {
        pr_number: String,
    },

    /// Show the diff of a PR compared to main
    ShowDiff {
        pr_number: String,
    },
    /// Submit an approval review for a PR
    SubmitReview {
        /// Pull Request number (e.g., 42)
        pr_number: String,

        /// Optional review message (defaults to LGTM)
        #[arg(short, long, default_value = "Looks good to me.")]
        message: String,

        /// Action on the pull request: Approves
        #[arg(long, conflicts_with_all=&["reject", "comment_only"])]
        approve: bool,

        /// Action on the pull request: Rejects
        #[arg(long, conflicts_with_all=&["approve", "comment_only"])]
        reject: bool,

        /// Action on the pull request: Comments Only
        #[arg(long, conflicts_with_all=&["approve", "reject"])]
        comment_only: bool,
    },
    /// List all currently open pull requests for the repository
    List,
}

fn main() {
    // Parse CLI arguments using Clap
    let cli = Cli::parse();

    // Try to retrieve the Git remote origin URL for the repo
    // This is hard requirement that the Git repository has ORIGIN set
    // with remote URL
    let remote_url = match utils::get_remote_url() {
        Some(url) => url,
        None => {
            // Exit early if we can’t determine the remote. Git repo may be misconfigured.
            eprintln!("{}", "❌ Could not determine remote origin URL.".red());
            std::process::exit(1);
        }
    };

    // Choose the right `SourceControlProvider` implementation based on the remote.
    // Currently only GitHub is supported, but extensible for GitLab/Bitbucket later.
    let provider = match get_provider(&remote_url) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{} {}", "❌ Provider error:".red(), e);
            std::process::exit(1);
        }
    };

    // Dispatch based on which subcommand was used
    // For any of these commands to work
    // GITHUB_TOKEN variable needs to be set
    match cli.command {
        // Show a list of open PRs using ORIGIN URL
        Commands::List => {
            if let Err(e) = provider.list_pull_requests() {
                eprintln!("{} {}", "❌ Error listing PRs:".red(), e);
                std::process::exit(1);
            }
        }
        // Fetch PR details for a given PR Number
        Commands::ShowDetails { pr_number } => {
            if let Err(e) = provider.show_pull_request_details(&pr_number) {
                eprintln!("{} {}", "❌ Error showing PR details:".red(), e);
                std::process::exit(1);
            }
        }

        // Fetch and checkout to a branch for a specific PR by number
        Commands::Pull { pr_number } => {
            println!("{}", format!("📥 Pulling PR #{}...", pr_number).green());
            provider.pull_pr(&pr_number);
        }
        // Show the diff of a PR vs main
        // keep in mind that show-diff to work
        // present checked out branch should be the one with PR changes
        Commands::ShowDiff { pr_number } => {
            println!(
                "{}",
                format!("🔍 Showing diff for PR #{}...", pr_number).green()
            );
            provider.show_diff(&pr_number);
        }
        // Submit a code review for the PR
        // This is the little complicated one
        // Presently it supports following:
        ///////////////////////////////////////////////////////////////////////
        // Action: Approve
        // i.e. `git pr submit-review 4 -m "Looks good to me" --approve`
        //
        // Action: Reject and close the PR
        // i.e. `git pr submit-review 4 -m "Not Good" --reject`
        //
        // Action: Approve but comment only
        // i.e. `git pr submit-review 4 -m "Looks good to me" --comment-only`
        ///////////////////////////////////////////////////////////////////////
        Commands::SubmitReview {
            pr_number,
            message,
            approve,
            reject,
            comment_only,
        } => {
            if approve {
                println!(
                    "📝 Submitting APPROVAL review for PR #{}...",
                    pr_number.green()
                );
                if let Err(e) = provider.submit_review(&pr_number, &message, "APPROVE") {
                    eprintln!("{} {}", "❌ Error submitting review:".red(), e);
                    std::process::exit(1);
                }
            } else if reject {
                println!(
                    "📝 Submitting REQUEST_CHANGES review and closing PR #{}...",
                    pr_number.red()
                );

                if let Err(e) = provider.submit_review(&pr_number, &message, "REQUEST_CHANGES") {
                    eprintln!("{} {}", "❌ Error submitting review:".red(), e);
                    std::process::exit(1);
                }

                if let Err(e) = provider.close_pull_request(&pr_number) {
                    eprintln!("{} {}", "❌ Failed to close PR:".red(), e);
                    std::process::exit(1);
                }

                println!("✅ PR #{} successfully closed.", pr_number.green());
            } else if comment_only {
                println!(
                    "📝 Submitting COMMENT only review for PR #{}...",
                    pr_number.yellow()
                );
                if let Err(e) = provider.submit_review(&pr_number, &message, "COMMENT") {
                    eprintln!("{} {}", "❌ Error submitting review:".red(), e);
                    std::process::exit(1);
                }
            } else {
                println!(
                    "📝 No review flag specified, defaulting to APPROVE for PR #{}...",
                    pr_number.green()
                );
                if let Err(e) = provider.submit_review(&pr_number, &message, "APPROVE") {
                    eprintln!("{} {}", "❌ Error submitting review:".red(), e);
                    std::process::exit(1);
                }
            }
        }
    }
}
