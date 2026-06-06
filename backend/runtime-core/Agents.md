---
module-name: sea-lantern-runtime
update-time: 2026-06-06
description: Shared runtime utilities for logging, headless HTTP hosting, status parsing, path helpers, and Tokio bootstrap.
tag: ["runtime-core", "logging", "headless-http", "path-utils", "tokio"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Runtime Core

`sea-lantern-runtime` is the shared backend utility crate.

This crate should own:

- headless HTTP runtime bootstrap helpers
- shared logging and log formatting helpers
- shared path and app-data helpers
- runtime-status parsing helpers
- shared Tokio runtime bootstrap utilities

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
    - `http_dispatch.rs`
    - `logging.rs`
    - `path_utils.rs`
    - `server_status.rs`
    - `tokio_runtime.rs`
  - `Cargo.toml`

## Module Info

[`backend/runtime-core/src/lib.rs`](../runtime-core/src/lib.rs): Public re-export surface for shared runtime helpers. | Main entry for consumers.
- `run_tokio_service` -> `fn(...)`: Runs a Tokio service with shared bootstrap behavior.
- `default_headless_http_config` / `prepare_headless_http_listener`: Shared headless HTTP setup.

[`backend/runtime-core/src/headless_http.rs`](../runtime-core/src/headless_http.rs): Headless HTTP config and listener preparation. | Used by Docker and other headless hosts.

[`backend/runtime-core/src/http_dispatch.rs`](../runtime-core/src/http_dispatch.rs): Shared command-registry and HTTP dispatch helpers. | Reusable registry and dispatch flow.

[`backend/runtime-core/src/logging.rs`](../runtime-core/src/logging.rs): Shared logging, capture, and formatting helpers. | Runtime-wide log support.

[`backend/runtime-core/src/path_utils.rs`](../runtime-core/src/path_utils.rs): App-data, path, executable, and startup-file helpers. | Shared filesystem and path support.

[`backend/runtime-core/src/server_status.rs`](../runtime-core/src/server_status.rs): Shared status-detail parsing and readiness helpers. | Used to interpret runtime state consistently.

[`backend/runtime-core/src/tokio_runtime.rs`](../runtime-core/src/tokio_runtime.rs): Shared Tokio runtime creation helpers. | Bootstrap boundary.

## Change Guidance For Agents

- Prefer adding reusable runtime helpers here instead of duplicating them in host crates.
- Do not move app-specific business logic into this crate.
- Treat exported helpers as shared contracts used across multiple crates.
