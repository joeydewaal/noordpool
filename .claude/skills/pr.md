---
description: Create a pull request on a new branch with a conventional commit title
---

## Steps

1. Create a new branch from `origin/main`:
   ```sh
   git checkout -b <prefix>/<short-description> origin/main
   ```

2. Stage and commit changes with a conventional commit message.

3. Push the branch:
   ```sh
   git push -u origin <prefix>/<short-description>
   ```

4. Open the PR with `gh pr create`.

## Branch and title prefixes

Use conventional commit prefixes for both the branch name and PR title:

| Prefix       | When to use                              |
|------------- |------------------------------------------|
| `feat`       | New feature                              |
| `fix`        | Bug fix                                  |
| `docs`       | Documentation only                       |
| `refactor`   | Code change that doesn't fix or add      |
| `chore`      | Build, CI, deps, tooling                 |
| `test`       | Adding or updating tests                 |
| `style`      | Formatting, whitespace, no code change   |

## PR title format

```
<prefix>: short imperative description (under 70 chars)
```

Examples:
- `feat: add admin user management page`
- `fix: prevent duplicate push subscriptions`
- `chore: update rust toolchain to 1.82`

## PR body format

```markdown
## Summary
- Bullet points describing what changed and why

## Test plan
- [ ] How to verify the changes work
```

## Example

```sh
git checkout -b feat/admin-users origin/main
git add ...
git commit -m "feat: add admin user management page

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
git push -u origin feat/admin-users
gh pr create --title "feat: add admin user management page" --body "## Summary
- Add /admin/users page for managing user roles
- List all users with role badges and moderator toggle

## Test plan
- [ ] Admin can view user list and toggle moderator role
- [ ] Non-admins are redirected"
```
