---
module-name: docker-entry
update-time: 2026-07-05
description: Thin executable entry crate that runs SeaLantern in headless HTTP mode for Docker environments.
tag: ["docker", "entrypoint", "headless-http"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`docker-entry` is the Docker-specific executable entry crate.

This crate should own:

- the thin binary entrypoint used for Docker and headless HTTP startup
- the dependency edge from the Docker executable to `sea-lantern`

This crate should not own:

- Docker runtime logic
- HTTP routing logic
- business services
- shared runtime helpers

## Module Entry

- `docker-entry/`
  - `src/`
    - `main.rs`
  - `Cargo.toml`

## Key Files

[`src/main.rs`](src/main.rs): Docker executable entrypoint for SeaLantern headless mode.
- `main` -> `fn(sea_lantern_lib::run_headless_http())`: Starts the headless HTTP host.

[`Cargo.toml`](Cargo.toml): Declares the executable crate and depends on `sea-lantern` with the `docker` feature.
- Defines the crate's only meaningful dependency boundary.

## Stable Boundaries

- Keep this crate extremely thin.
- If runtime behavior changes are needed, prefer editing `tauri-host` or `runtime-core` instead.
- The binary should stay as a wrapper around `sea_lantern_lib::run_headless_http()` unless the entry boundary itself changes.

## Change Guidance For Agents

- Keep `main.rs` minimal.
- Do not move Docker policy or headless HTTP setup into this crate.
- Only change this crate when the executable entry boundary itself needs to change.

## Validation Checklist

- Run a targeted `cargo check --manifest-path backend/docker-entry/Cargo.toml` if this crate changed.
- Re-check Docker or headless-host startup from `tauri-host` if the entry function or feature wiring changed.
