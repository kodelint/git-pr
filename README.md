[![CI](https://github.com/kodelint/git-pr/actions/workflows/release.yml/badge.svg)](https://github.com/kodelint/git-pr/actions/workflows/release.yml)
[![GitHub release](https://img.shields.io/github/release/kodelint/git-pr.svg)](https://github.com/kodelint/git-pr/releases)
[![GitHub stars](https://img.shields.io/github/stars/kodelint/git-pr.svg)](https://github.com/kodelint/git-pr/stargazers)
[![Last commit](https://img.shields.io/github/last-commit/kodelint/git-pr.svg)](https://github.com/kodelint/git-pr/commits/main)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/kodelint/git-pr/pulls)

# git-pr 🧩

```bash
           🧙‍♂️ The Legend of the Git Pull Request ✨
         ════════════════════════════════════════════════════

                ┌────────────────────────────────┐
                │   🔮 A New PR Has Appeared!    │
                └────────────────────────────────┘
                                ⬇️
       ┌────────────────────────────────────────────────────┐
       │ 💀 A merge conflict goblin blocks your progress!   │
       │                                                    │
       │ 🧝 Dev-Elf: "I'll summon git-fu to defeat it!"     │
       │ 🗡️  'git fetch origin pull/42/head:epic-quest-42'  │
       │ 🧌 Dev-Orc: "Bah! I use the GitHub UI."            │
       └────────────────────────────────────────────────────┘
                                ⬇️
       ┌────────────────────────────────────────────────────┐
       │ 🧙 You arrive at the Review Council                │
       │                                                    │
       │ 👑 Reviewer 1: "Did you test this on staging?"     │
       │ 🧟 Reviewer 2: *approves their own PR silently*    │
       │ 🐉 Reviewer 3: *starts a flame war in comments*    │
       └────────────────────────────────────────────────────┘
                                ⬇️
       ┌────────────────────────────────────────────────────┐
       │ 🦸 Enter `git-pr` – the Terminal Hero!             │
       │                                                    │
       │ 🛡️ `git pr pull 42` – pulls the enchanted branch   │
       │ 🔍 `git pr show-diff 42` – reveals arcane changes  │
       │ 📊 `git pr show-details 42` – inspect PR contents  │
       │ ✨ `git pr submit-review 42` – casts approval spell│
       │ 📜 `git pr list` – scrolls of pending quests       │
       └────────────────────────────────────────────────────┘
                                ⬇️
       ┌────────────────────────────────────────────────────┐
       │ 🎉 Victory! The PR is merged into the sacred main! │
       └────────────────────────────────────────────────────┘

               🏰 May your conflicts be few
               💡 And your reviews ever wise
               🔧 Let `git-pr` be your trusted blade

```

> A Git CLI extension to streamline your workflow with GitHub Pull Requests — view, pull, diff, and review PRs right
> from your terminal.

✨ Features

- 📋 List open PRs in your current GitHub repo
- 📥 Pull PR branches into local Git
- 🔍 Show Git diffs between PR branch and main
- 📊 Inspect PR metadata: title, status, author, commits, files
- 📝 Submit reviews: `--approve`, `--comment-only`, or `--reject`
- ❌ Close PRs directly from terminal (when rejected)
- 🐞 `DEBUG=1` support for verbose output & GitHub API traces
- ⚙️ Works with both same-repo and forked PRs

## Installation

```bash
git clone https://github.com/yourusername/git-pr.git
cd git-pr
cargo install --path .

```

## 🔧 Setup

```bash
export GITHUB_TOKEN=ghp_xxx123yourtoken
```

### 💡 Minimum scopes required:

- `repo` (for private repos)
- `public_repo` (for public-only)
- `write:discussion` (to review PRs)

## 🚀 Usage

All commands operate in the context of the GitHub repo defined by your local git remote.

```bash
git pr list                                                 # Gets PR List
git pr pull <PR_NUMBER>                                     # Pulls the PR locally
git pr show-diff <PR_NUMBER>                                # Shows the PR diff
git pr submit-review <PR_NUMBER> --message "Looks great!"   # Submits review
git pr show-details 5                                       # Show details about the PR
```

## 🛠️ Command Reference

| Command                     | Description                         |
|-----------------------------|-------------------------------------|
| `list`                      | List open pull requests             |
| `pull <pr_number>`          | Fetch and checkout a PR             |
| `show-diff <pr_number>`     | Show diff between `main` and the PR |
| `submit-review <pr_number>` | Submit a review with a message      |
| `show-details <pr_number>`  | Shows the details about the PR      |

```bash
git pr -help
A Git plugin to interact with pull requests

Usage: git pr <COMMAND>

Commands:
  pull           Pull and checkout a PR branch locally
  show-details   
  show-diff      Show the diff of a PR compared to main
  submit-review  Submit an approval review for a PR
  list           List all currently open pull requests for the repository
  help           Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## 🐛 Debug Mode

Enable `debug` logs by setting the environment variable:

```bash
export DEBUG=1
```

You'll see helpful debug messages like:

```bash
📡 [DEBUG] GET https://api.github.com/repos/owner/repo/pulls
📬 [DEBUG] Response status: 200 OK
📦 [DEBUG] Payload: {"body":"Looks good","event":"APPROVE"}
```

## 🚀 Examples

#### List of PRs

```bash
> git pr list

╭────────┬──────────┬──────────┬───────┬───────────────┬─────────────────────────┬────────┬──────────────────────────────────╮
│ Number │ Title    │ Author   │ Age   │ Total Commits │ Number of Changed Files │ Labels │ Description                      │
├────────┼──────────┼──────────┼───────┼───────────────┼─────────────────────────┼────────┼──────────────────────────────────┤
│ #5     │ Patch 1  │ github-u │ today │ 2             │ 2                       │ -      │ -                                │
│ #4     │ Check it │ github-u │ 1d    │ 2             │ 2                       │ -      │ This is for testing purpose only │
╰────────┴──────────┴──────────┴───────┴───────────────┴─────────────────────────┴────────┴──────────────────────────────────╯
```

#### Pull a PR locally

```bash
> git pr pull 1
📥 Pulling PR #1...
Switched to branch 'pr-request-1'
✅ Switched to branch pr-request-1
```

> Philosophy of pushing improvements or update the pull-request is simple:
> - The PR is to same Repo
    >

- The branch will be fetched, checked out and contributors with write access can push changes directly to the PR branch.

> - The PR is from a forked Repo
    >

- The PR is checked out locally as a new branch named `<fork-owner>-pr-<number>`, which cannot be pushed back to the
  fork. If needed, changes can be committed and pushed to a new branch in the original repo, continuing the work.

#### Show the Diff

```bash
> git pr show-diff 7
🔍 Showing diff for PR #7...

added: New-PR.md
───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

───┐
1: │
───┘
## Adding README
### Adding more to see if interactive works or not
### It should be able to add

removed: README.md
───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

───┐
0: │
───┘
## Adding README

Testing.md
───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

───┐
1: │
───┘
# Testing Markdown file
## Let's add few more lines
```

#### Show the Diff with `--raw`

```bash
git pr show-diff 7 --raw
🔍 Showing diff for PR #7...
diff --git a/New-PR.md b/New-PR.md
new file mode 100644
index 0000000..dbf6b10
--- /dev/null
+++ b/New-PR.md
@@ -0,0 +1,5 @@
+## Adding README
+
+### Adding more to see if interactive works or not
+
+### It should be able to add
diff --git a/README.md b/README.md
deleted file mode 100644
index 89ff3ba..0000000
--- a/README.md
+++ /dev/null
@@ -1 +0,0 @@
-## Adding README
diff --git a/Testing.md b/Testing.md
index 411c237..ae05846 100644
--- a/Testing.md
+++ b/Testing.md
@@ -1 +1,3 @@
 # Testing Markdown file
+
+## Let's add few more lines
```

#### Tried to review my own PR 😉

```bash
> git pr submit-review --message "Looks good to me" 1 --approve
📝 Submitting review for PR #1...
❌ Failed to submit review: Unprocessable Entity: Can not approve your own pull request
```

#### Approving/Rejecting/Commenting PR (not my own 😉)

```bash
> git pr submit-review 42 -m "Looks good!" --approve
📝 Submitting review for PR #42...
✅ Review submitted successfully for PR #42

> git pr submit-review 45 --message "Just Commenting" --comment-only
📝 Submitting COMMENT only review for PR #45...
✅ Review submitted successfully for PR #45

> git pr submit-review 46 --message "needs work" --reject
📝 Submitting REQUEST_CHANGES review and closing PR #46...
✅ Review submitted successfully for PR #46
✅ Successfully closed PR #46
✅ PR #46 successfully closed.
```

Note: the `show-diff` is using [`delta`](https://github.com/dandavison/delta) as git's default diff viewer

#### Show Details about a PR

```bash
git pr show-details 5
╭───────────┬─────────┬────────┬───────┬──────────┬────────────┬─────────────────────────╮
│ PR Number │ Title   │ Status │ Age   │ Authors  │ Commit SHA │ Changed Files           │
├───────────┼─────────┼────────┼───────┼──────────┼────────────┼─────────────────────────┤
│ #5        │ Patch 1 │ open   │ today │ github-u │ 2f72501    │ Add-file2.md, README.md │
│           │         │        │       │          │ 205178f    │ README.md               │
╰───────────┴─────────┴────────┴───────┴──────────┴────────────┴─────────────────────────╯
```

## Limitations

- Only works with GitHub remotes.
- Assumes `origin` is your GitHub remote.