# Rust Backend Crate Design Draft

## Status

This is a temporary design draft for repository structure evolution.

- Do not treat this file as an implementation commit plan.
- Do not commit this file unless explicitly requested later.
- The current goal is to clarify boundaries, not to force an immediate migration.

## Problem Statement

The current Rust layout mixes several responsibilities inside `src-tauri`:

- Tauri desktop entry and plugin wiring
- Tauri command exposure
- Headless HTTP runtime entry
- Core backend services and domain logic
- Plugin runtime and shared infrastructure

This creates two structural issues:

1. The crate named `src-tauri` is acting as both a host application and the main reusable backend library.
2. Shared backend logic is already reused by more than one entry point, but the boundary is implicit instead of explicit.

The repository is therefore already close to a multi-entry architecture, but the shared backend has not been formalized as its own crate.

## Design Goals

The target structure should satisfy these goals:

1. Keep desktop-host concerns isolated from reusable backend logic.
2. Allow Docker or headless HTTP entry points to depend on shared backend code without depending on desktop-only Tauri setup.
3. Avoid exploding the backend into too many small crates too early.
4. Preserve the current service and runtime layering where it is already becoming more modular.
5. Make future testing, CI, and dependency ownership clearer.

## Non-Goals

This draft does not aim to:

- redesign the full domain model
- split every module into an independent crate
- rewrite the frontend structure in the same migration
- force an immediate rename of every existing directory

## Recommended Direction

Use limited crate extraction, not aggressive micro-crate decomposition.

The recommended medium-term shape is:

```text
SeaLantern/
├── frontend/
│   ├── package.json
│   ├── index.html
│   └── src/
├── backend/
│   ├── core/
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── http/
│   │   ├── Cargo.toml
│   │   └── src/
│   └── tauri-host/
│       ├── Cargo.toml
│       ├── tauri.conf.json
│       └── src/
├── docker-entry/
│   ├── Cargo.toml
│   └── src/
├── docker/
├── docs/
└── Cargo.toml
```

If the team wants fewer top-level moves initially, an equivalent transitional form is acceptable:

```text
SeaLantern/
├── src-tauri/
│   ├── host-tauri/
│   └── backend-core/
```

However, this transitional form should be treated as a stepping stone, not the final structure.

## Crate Roles

### `backend/core`

This should hold reusable Rust backend logic that does not require desktop-host setup.

Likely contents:

- domain models currently under `models`
- server lifecycle and runtime management under `services/server`
- plugin runtime core and shared logic
- shared logging, path, and configuration utilities that are not Tauri-specific
- shared managers used by both desktop and headless modes

Constraints:

- avoid depending directly on tray, window, notification, or desktop-only Tauri plugins
- avoid embedding Tauri command registration here
- keep this crate usable from more than one host

### `backend/http`

This crate is optional in the first migration wave.

Use it only if the HTTP/headless runtime continues to grow as a separate adapter boundary.

Likely contents:

- HTTP adapter and command registry
- upload endpoints
- SSE/log streaming endpoints
- HTTP authentication and CORS configuration

Keep it separate only if that boundary improves clarity. Otherwise it can temporarily remain inside the host or under `backend/core` as an adapter module.

### `backend/tauri-host`

This should be the desktop host application crate.

Likely contents:

- `main.rs` and library entry for desktop mode
- Tauri setup and bootstrap
- tray integration
- desktop window behavior
- Tauri plugin wiring
- Tauri `generate_handler!` command registration

Rule:

- this crate depends on `backend/core`
- `backend/core` must not depend on this crate

### `docker-entry`

This should remain a very thin executable crate.

Likely behavior:

- parse Docker/headless startup shape if needed
- call into `backend/http` or `backend/core` host functions

Rule:

- keep this crate minimal
- do not move Docker image assets or Dockerfile logic into it

## What Should Stay Out of `backend/core`

These concerns should not be pulled into the shared backend crate unless there is a strong reason:

- tray handling
- desktop window styling
- window close behavior
- drag and drop events from Tauri windows
- Tauri plugin registration details
- desktop notification emission that is only meaningful in the desktop host

These are host concerns, not shared backend concerns.

## Likely Module Mapping

This is an approximate mapping from the current repository shape.

### Move toward `backend/core`

- `src-tauri/src/models`
- most of `src-tauri/src/services`
- most of `src-tauri/src/plugins/runtime`
- selected parts of `src-tauri/src/utils`
- shared runtime mode-independent logic

### Keep in desktop host crate

- `src-tauri/src/runtime/bootstrap.rs`
- `src-tauri/src/runtime/command_catalog.rs`
- `src-tauri/src/runtime/desktop_setup.rs`
- `src-tauri/src/runtime/desktop_shell.rs`
- Tauri-facing command adapter layer under `commands`

### Evaluate case by case

- `src-tauri/src/adapters/http`
- `src-tauri/src/runtime/mode.rs`
- `src-tauri/src/runtime/plugin_bridge.rs`
- any utility module that currently mixes host calls with core logic

## Migration Principles

### Principle 1: Extract shared code before renaming everything

Do not start with a large directory rename only for aesthetics.

First make the shared backend explicit. Then rename or relocate hosts around it.

### Principle 2: Preserve runtime boundaries that already exist

The repository already has meaningful backend/runtime seams, especially around:

- server manager and runtime control
- local helper orchestration
- Docker runtime handling
- plugin runtime subsystems

Those seams should be used when defining crate boundaries.

### Principle 3: Prefer one new shared crate over many small crates

The first useful extraction is a single `backend/core` crate.

Only add more crates when one of these becomes true:

- compile times become dominated by a stable subsystem
- dependency isolation becomes necessary
- a subsystem clearly serves multiple hosts with separate release cadence
- the API boundary is already stable and coherent

## Suggested Migration Phases

### Phase 0: Freeze the design boundary

Deliverables:

- agree on crate roles
- agree on naming
- identify forbidden dependencies from core to host

### Phase 1: Extract a shared backend crate

Goal:

- introduce `backend/core` without changing runtime behavior

Steps:

1. Create new crate.
2. Move pure models and a small shared service slice first.
3. Re-export or adapt imports temporarily to reduce churn.
4. Keep `src-tauri` as host while migrating internal modules incrementally.

### Phase 2: Narrow the Tauri host crate

Goal:

- make the desktop host mostly wiring and command adapter code

Steps:

1. Move reusable services and managers behind `backend/core`.
2. Keep Tauri command functions thin.
3. Keep desktop bootstrap and plugin setup local to the host.

### Phase 3: Rewire Docker/headless entry

Goal:

- make `docker-entry` depend on shared backend crates directly

Steps:

1. Replace dependency on the fat host crate with dependency on `backend/core` and optional `backend/http`.
2. Keep entry logic shallow.

### Phase 4: Optional repository-level naming cleanup

Only after boundaries are working:

- move frontend to `frontend/`
- rename host crate directories if still needed
- simplify scripts and CI paths

## Risks

### Risk 1: Circular dependency pressure

If Tauri-facing command or plugin setup logic has leaked into services, extraction may reveal hidden host dependencies.

Response:

- introduce small adapter traits or host callbacks only where necessary
- do not solve every leak with a new crate

### Risk 2: Over-splitting too early

Too many crates will increase:

- manifest maintenance
- feature wiring complexity
- import churn
- test setup complexity

Response:

- start with one shared crate
- add more only when a clear boundary has proven itself

### Risk 3: Mixed host assumptions inside shared utilities

Some utility modules may look generic but still assume:

- desktop environment
- Tauri app handle availability
- notification or window access

Response:

- audit utility modules before moving them wholesale
- split mixed modules instead of exporting host assumptions into core

## Decision Summary

The Rust backend should be crate-ized, but selectively.

Recommended first target:

- one shared reusable backend crate
- one desktop host crate
- one thin Docker/headless entry crate

Not recommended right now:

- turning every subsystem into its own crate
- moving Docker build assets into Rust backend crates
- treating `src-tauri` as the long-term home of both host wiring and all backend implementation

## Immediate Next Step

If this draft is accepted, the next planning artifact should be a concrete extraction map:

- modules that can move immediately
- modules blocked by Tauri dependencies
- expected new crate dependency graph
- validation commands for each migration phase