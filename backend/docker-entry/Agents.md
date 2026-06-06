---
module-name: docker-entry
update-time: 2026-06-06
description: Thin executable entry crate that runs SeaLantern in headless HTTP mode for Docker environments.
tag: ["docker", "entrypoint", "headless-http"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Docker Entry

`docker-entry` is a very small executable crate.

Its only job is to call the headless HTTP runtime exposed by `sea-lantern`.

Keep this crate thin. Do not move Docker runtime logic, HTTP routing logic, or business logic into this crate.

## Module Entry

- `docker-entry/`
  - `src/`
    - `main.rs`
  - `Cargo.toml`

## Module Info

[`backend/docker-entry/src/main.rs`](../docker-entry/src/main.rs): Docker executable entrypoint for SeaLantern headless mode. | Thin wrapper only.
- `main` -> `fn(sea_lantern_lib::run_headless_http())`: Starts the headless HTTP host.

[`backend/docker-entry/Cargo.toml`](../docker-entry/Cargo.toml): Declares the executable crate and depends on `sea-lantern` with the `docker` feature. | Dependency boundary.

## Change Guidance For Agents

- Keep `main.rs` minimal.
- If behavior changes are needed, prefer editing `backend/tauri-host` or `backend/runtime-core` instead.
- Only change this crate when the executable entry boundary itself needs to change.
