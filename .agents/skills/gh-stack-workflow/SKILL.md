---
name: gh-stack-workflow
description: Manage GitHub stacked pull requests with the gh-stack CLI and GitHub Stack APIs. Use when Codex needs to create, link, inspect, review, synchronize, rebase, atomically merge, or clean up a chain of dependent GitHub PRs; distinguish a real server-side Stack from ordinary base-branch chaining; diagnose review, ruleset, CI, merge-queue, or squash-rebase blockers; or recover safely after part of a stack has merged.
---

# GitHub Stacked PR Workflow

Use GitHub's server-side Stack object as the source of truth. Query live state before changing branches, reviews, PR bases, or merge state because stacked PR support is evolving.

## Operating Rules

1. Read the repository's local contribution and PR rules first.
2. Inspect the working tree and remote PR state before mutating anything.
3. Treat this Codex skill and the `github/gh-stack` CLI extension as separate installations. Loading the skill does not provide the `gh stack` command.
4. Treat a branch chain and a GitHub Stack as different concepts. A real Stack has a non-null `stack` object on each PR.
5. Treat `stack.number` as a Stack identifier, not a missing PR number.
6. Never infer readiness from "reviewed" text, green-looking checks, or `mergeable` alone. Query approvals, approver permissions, required checks, unresolved threads, and all applicable branch policies.
7. Use `gh stack merge` for a linked Stack. Do not merge its members one by one with `gh pr merge` unless the Stack feature is unavailable and the user explicitly accepts the manual fallback.
8. Prefer an atomic merge through the top intended PR. If any member is blocked, fix the blocker instead of partially landing the stack.
9. Do not bypass branch protection or approve a PR under the user's identity without explicit authorization.
10. Verify all target PRs are merged before deleting any branch.

## Workflow

### 0. Verify Tooling

- Run `gh auth status`, `gh extension list`, and `gh stack --help`.
- If `gh stack` is unavailable and mutation is authorized, install the official extension with `gh extension install github/gh-stack` and verify the command again.
- Do not confuse successful skill discovery with extension installation.
- Record the extension version when diagnosing preview behavior.

Read [references/operations.md](references/operations.md) for installation and creation commands.

### 1. Discover The Stack

- Resolve repository owner/name and fetch remote refs.
- Query the candidate PR through the REST API and inspect `stack.number`, `size`, `position`, and base.
- Enumerate every Stack member from bottom to top.
- Confirm branch ancestry with `git merge-base --is-ancestor` when local refs are available.
- If `stack` is null, classify the PRs as an ordinary dependent chain. Link them only when the user intends to create a GitHub Stack.

Read [references/diagnostics.md](references/diagnostics.md) for inspection commands.

### 2. Establish Readiness

For every selected member, record:

- PR state and draft state
- direct base and Stack target base
- head SHA and commit count
- `reviewDecision` and latest review states
- latest approval commit SHA and each approver's repository permission
- required status checks from applicable rulesets and classic branch protection
- unresolved review threads
- merge queue or auto-merge state

Differentiate a failed optional check from a failed required check. GitHub combines applicable rulesets and classic branch protection; inspect the effective policy set. A push may dismiss an earlier approval.

### 3. Choose The Operation

- **Create a new stack:** use `gh stack init`, `add`, and `submit`.
- **Adopt existing branches locally:** use `gh stack init --base <trunk> <bottom> ... <top>`.
- **Link existing PRs without local tracking:** use `gh stack link --base <trunk> <bottom-pr> ... <top-pr>`.
- **Update a healthy stack:** use `gh stack sync`.
- **Rebase after trunk or a lower layer changes:** use `gh stack rebase`; let its merge-aware `--onto` behavior avoid replaying squash-merged commits.
- **Merge:** use `gh stack merge <top-pr> --yes --squash` unless repository policy selects another method.
- **Restructure:** use `gh stack modify` only with a clean worktree, linear history, no active rebase, and no member queued for merge.

Read [references/operations.md](references/operations.md) before mutating a stack.

### 4. Merge Atomically

Passing a PR number merges every Stack member from the bottom through that PR. Confirm that the chosen PR is the intended top boundary.

Use the repository's allowed merge method. In a linear-history repository, prefer squash unless local rules say otherwise. Stack merge is all-or-nothing outside merge queues: a protection or review failure must leave every member unmerged.

When a merge queue is active, let the queue choose the merge method. Do not claim that queued members will necessarily land in one commit group.

On failure, preserve atomicity, re-query every member, and diagnose the reported condition before retrying. Read [references/diagnostics.md](references/diagnostics.md) for failure patterns and evidence checks.

### 5. Verify And Clean Up

- Poll until the asynchronous merge reaches a terminal result.
- Re-query every member and verify `MERGED` plus the expected target base.
- Fetch/prune and fast-forward local trunk without discarding unrelated work.
- Run `gh stack sync --prune` only for local Stack tracking and merged local branches.
- Query `delete_branch_on_merge`; delete remote branches separately only after successful verification.
- Report merge method, merged PR range, resulting trunk SHA, retained/deleted branches, and any skipped optional checks.

## Manual Fallback

Use manual bottom-up merging only when GitHub Stack merge is unavailable and the user approves the loss of atomicity.

Sequentially merging one Stack boundary at a time also loses whole-stack atomicity. Follow [references/operations.md](references/operations.md), re-query after every mutation, and stop on the first conflict or unmet policy.

## Skill Maintenance

When this skill is installed at project scope, treat the repository-local copy (commonly `.agents/skills/gh-stack-workflow`) as canonical. Keep temporary plans and generated artifacts outside the skill. Edit and validate the project source first; synchronize any user-level installed copy only afterward.

Do not encode one repository's branch names, CI jobs, merge policy, reviewer identities, or timing assumptions as defaults. Discover those values from live repository state and local contribution rules on every use.

Validate changes with the `skill-creator` validator through `uv` so YAML parsing is explicit and reproducible:

```powershell
uv run --with pyyaml python <skill-creator>/scripts/quick_validate.py .agents/skills/gh-stack-workflow
```

## Sources

Before relying on preview-specific behavior, verify the current official documentation:

- `https://github.com/github/gh-stack`
- `https://docs.github.com/en/pull-requests/how-tos/create-pull-requests/managing-stacked-pull-requests`
- `https://docs.github.com/en/pull-requests/reference/stacked-pull-requests-apis-and-webhooks`
