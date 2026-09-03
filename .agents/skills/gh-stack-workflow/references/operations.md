# Operations Reference

Commands in this reference use PowerShell where shell syntax matters. The underlying `gh` and `git` operations are not repository-specific; substitute the repository's actual trunk, branches, PR numbers, rules, and merge method.

## Install And Inspect

```powershell
gh auth status
gh extension list
gh stack --help
```

The Codex skill and the GitHub CLI extension are independent. If `gh stack --help` reports an unknown command and `github/gh-stack` is absent, install it explicitly:

```powershell
gh extension install github/gh-stack
gh extension list
gh stack --help
```

Do not install or upgrade the extension when the user requested a read-only audit. Query the REST API directly instead. Do not reinstall an already listed extension merely because an operation failed; inspect authentication, repository access, Stack state, and the reported API error first.

Interpret common setup results:

| Result                                                | Meaning                                                                                 | Action                                                         |
| ----------------------------------------------------- | --------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| `unknown command "stack"`                             | The skill is present but the CLI extension is not available                             | Install `github/gh-stack`, then verify `gh stack --help`       |
| `gh extension list` is empty                          | No GitHub CLI extension is installed for this user profile                              | Install only when remote mutation/tool installation is allowed |
| Extension is listed but commands fail authentication  | The binary exists, but the active `gh` account or token is unsuitable                   | Run `gh auth status`; do not reinstall                         |
| Extension is listed and merge returns a ruleset error | Tool setup succeeded; repository policy or a Stack Preview defect blocked the operation | Follow the readiness and preview-failure diagnostics           |

## Create A Stack

```powershell
# Create the bottom branch from the repository trunk.
gh stack init

# Commit on the current layer, then add another branch above it.
gh stack add api-layer
gh stack add host-adapter

# Push branches, create PRs, and create the server-side Stack object.
gh stack submit
```

Use `gh stack submit --auto --open` only when generated PR metadata is acceptable. Follow repository rules for PR titles and descriptions.

After `submit`, query the resulting PRs and verify that each has a non-null `stack` object, a common Stack number and target base, and contiguous bottom-to-top positions. A dependent branch chain without those fields is not a server-side Stack.

`submit` is a multi-phase operation: it pushes branches sequentially, creates or updates PRs, adjusts bases, and then creates or updates the Stack object. A later failure can leave earlier branches or PRs changed. On any error, re-query every branch, PR, and Stack member before retrying; do not assume rollback.

## Adopt Or Link Existing Work

Adopt existing local branches in bottom-to-top order:

```powershell
gh stack init --base main feature-core feature-api feature-host
gh stack submit
```

Link existing remote PRs without creating local tracking:

```powershell
gh stack link --base main 101 102 103
```

`link` may correct PR base branches and creates or updates a server-side Stack. Treat it as a remote mutation and require clear user intent.

After `link`, repeat the same server-side membership check. Do not proceed from successful command exit alone.

`link` can create missing PRs, update existing PR bases, and then create or extend a Stack. These phases are not one transaction. If a phase fails, inventory created PRs, changed bases, and current Stack membership before deciding whether to resume or undo anything.

## Navigate And View

```powershell
gh stack checkout 103
gh stack view --json
gh stack bottom
gh stack up
gh stack top
gh stack trunk
```

A bare number may resolve as a Stack number or PR number. Inspect output rather than assuming which identifier was selected.

## Synchronize

```powershell
gh stack sync
gh stack sync --prune
```

`sync` fetches, reconciles local/remote Stack composition, fast-forwards trunk when possible, cascades rebases after trunk movement, pushes with lease protection, refreshes PR state, and updates the server-side Stack.

Use `--prune` only after confirming merged local branches are disposable. It does not imply remote branch deletion.

Do not treat exit code zero alone as proof that synchronization occurred. In a non-interactive terminal, detected local/remote Stack divergence can stop the operation before pushing or updating PRs and still return success after printing the reason. Require the final `Stack synced` or `Branches synced` message, then re-query local refs, remote refs, PR bases, and Stack composition.

`sync` uses an atomic multi-branch push, but the complete command is not a remote transaction: later PR or Stack reconciliation can still fail after branches were pushed.

## Rebase

```powershell
gh stack rebase
gh stack rebase --downstack
gh stack rebase --upstack
gh stack rebase --no-trunk
```

On conflict:

```powershell
git status
# Resolve only the reported files.
git add <resolved-files>
gh stack rebase --continue

# Restore all Stack branches to their pre-rebase state when the operation is unsafe.
gh stack rebase --abort
```

Use `gh stack rebase` after a lower PR was squash-merged. Its merged-PR awareness switches to the equivalent of `git rebase --onto` and avoids replaying already merged commits.

## Review And Update A Lower Layer

```powershell
gh stack bottom
# Apply requested changes and commit.
gh stack rebase --upstack
gh stack push
```

`push` uses explicit per-branch force-with-lease checks and is not atomic across branches. One branch may update before another lease fails. On failure, fetch and compare every local/remote branch before retrying; already updated branches should not be assumed unchanged or rolled back.

Re-query approvals after pushing. Repositories may dismiss stale approvals on every new head commit.

## Atomic Merge

Merge every member through a chosen top PR:

```powershell
gh stack merge 103 --yes --squash
```

Merge a remote Stack by Stack number:

```powershell
gh stack merge 27 --yes --squash
```

Do not confuse these identifiers. Verify `stack.number`, positions, and the selected top PR before execution.

Properties:

- All selected members must be open and non-draft.
- GitHub evaluates branch protection and repository rules asynchronously.
- Bypassing merge requirements is unsupported.
- Outside a merge queue, selected members merge atomically or none merge.
- With a merge queue, members are enqueued together but may land in separate groups; the queue chooses the merge method.

On failure, preserve the complete command output and re-query every member before taking another action. An atomic failure should leave all selected PRs open. If any member changed state, stop and reconstruct the Stack state rather than assuming rollback or success.

## Sequential Fallback

Use this only after the user explicitly accepts the loss of whole-stack atomicity. Designate one merge actor and process one bottom boundary at a time:

```powershell
gh stack merge 101 --yes --squash
# Re-query the complete Stack before considering PR 102.
```

After every successful lower merge:

1. Query every member through REST and record `state`, `merged`, `merged_at`, direct base, and head SHA.
2. Expect GitHub to retarget the next PR to trunk and possibly force-update all upper heads.
3. Do not push or locally rebase an upper branch while GitHub is performing that update.
4. Re-check approvals, unresolved threads, and required checks against the new head. Wait for the new CI run; do not reuse green results from the previous head.
5. Merge the next boundary only after its remote state is ready.

Before each merge command, re-query the target again. If another actor already merged it, treat the remote result as authoritative and continue from the new bottom. If another actor used an unintended merge method, report the actual trunk history and stop trying to "correct" it by rewriting shared trunk.

## Post-Merge Verification

```powershell
$ids = 101, 102, 103
foreach ($id in $ids) {
  gh pr view $id --json number,state,mergedAt,mergeCommit,baseRefName,headRefName
}

git fetch --prune origin
git switch main
git pull --ff-only origin main
```

Check automatic remote branch deletion:

```powershell
gh api repos/{owner}/{repo} --jq '.delete_branch_on_merge'
```

When it is disabled, delete only verified merged remote branches:

```powershell
git push origin --delete feature-core feature-api feature-host
```

Never delete branches while the asynchronous Stack merge is pending or failed.

Verify the actual merge method from trunk history rather than command intent alone. Record trunk before and after each fallback merge and inspect the introduced commits:

```powershell
git log --oneline <before-trunk>..<after-trunk>
```

A command requesting squash does not prove that a concurrent actor used squash. Preserve unexpected rebase merges unless the user separately authorizes a history rewrite.
