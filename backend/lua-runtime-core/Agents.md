---
module-name: sea-lantern-lua-runtime-core
update-time: 2026-07-05
description: Host-facing runtime API boundary for SeaLantern Lua plugins, including shared host, plugin, and i18n traits.
tag: ["lua-runtime-core", "plugin-runtime", "host-api", "i18n-bridge", "runtime-boundary"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`sea-lantern-lua-runtime-core` is the shared runtime-boundary crate for Lua plugins.

This crate should own:

- shared trait definitions that the Lua runtime depends on
- the host/plugin/i18n API contract exposed to plugin runtime code
- global installation and access to the runtime host API handle
- simple shared DTOs needed by the runtime bridge

This crate should not own:

- Tauri command handlers
- plugin manifest trust policy
- concrete `mlua` execution wiring
- frontend component rendering logic

## Module Entry

- `lua-runtime-core/`
  - `src/`
    - `lib.rs`
    - `host.rs`
  - `Cargo.toml`

## Key Files

[`src/lib.rs`](src/lib.rs): Public entry for the runtime boundary.
- Re-exports the `host` module.

[`src/host.rs`](src/host.rs): Shared runtime host contract for Lua plugin execution.
- `RuntimeI18nApi`: Shared translation and locale API visible to runtime code.
- `RuntimePluginApi`: Shared plugin-to-host API for cross-plugin calls, UI events, logs, and element responses.
- `RuntimeHostApi`: Root host contract that bundles i18n and plugin APIs.
- `RuntimeComponentEntry`: Shared component metadata entry.
- `install_runtime_host_api` / `runtime_host_api`: Global install-and-access boundary for the runtime host API.

## Stable Boundaries

- Keep this crate as a narrow interface layer between host code and Lua runtime code.
- Trait signatures here are effectively runtime contracts; changing them usually requires coordinated updates in the host implementation and plugin runtime.
- Avoid leaking concrete host implementation details into this crate.

## Change Guidance For Agents

- Add new runtime bridge capabilities here only when they are truly shared runtime contracts.
- Prefer capability-focused trait methods over exposing broad host internals.
- If you change callback or payload semantics, verify all host implementations and runtime callers together.

## Validation Checklist

- Re-check the plugin runtime host implementation in `tauri-host` if trait signatures or event payload shapes changed.
- Run any runtime bridge tests that cover host installation, API calls, or locale access.
