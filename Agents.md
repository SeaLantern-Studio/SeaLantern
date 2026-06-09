# SeaLantern Agents Guide

## Scope

This file applies to work under `E:\repo\SeaLantern`.

## Working Defaults

- Inspect the live repository before making claims about behavior, structure, or capabilities.
- Keep changes minimal and auditable unless the task is explicitly exploratory or planning-heavy.
- Prefer the smallest meaningful validation step that matches the requested scope.
- Keep planning tied to the current codebase, current constraints, and clear next actions.

## Repo Map

- `backend/`: Rust backend, host logic, and most implementation-critical behavior.
- `frontend/`: UI and user-facing flows.
- `docs/`: project documentation and supporting design notes.
- `docker/`: container and local environment support.
- `scripts/`: helper scripts used during development or packaging.
- `src-tauri/`: legacy or auxiliary Tauri-related material; verify whether the active code path is now in `backend/` before editing.

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
