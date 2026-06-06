---
module-name: sea-lantern
update-time: 2026-06-06
description: Main Tauri host crate for SeaLantern desktop runtime, command surface, plugins, services, and host integration.
tag: ["tauri-host", "commands", "services", "plugins", "host-runtime"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Tauri Host

`sea-lantern` is the main host crate for the desktop app and headless-host entry surface.

This crate should own:

- Tauri app bootstrap
- Tauri command registration and host adapters
- application services, managers, and runtime orchestration
- plugin runtime integration
- app-specific models, hardcoded data, and host utilities

This crate should not absorb shared pure logic that belongs in backend core crates such as:

- `server-config-core`
- `docker-core`
- `server-local-setup-core`
- `server-installer-core`
- `runtime-core`

## Module Entry

- `tauri-host/`
  - `src/`
    - `lib.rs`
    - `main.rs`
    - `commands/`
    - `services/`
    - `runtime/`
    - `plugins/`
    - `models/`
    - `utils/`
  - `Cargo.toml`

## Module Info

[`backend/tauri-host/src/lib.rs`](../tauri-host/src/lib.rs): Main library entry for SeaLantern host runtime. | Host bootstrap surface.
- `run` -> `fn()`: Starts the desktop and Tauri host runtime.
- `run_headless_http` -> `fn()`: Starts the headless HTTP host path used by Docker and external hosts.

[`backend/tauri-host/src/main.rs`](../tauri-host/src/main.rs): Desktop executable entrypoint. | Thin binary wrapper.
- `main` -> `fn(sea_lantern_lib::run())`: Starts the main desktop host.

[`backend/tauri-host/src/commands/`](../tauri-host/src/commands): Tauri command surface consumed by frontend code and host adapters. | Host API layer.

[`backend/tauri-host/src/services/`](../tauri-host/src/services): Host-side business services, managers, and runtime orchestration. | Main app logic layer.

[`backend/tauri-host/src/plugins/`](../tauri-host/src/plugins): SeaLantern plugin runtime, plugin APIs, and plugin manager logic. | Host plugin subsystem.

[`backend/tauri-host/src/runtime/`](../tauri-host/src/runtime): High-level runtime bootstrap and command catalog wiring. | App bootstrap details.

[`backend/tauri-host/src/models/`](../tauri-host/src/models): Host-facing data models and serialization payloads. | App model layer.

[`backend/tauri-host/src/utils/`](../tauri-host/src/utils): Host utility modules, CLI helpers, and adapters. | App support layer.

## Stable Boundaries

- Shared reusable rules belong in backend core crates first.
- `tauri-host` should mostly adapt app models and host state into those core crates.
- Keep crate-local logic focused on orchestration, persistence, command exposure, plugin runtime, and OS host integration.

## Important Slice: Startup And Docker Runtime

This crate contains a stabilized startup/runtime slice centered on:

- `src/services/server/manager/startup_support.rs`
- `src/services/server/manager/cpu_policy.rs`
- `src/services/server/manager/runtime_start/local_launch_detail.rs`
- `src/services/server/manager/runtime_start/launch/`
- `src/services/server/runtime/docker_itzg/helpers.rs`

When touching startup preview, local managed JVM args, Docker launch preview, or Java-path persistence, check those files together and prefer shared-rule changes in `server-config-core`, `docker-core`, or `server-local-setup-core` first.

Preserve the current launch boundary unless the user explicitly asks to change behavior:

- jar and starter use direct managed JVM launch
- bat writes managed JVM args to file and keeps its current inline-env path
- sh, ps1, and custom use process-env Java injection

## Change Guidance For Agents

- Do not re-implement shared startup or Docker rules in this crate when a core crate already owns them.
- Keep `src/main.rs` and `src/lib.rs` thin.
- When changing local launch behavior, check both preview and actual runtime paths.
- When changing plugin-runtime or command-surface behavior, treat it as a host API change.

## Frontend Contract Notes

- Many frontend flows depend on Tauri command payload stability.
- Changes to command return shapes should be treated as API changes and verified against frontend consumers.

## Validation Checklist

- Run relevant `tauri-host` tests when command payloads, host orchestration, plugin runtime, startup preview, or Docker runtime behavior changes.
- Re-check core crate tests too if a host change required moving or changing shared rules.
