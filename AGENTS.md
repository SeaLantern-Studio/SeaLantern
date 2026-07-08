# SeaLantern Maintainer Notes

This file is a compact maintainer entrypoint. Keep detailed design, roadmap, and
project direction outside this file unless the detail directly affects safe edits.

## Repository Shape

- `frontend/`: Vue, TypeScript, Tauri command callers, UI state, and user flows.
- `backend/`: Rust workspace. Shared logic lives in `*-core` crates; host wiring lives in
  `backend/tauri-host`.
- `docs/`: user, contributor, design, and API documentation.
- `scripts/`: repo-level build, Tauri, notice, and version helpers.
- `src-tauri/`: legacy or auxiliary Tauri material. Check the active backend path before editing it.

## Maintenance Rules

- Inspect the live tree before making claims about structure or behavior.
- Keep changes small and auditable.
- Prefer existing local patterns over new abstractions.
- Preserve frontend-backend command names, payloads, and response shapes unless the task
  explicitly changes the contract.

## Planning Rules

- Start from facts: current code, documented workflows, known constraints, and reproducible issues.
- Define the boundary before proposing work: target area, success criteria, non-goals, and hard limits.
- If facts are missing, verify them from the repository first; ask only for decisions that change direction.
- For larger changes, compare one to three practical options by scope, cost, risk, migration impact,
  maintainability, and rollback path.
- Recommend a direction instead of only listing choices, and explain the tradeoff in concrete terms.
- Split execution into reviewable phases with a clear result and validation step for each phase.
- Do not let planning expand the task without a reason tied to future maintenance or user-visible value.

## Compatibility Surfaces

- Frontend command wrappers: `frontend/src/api/`.
- Backend command registration: `backend/tauri-host/src/runtime/command_catalog.rs`.
- Backend command implementations: `backend/tauri-host/src/commands/`.
- Tauri launch path: `pnpm --dir frontend run tauri:dev`.
- Repo launcher script: `scripts/tauri.mjs`.

When changing a command contract, read the frontend caller first, then the backend registration
and implementation.

## Validation

Use the smallest check that proves the touched area. Baselines:

- Frontend:
  - `pnpm --dir frontend run lint`
  - `pnpm --dir frontend run build:check`
  - `pnpm --dir frontend run fmt:check`
- Backend:
  - `cargo fmt --all -- --check`
  - `cargo check --workspace`
  - `cargo clippy --workspace -- -D warnings`

For docs-only edits, validate links and affected paths.

## Local Notes

- Do not assume old `frontend/next-src` paths exist. Current entries are
  `frontend/src/main.ts` and `frontend/src/main.next.ts`.
- Low-memory development paths are opt-in only; keep the documented Tauri dev path as the default.
- Avoid adding generated-looking metadata to docs: front matter, author tags, update stamps, and
  template instructions should not be introduced unless a real docs tool requires them.
