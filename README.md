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

- 📋 List open pull requests in the current repository (in tabular format)
- 📥 Pull a PR into a local branch
- 🔍 Show diffs between PR and main
- 📊 Display full PR details with commits and changed files
- 📝 Submit a review with approval, comment-only, or rejection
- 🐞 Debug mode for verbose API logging

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
git-pr list
git-pr pull <PR_NUMBER>
git-pr show-diff <PR_NUMBER>
git-pr submit-review <PR_NUMBER> --message "Looks great!"
git pr show-details 5
```

## 🛠️ Command Reference

| Command           | Description                         |
|-------------------|-------------------------------------|
| `list`            | List open pull requests             |
| `pull <number>`   | Fetch and checkout a PR             |
| `show-diff <num>` | Show diff between `main` and the PR |
| `submit-review`   | Submit a review with a message      |
| `show-details`    | Shows the details about the PR      |

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

#### Show the Diff

```bash
> git pr show-diff 1
🔍 Showing diff for PR #1...

README.md
───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

───┐
1: │
───┘
# testing-git-pr]
## First PR to test
```

#### Tried to review my own PR 😉

```bash
> git pr submit-review --message "Looks good to me" 1
📝 Submitting review for PR #1...
❌ Failed to submit review: Unprocessable Entity: Can not approve your own pull request
```

#### Approving PR (not my own 😉)

```bash
> git pr submit-review 2 -m "Looks good to me"
📝 Submitting review for PR #2...
✅ Review submitted successfully for PR #2
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