---
module-name: sea-lantern-runtime
update-time: 2026-07-05
description: Shared runtime utilities for logging, headless HTTP hosting, status parsing, path helpers, panic reporting, and Tokio bootstrap.
tag: ["runtime-core", "logging", "headless-http", "path-utils", "tokio"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`sea-lantern-runtime` is the shared backend utility crate.

This crate should own:

- headless HTTP runtime bootstrap helpers
- shared command-registry and HTTP dispatch helpers
- shared logging and log formatting helpers
- shared panic-report and path helpers
- runtime-mode, status, port-usage, and Tokio bootstrap utilities

This crate should not own:

- Tauri commands
- Minecraft server business logic
- frontend contracts specific to one page

## Module Entry

- `runtime-core/`
  - `src/`
    - `lib.rs`
    - `headless_http.rs`
    - `headless_runtime.rs`
    - `http_bind.rs`
    - `http_dispatch.rs`
    - `logging.rs`
    - `panic_report.rs`
    - `panic_report_pathing.rs`
    - `panic_report_system_info.rs`
    - `path_utils.rs`
    - `port_usage.rs`
    - `runtime_mode.rs`
    - `server_status.rs`
    - `tokio_runtime.rs`
  - `Cargo.toml`

## Key Files

[`src/lib.rs`](src/lib.rs): Public re-export surface for shared runtime helpers.
- `run_tokio_service` -> `fn(...)`: Runs a Tokio service with shared bootstrap behavior.
- `default_headless_http_config` / `prepare_headless_http_listener`: Shared headless HTTP setup.
- Re-exports logging, path, dispatch, runtime-mode, status, and port-usage helpers.

[`src/headless_http.rs`](src/headless_http.rs): Headless HTTP config and listener preparation.
- Used by Docker and other headless hosts.

[`src/http_bind.rs`](src/http_bind.rs): HTTP bind-host and bind-address resolution helpers.
- Owns shared bind normalization and validation behavior.

[`src/http_dispatch.rs`](src/http_dispatch.rs): Shared command-registry and HTTP dispatch helpers.
- Reusable registry and dispatch flow for headless command surfaces.

[`src/logging.rs`](src/logging.rs): Shared logging, capture, and formatting helpers.
- Runtime-wide log support.

[`src/panic_report.rs`](src/panic_report.rs): Shared panic-hook initialization.
- Panic-report bootstrap boundary.

[`src/path_utils.rs`](src/path_utils.rs): App-data, path, executable, and startup-file helpers.
- Shared filesystem and path support.

[`src/port_usage.rs`](src/port_usage.rs): TCP port-usage helpers.
- Shared readiness and environment checks.

[`src/runtime_mode.rs`](src/runtime_mode.rs): Shared runtime-mode enum and helpers.
- Keeps runtime-mode interpretation consistent across host paths.

[`src/server_status.rs`](src/server_status.rs): Shared status-detail parsing and readiness helpers.
- Used to interpret runtime state consistently.

[`src/tokio_runtime.rs`](src/tokio_runtime.rs): Shared Tokio runtime creation helpers.
- Bootstrap boundary.

## Stable Boundaries

- Prefer adding reusable runtime helpers here instead of duplicating them in host crates.
- Keep this crate focused on infrastructure concerns, not app-specific orchestration.
- Treat exported helpers as shared contracts used across multiple crates.

## Change Guidance For Agents

- Do not move app-specific business logic into this crate.
- If a helper is used by more than one host path, prefer stabilizing it here.
- Keep panic-report, logging, and headless-runtime helpers decoupled from Tauri-only concerns.

## Validation Checklist

- Run this crate's tests if shared runtime helpers, logging, bind resolution, status parsing, or Tokio bootstrap behavior changed.
- Re-check `tauri-host` and `docker-entry` consumers if exported helper names or public behavior changed.
