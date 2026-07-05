---
module-name: backend-workspace
update-time: 2026-07-05
description: Backend workspace entry for SeaLantern Rust crates, including host runtime, shared core crates, and Docker/headless entry surfaces.
tag: ["backend", "workspace", "rust", "crate-map", "entry-guide"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Scope

This file is the entry guide for the Rust backend workspace under `backend/`.

Use it to decide:

- which crate should own a backend change
- whether logic belongs in a shared `*-core` crate or in `tauri-host`
- where Docker, headless HTTP, update, plugin, startup, log, and i18n behavior currently live

## Workspace Entry

- Workspace manifest: [`../Cargo.toml`](../Cargo.toml)
- Main host crate: [`tauri-host/`](tauri-host)
- Docker executable entry: [`docker-entry/`](docker-entry)
- Shared backend crates: the remaining `*-core` and utility crates under `backend/`

The Rust workspace is rooted at the repository-level [`Cargo.toml`](../Cargo.toml), which lists all backend crates under `backend/`.

## Working Defaults

- Inspect the live crate before making claims about ownership or behavior.
- Prefer changing a shared `*-core` crate when the rule is reusable across host paths.
- Prefer changing `tauri-host` when the task is orchestration, command exposure, persistence, plugin runtime wiring, or OS host integration.
- Keep `docker-entry` thin; runtime behavior belongs in `tauri-host` or `runtime-core`.
- When a frontend-backend contract is involved, check the frontend caller first and then the command registration and implementation in `tauri-host`.

## Module Map

### Host and entry crates

- [`tauri-host/Agents.md`](tauri-host/Agents.md): Main desktop/headless host runtime, command surface, services, plugin runtime, and host integration.
- [`docker-entry/Agents.md`](docker-entry/Agents.md): Thin Docker/headless HTTP executable entrypoint.

### Shared infrastructure crates

- [`runtime-core/Agents.md`](runtime-core/Agents.md): Shared runtime bootstrap, logging, headless HTTP helpers, bind/dispatch helpers, path utilities, panic reporting, status parsing, and Tokio helpers.
- [`event-core/Agents.md`](event-core/Agents.md): Shared app/server event envelopes, subscriptions, event manager behavior, and consumer registry DTOs.
- [`i18n-core/Agents.md`](i18n-core/Agents.md): Backend i18n service, embedded locale assets, locale callbacks, and plugin translation registration.

### Startup, install, and runtime-shape crates

- [`server-config-core/Agents.md`](server-config-core/Agents.md): Shared startup-config, JVM-arg, CPU-policy, and `server.properties` rules.
- [`server-installer-core/Agents.md`](server-installer-core/Agents.md): Archive extraction, core detection, taxonomy mapping, and MC-version inference.
- [`server-local-setup-core/Agents.md`](server-local-setup-core/Agents.md): Local setup helpers, startup-mode inference, Java env/path handling, and launch preview helpers.
- [`server-startup-scan-core/Agents.md`](server-startup-scan-core/Agents.md): Startup-candidate scanning for folders and archives.
- [`java-installer-core/Agents.md`](java-installer-core/Agents.md): Java runtime download, extraction, and install-finalization helpers.
- [`docker-core/Agents.md`](docker-core/Agents.md): Shared Docker launch-shape, preview, JVM env synthesis, and Docker command interpretation rules.

### Content, plugin, and metadata crates

- [`server-plugin-core/Agents.md`](server-plugin-core/Agents.md): Minecraft server plugin JAR parsing, extension target resolution, and plugin-config listing.
- [`plugin-trust-core/Agents.md`](plugin-trust-core/Agents.md): Plugin permission metadata, trust assessment, install metadata, trusted catalog parsing, and enable-policy logic.
- [`lua-runtime-core/Agents.md`](lua-runtime-core/Agents.md): Shared runtime trait boundary for Lua plugins.
- [`server-download-links-core/Agents.md`](server-download-links-core/Agents.md): Parser for server-download link payloads.
- [`starter-links-core/Agents.md`](starter-links-core/Agents.md): Cached starter-installer link loading, validation, and URL resolution.
- [`update-core/Agents.md`](update-core/Agents.md): Update checking, download, pending-install, and install-launch planning.

### Logging crate

- [`server-log-core/Agents.md`](server-log-core/Agents.md): Server log persistence, history reads, output-reader hooks, and structured event mapping.

## Ownership Heuristics

- If the rule is pure and reusable across desktop, Docker, preview, or scan flows, put it in a shared core crate.
- If the change is about command names, command payloads, manager orchestration, plugin runtime execution, or host persistence, put it in `tauri-host`.
- If the change only affects process entry, keep it in `docker-entry` or `tauri-host/main.rs`, not in a shared core crate.
- If the change needs to stay consistent between actual runtime behavior and preview behavior, prefer the core crate that already owns the normalized shape.

## Validation Defaults

- Backend baseline from the repo guide:
  - `cargo fmt --all -- --check`
  - `cargo check --workspace`
  - `cargo clippy --workspace -- -D warnings`
- For scoped crate-only work, a targeted check such as `cargo check --manifest-path backend/<crate>/Cargo.toml` is acceptable when the task does not justify a full workspace pass.
- For local desktop integration, prefer the documented launcher path through the frontend script layer rather than reconstructing Tauri CLI calls manually.

## Build and run entry notes

- The workspace manifest is at [`../Cargo.toml`](../Cargo.toml).
- The repo-level launcher script [`../scripts/tauri.mjs`](../scripts/tauri.mjs) runs Tauri from `backend/tauri-host` while reusing the frontend CLI install.
- Scenario aliases such as desktop full/min are configured in `scripts/tauri.mjs`, not in a backend-local script.

## Change Guidance For Agents

- Read the crate-local `Agents.md` before making a non-trivial change inside that crate.
- Keep docs lightweight but factual; update them when crate boundaries or important entrypoints move.
- Do not assume old `next-src` or old backend wiring without checking the live repo.
- When multiple crates are touched, prefer putting the shared rule in one owner crate and adapting the other crate around it.
