# Diagnostics Reference

Commands in this reference use PowerShell for structured output. The `gh` and `git` operations are portable; translate only the shell-specific loops and object formatting when using another shell.

## Confirm Server-Side Stack Membership

```powershell
$owner = "OWNER"
$repo = "REPO"
$pr = 103
$data = gh api "repos/$owner/$repo/pulls/$pr" | ConvertFrom-Json
$data.stack | Format-List
```

Expected fields include Stack number, size, PR position, and Stack target base. A null object means the PR is not linked to a GitHub Stack even when its branch targets another feature branch.

Enumerate the authoritative member list from the candidate's Stack object, then inspect every member:

```powershell
$stack = gh api "repos/$owner/$repo/stacks/$($data.stack.number)" |
  ConvertFrom-Json
$ids = @($stack.pull_requests | ForEach-Object number)
$rows = @(
  foreach ($id in $ids) {
    $pr = gh api "repos/$owner/$repo/pulls/$id" | ConvertFrom-Json
    [pscustomobject]@{
      PR = $id
      Stack = $pr.stack.number
      Size = $pr.stack.size
      Position = $pr.stack.position
      StackBase = $pr.stack.base.ref
      DirectBase = $pr.base.ref
      Head = $pr.head.ref
    }
  }
)
$rows | Sort-Object Position | Format-Table -AutoSize
```

Preserve the Stack endpoint's member order and verify it against each PR's position. All selected members must have the same Stack number, size, and target base, with unique contiguous positions. Do not merge from a manually guessed PR list.

## Inspect PR Readiness

```powershell
gh pr view 103 --json `
  number,title,state,isDraft,baseRefName,headRefName,headRefOid,mergeable,mergeStateStatus,reviewDecision,statusCheckRollup,autoMergeRequest,commits,url
```

Interpretation:

- `mergeable=MERGEABLE` proves Git can combine the refs, not that policy allows merging.
- `mergeStateStatus=UNSTABLE` can result from a failed optional check.
- `reviewDecision=REVIEW_REQUIRED` is a real blocker when the Stack target ruleset requires approval.
- A `COMMENTED` review is not an approval.
- A newer head SHA may invalidate an earlier approval when stale-review dismissal is enabled.

## Inspect Rulesets

```powershell
$branch = "main"
$encodedBranch = [Uri]::EscapeDataString($branch)

# Aggregated active repository and organization rules that apply to this branch.
gh api --paginate "repos/$owner/$repo/rules/branches/$encodedBranch"

# Classic branch protection is a separate policy surface. A 404 means this
# branch has no readable classic protection configuration.
gh api "repos/$owner/$repo/branches/$encodedBranch/protection"
```

Read the combined policy that targets the Stack base branch:

- required approval count
- stale review dismissal
- required review-thread resolution
- required status check contexts and integrations
- allowed merge methods
- linear-history requirement

Do not classify a failed optional check as blocking merely because it is visible. Conversely, do not ignore a required check because another aggregate check passed. Determine required contexts from the combined applicable branch policies rather than from job names or repository-specific assumptions.

## Inspect Reviews Against The Head

```powershell
$prNumber = 103
$pr = gh pr view $prNumber --json headRefOid,reviewDecision | ConvertFrom-Json
$reviews = @(
  gh api --paginate "repos/$owner/$repo/pulls/$prNumber/reviews" `
    --jq '.[] | {author: .user.login, state, submittedAt: .submitted_at, commit: .commit_id}' |
    ConvertFrom-Json
)

# COMMENTED does not revoke a prior decision. Select each reviewer's latest
# decisive, non-dismissed review.
$effective = @(
  $reviews |
    Where-Object { $_.state -in 'APPROVED', 'CHANGES_REQUESTED' } |
    Sort-Object submittedAt |
    Group-Object author |
    ForEach-Object { $_.Group | Select-Object -Last 1 }
)

"head=$($pr.headRefOid) decision=$($pr.reviewDecision)"
$effective | Format-Table author,state,submittedAt,commit -AutoSize
```

Use REST review `commit_id` for evidence. Some `gh` versions expose empty `latestReviews.commit.oid` values, so do not use that field to decide whether an approval is stale. A differing review commit is not automatically invalid: check `reviewDecision` and the repository's stale-review policy because GitHub may preserve reviews across some server-side Stack updates.

Ask for a fresh approval when the effective policy requires it. Do not submit an approval through the authenticated user's account unless the user explicitly authorizes that action and the account is allowed to review the PR.

Confirm that an approver satisfies the repository's permission rule:

```powershell
$login = "REVIEWER"
gh api "repos/$owner/$repo/collaborators/$login/permission"
```

For a rule requiring write access, do not treat an approval as qualifying until the permission response is `write`, `maintain`, or `admin`. If stale-review dismissal is enabled and the approval commit differs from the head, rely on the live policy decision and investigate why GitHub retained or dismissed it rather than assuming either outcome.

## Inspect Threads, Auto-Merge, And Merge Queue

```powershell
$query = @'
query($owner: String!, $repo: String!, $number: Int!, $endCursor: String) {
  repository(owner: $owner, name: $repo) {
    pullRequest(number: $number) {
      autoMergeRequest { enabledAt mergeMethod }
      isInMergeQueue
      isMergeQueueEnabled
      reviewThreads(first: 100, after: $endCursor) {
        nodes { isResolved isOutdated }
        pageInfo { hasNextPage endCursor }
      }
    }
  }
}
'@

$pages = @(
  gh api graphql --paginate -f query=$query `
    -F owner=$owner -F repo=$repo -F number=$prNumber |
    ConvertFrom-Json
)
$pull = $pages[0].data.repository.pullRequest
$threads = @($pages | ForEach-Object {
  $_.data.repository.pullRequest.reviewThreads.nodes
})

[pscustomobject]@{
  UnresolvedThreads = @($threads | Where-Object { -not $_.isResolved }).Count
  AutoMergeEnabled = $null -ne $pull.autoMergeRequest
  InMergeQueue = $pull.isInMergeQueue
  MergeQueueEnabled = $pull.isMergeQueueEnabled
}
```

Paginate review threads; the first 100 are not sufficient for a large review. A queued PR is not equivalent to an atomically merged Stack, and an auto-merge request can make a PR ineligible for Stack linking.

## Investigate A Reported False Approval Violation

A multi-member Stack merge has been observed to fail with this message even when the UI and API report every member as approved:

```text
Repository rule violations found

At least 1 approving review is required by reviewers with write access.

Stack merges are atomic, so nothing was merged.
```

Do not classify this as a suspected GitHub Stack Preview failure until all of these are true for every selected member:

1. The PR is open, non-draft, and still belongs to the expected Stack.
2. `reviewDecision` is `APPROVED`.
3. REST reviews contain a qualifying effective approval, and live `reviewDecision` is `APPROVED` under the current stale-review policy.
4. The approver currently has write, maintain, or admin permission.
5. Required checks pass and required review threads are resolved.
6. No push occurred after the approval when stale-review dismissal is active.

If all checks pass and the atomic request still returns the exact review violation, record the extension version and Stack membership, verify that every PR remains open, and stop retrying unchanged. This failure pattern has been tracked in `github/gh-stack#323`:

- `https://github.com/github/gh-stack/issues/323`

Stack support is a Preview and can change. Check the current issue status and official documentation before treating this historical failure pattern as an active product defect. The exact message alone is not proof; complete the readiness checks first.

Safe next actions, in order:

1. If the only approval came from the merge actor, ask a different write-qualified reviewer to approve the affected members, then re-run the full readiness check and retry the atomic merge once.
2. If every member already has a current approval from a distinct qualified reviewer, or that single retry returns the same violation, stop requesting more reviews. Treat another approval as a diagnostic attempt, not as a guaranteed workaround.
3. Wait for GitHub to fix the Preview behavior.
4. With explicit user approval, merge one Stack boundary at a time from bottom to top, re-checking bases, CI, reviews, and Stack membership after each result.

Never silently fall back to sequential `gh stack merge` or individual `gh pr merge`. The former loses whole-stack atomicity; the latter also bypasses the Stack-aware workflow.

## Diagnose Concurrent Or Completed Merges

GraphQL-backed `gh pr view` can temporarily return `mergeable=UNKNOWN` and `mergeStateStatus=UNKNOWN` while a Stack operation rewrites upper branches. The same symptom can persist after another actor has already merged the PR. Do not keep polling mergeability without also checking terminal state through REST:

```powershell
$pr = gh api "repos/$owner/$repo/pulls/$id" | ConvertFrom-Json
[pscustomobject]@{
  State = $pr.state
  Merged = $pr.merged
  MergedAt = $pr.merged_at
  MergeCommit = $pr.merge_commit_sha
  Base = $pr.base.ref
  Head = $pr.head.sha
}
```

Interpretation:

- `merged=true` and `state=closed`: stop mergeability polling; the PR is terminal even when GraphQL still reports `UNKNOWN`.
- `gh stack merge` says `already merged`: verify the REST fields and trunk SHA before moving to the next layer.
- The head changes after a lower merge: GitHub updated the Stack. Re-run readiness checks and wait for CI against the new head.
- The head or state changes while an operator is waiting: assume a concurrent actor or server-side Stack update; reconstruct the whole remaining chain before issuing another mutation.

Do not attempt to revert or rewrite a concurrent rebase merge merely to restore the intended squash method. Report the mismatch and preserve shared history unless the user explicitly authorizes a coordinated history rewrite.

## Verify Commit Ancestry

```powershell
git fetch --prune origin
git merge-base --is-ancestor origin/feature-core origin/feature-api
git merge-base --is-ancestor origin/feature-api origin/feature-host
git log --graph --oneline --decorate `
  origin/main `
  origin/feature-core `
  origin/feature-api `
  origin/feature-host
```

Stop when a declared parent is not an ancestor of its child. Resolve divergence before syncing, rebasing, or merging.

## Common Blockers

| Symptom                                                      | Meaning                                                          | Action                                                                            |
| ------------------------------------------------------------ | ---------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| `stack` is null                                              | Branch chain is not a GitHub Stack                               | Keep it manual or link it with explicit authorization                             |
| `REVIEW_REQUIRED`                                            | Required approval is absent or stale                             | Obtain a current approval                                                         |
| `BLOCKED`                                                    | A ruleset, review, thread, or queue condition is unmet           | Inspect rulesets and merge status details                                         |
| `UNSTABLE`                                                   | At least one check is non-successful                             | Compare failed checks with required contexts                                      |
| Stack merge fails atomically                                 | One selected member failed policy or merge evaluation            | Fix that member; do not fall back to partial merge automatically                  |
| Approval violation despite current write-qualified approvals | Possible Preview false rejection or changed policy evaluation    | Complete the checklist, verify current upstream behavior and confirm no PR merged |
| `UNKNOWN` persists after checks finish                       | Stack update or concurrent merge may have changed terminal state | Query REST `state` and `merged`, then reconstruct remaining members               |
| `already merged` from a fallback command                     | The target reached terminal state before this command            | Verify merge commit and trunk; do not retry                                       |
| Upper heads change after a lower merge                       | GitHub retargeted and rebased the remaining Stack                | Wait for fresh CI and re-check approvals on each new head                         |
| Plain rebase repeats merged commits                          | Lower layer was squash-merged                                    | Use `gh stack rebase` or explicit `git rebase --onto`                             |
| `sync` reports divergence                                    | Local and remote Stack compositions differ                       | Stop and choose a single source of truth interactively                            |
| Branch remains after merge                                   | Automatic deletion is disabled or Stack cleanup is separate      | Verify merge, then delete explicitly                                              |

## Report Format

Report only decisive state:

```text
Stack: #<number>, <size> members, base <branch>
Range: PR #<bottom> through PR #<top>
Ready: yes/no
Blockers: <required review/check/thread/conflict>
Operation: atomic squash merge / sync / rebase / manual fallback
Result: merged/enqueued/failed, trunk SHA, branch cleanup status
```
