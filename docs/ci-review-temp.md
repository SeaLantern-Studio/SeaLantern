# CI Review Temp

## Goal

Stabilize and streamline the GitHub CI workflows under `.github/workflows`.

## Problems Confirmed

- Main CI does not run Rust tests.
- Main CI does not run frontend type checking even though `build:check` exists; current branch still has many existing TS issues, so this should be enabled only after a dedicated cleanup.
- `paths-filter` misses real build inputs such as `docker-entry/**`, `.cargo/**`, repo scripts, formatter/linter config, and workflow files.
- Several workflows lack `concurrency` and waste runner time on superseded pushes.
- Docker check does full cold builds without cache and performs unnecessary cleanup.
- The issue gate is too strict for low-risk or maintenance PRs.

## Optimization Scope

- Update `check.yml` to improve coverage and avoid unnecessary runners.
- Update `docker-check.yml` to reduce wasted runtime and add cancellation.
- Relax `issue-check.yml` for practical development flow while preserving issue linkage.
- Tighten release-side workflows so tags and release fan-out jobs are less likely to publish from an unverified state.
- Keep release-side workflows mostly intact unless they directly block CI efficiency.

## Acceptance

- Rust backend CI runs format, clippy, and tests when backend-related files change.
- Frontend CI runs format, lint, and build when frontend-related files change.
- Frontend type-checking remains a follow-up item after existing TypeScript errors are cleaned up.
- CI jobs are skipped at the job level when no relevant files changed.
- PR updates cancel outdated CI runs where appropriate.
- Docker validation remains present but wastes less runner time.
- Release workflows should reject tags whose target commit has not passed the main quality workflow.
