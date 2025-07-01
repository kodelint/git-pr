# git-pr 🧩

> A Git CLI extension to streamline your workflow with GitHub Pull Requests — view, pull, diff, and review PRs right
> from your terminal.

✨ Features

- 📋 List open pull requests in the current repository
- 📥 Pull a PR into a local branch
- 🔍 Show diffs for a PR against main
- 📝 Submit an approval review on a pull request
- 🐞 Verbose debug mode for transparency during GitHub API interactions

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
```

## 🛠️ Command Reference

| Command           | Description                         |
|-------------------|-------------------------------------|
| `list`            | List open pull requests             |
| `pull <number>`   | Fetch and checkout a PR             |
| `show-diff <num>` | Show diff between `main` and the PR |
| `submit-review`   | Submit a review with a message      |

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

```bash
# List of PRs

> git pr list
📋 Open Pull Requests:
#1: Testing my first PR

# Pull a PR locally

> git pr pull 1
📥 Pulling PR #1...
Switched to branch 'pr-request-1'
✅ Switched to branch pr-request-1

# Show the Diff
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

Note: the `show-diff` is using [`delta`](https://github.com/dandavison/delta) as git's default diff viewer

## Limitations

- Only works with GitHub remotes.
- Assumes `origin` is your GitHub remote.