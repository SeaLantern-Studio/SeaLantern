# SeaLantern Agents Guide

## Working Defaults

- Inspect the live repository before making claims about behavior, structure, or capabilities.
- Keep changes minimal and auditable unless the task is explicitly exploratory or planning-heavy.
- Prefer the smallest meaningful validation step that matches the requested scope.
- Keep planning tied to the current codebase, current constraints, and clear next actions.
- Treat `README.md` as the default verification entry: prefer the documented commands there before inventing custom check flows, and explain briefly when you intentionally use a narrower validation pass.
- On Windows PowerShell, prefer `rg` for search and `apply_patch` for edits; if the wrapper mangles multi-line patches, call the bundled `codex.exe --codex-run-as-apply-patch` target directly.
- Keep scratch outputs under `tmp/output/` unless the user names another path. Treat them as disposable review artifacts, not source-of-truth docs.
- When reviewing or changing a frontend-backend contract, treat the live frontend call shape as the first compatibility surface to preserve, then verify the backend command/API still matches it.

## Repo Map

- `backend/`: Rust workspace. Shared pure logic lives in `*-core` crates; `backend/tauri-host/` is the orchestration layer and the active desktop/headless host entry.
- `frontend/`: UI and user-facing flows. Main runtime entry is `frontend/src/main.ts`; command wrappers live under `frontend/src/api/`; frontend dev helpers live under `frontend/scripts/`.
- `docs/`: project documentation and supporting design notes.
- `docker/`: container and local environment support.
- `scripts/`: repo-level helper scripts used during development or packaging. `scripts/tauri.mjs` launches Tauri from `backend/tauri-host` while reusing the frontend CLI install.
- `src-tauri/`: legacy or auxiliary Tauri-related material; verify whether the active code path is now in `backend/` before editing.

## Default Review Entry Points

- Frontend review: start from `frontend/src/main.ts`, `frontend/src/main.next.ts`, and the touched modules under `frontend/src/api/`, `stores/`, `router/`, `pages/`, `components/`, or `services/`.
- Backend review: start from `backend/tauri-host/src/lib.rs`, `backend/tauri-host/src/runtime/`, `backend/tauri-host/src/commands/`, and then follow the touched `backend/*-core` crate boundaries.
- Frontend-backend contract review: read the caller in `frontend/src/api/*.ts` first, then verify the command registration surface in `backend/tauri-host/src/runtime/command_catalog.rs` and the corresponding implementation under `backend/tauri-host/src/commands/`.
- Build/run entry review: prefer `frontend/package.json`, `frontend/scripts/dev.mjs`, and `scripts/tauri.mjs` before assuming how dev startup or preview modes work.

## Validation Defaults

- Use the smallest validation chain that proves the touched area, but keep README commands as the default baseline.
- Frontend baseline:
  - `pnpm --dir frontend run lint`
  - `pnpm --dir frontend run build:check`
  - `pnpm --dir frontend run fmt:check`
- Backend baseline:
  - `cargo fmt --all -- --check`
  - `cargo check --workspace`
  - `cargo clippy --workspace -- -D warnings`
- For scoped backend changes inside one crate, it is acceptable to tighten validation to a targeted command such as `cargo check --manifest-path backend/tauri-host/Cargo.toml` when the task does not justify a full workspace pass.
- For local desktop integration checks, prefer the documented launcher path `pnpm --dir frontend run tauri:dev` instead of reconstructing the Tauri command manually.

## Working Constraints For Agents

- Do not assume old `frontend/next-src` paths still exist. Check the live tree first; current next-shell entry is wired through `frontend/src/main.ts` and `frontend/src/main.next.ts`.
- Do not treat temporary notes under `tmp/` as canonical requirements unless the user points to them explicitly. Once pointed to, treat that file as the task contract for the current turn.
- When the frontend already defines command names, request payload shape, or response typing in `frontend/src/api/`, preserve that contract unless the task explicitly reopens it.
- When changing backend commands, verify both the registration point and the frontend caller/import surface; do not stop at only one side.
- Prefer repo-local scripts and documented commands over ad hoc shell compositions, especially for dev startup, Tauri launch, i18n sync/check, and versioning tasks.
- Treat low-memory development guidance as opt-in only. Do not replace the default `pnpm --dir frontend run tauri:dev` path or the README workspace validation baseline unless the task explicitly asks for low-memory-specific changes.

## Local Skills

### `sl-planning`

- Entry: `.skill/sl-planning/SKILL.md`
- Agent metadata: `.skill/sl-planning/agents/openai.yaml`
- Use when: planning architecture, feature roadmap, module boundaries, migration steps, technical options, phased execution, or other project-planning tasks.
- Role: turn a planning request into a structured result based on facts, constraints, tradeoffs, recommended direction, phased steps, risks, and validation.
- Expected output: current state, problem statement, goals and non-goals, one to three options, comparison, recommendation, implementation phases, and next actions.

## How To Use The Skill

- Use `sl-planning` when the task is primarily about deciding what to build, how to evolve the project, or how to break a change into stages.
- Do not use it as a substitute for direct implementation when the user clearly wants code changes now.
- Before relying on the skill, inspect the relevant code, docs, config, or tests so the plan is grounded in repository reality.

## Skill Entry Summary

If a request mentions architecture planning, future direction, roadmap, refactor strategy, migration design, or phased rollout, start from `.skill/sl-planning/SKILL.md` and follow its workflow.
