---
module-name: sea-lantern-server-config-core
update-time: 2026-06-05
description: Shared startup-config, JVM-arg, CPU-policy, and server.properties rules for SeaLantern.
tag: ["startup-config", "jvm-args", "cpu-policy", "server-properties"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`sea-lantern-server-config-core` is the pure rules crate for server startup configuration.

This crate should own:

- startup config file read/write rules for `SeaLantern/config.toml` and legacy `SL.json`
- effective startup config resolution: instance config -> runtime config -> app defaults
- managed JVM argument synthesis for local startup flows
- shared CPU policy parsing and normalization rules
- `server.properties` read/write helpers used by higher-level services

This crate should not own:

- Tauri commands
- process spawning
- platform-specific CPU affinity application
- Docker CLI execution
- UI-specific validation text or view logic

## Module Entry

- `server-config-core/`
  - `src/`
    - `lib.rs`
    - `startup.rs`
    - `cpu_policy.rs`
    - `properties.rs`
    - `types.rs`
  - `Cargo.toml`

## Stable Boundaries

- Keep this crate mostly pure and reusable.
- Prefer passing explicit input structs over reading global state.
- Host crates may adapt their own models into this crate, but rule changes should happen here first.
- If a change affects startup precedence, JVM arg ordering, or CPU-set parsing, update tests in this crate before touching host-side adapters.

## Key Files

[`src/lib.rs`](src/lib.rs): Public re-export surface for shared startup and CPU-policy helpers.
- `resolve_effective_startup_config_from_document`: Main precedence resolver for startup config.
- `build_managed_jvm_args_from_input`: Main JVM-arg synthesis entry for managed local startup.
- `resolve_cpu_policy` / `resolve_local_cpu_policy`: Shared CPU policy resolvers.

[`src/startup.rs`](src/startup.rs): Startup config IO, precedence resolution, JVM arg assembly, and `server.properties` helpers.
- `resolve_effective_startup_config_from_document` -> `EffectiveStartupConfig`: Merges instance config, runtime values, and defaults.
- `build_managed_jvm_args_from_input` -> `Vec<String>`: Builds final managed JVM args, including memory args, encoding args, presets, defaults, and user args.
- `read_server_startup_config_document` -> `ServerStartupConfigDocument`: Reads `SeaLantern/config.toml`, falls back to legacy `SL.json`, and preserves field presence.
- `write_server_startup_config` -> `Result<(), String>`: Writes canonical startup config.

[`src/cpu_policy.rs`](src/cpu_policy.rs): Pure CPU policy parsing and effective-count logic.
- `parse_cpu_set` -> `Vec<usize>`: Parses explicit CPU sets like `0-3,6`.
- `format_cpu_range` -> `String`: Compresses CPU indices into a display string.
- `resolve_active_processor_count` -> `Option<u16>`: Computes `-XX:ActiveProcessorCount` value for local bounded hosts.
- `resolve_unbounded_active_processor_count` -> `Option<u16>`: Computes effective processor count without host CPU-limit validation.

[`src/properties.rs`](src/properties.rs): `server.properties` parsing and rendering helpers shared by backend flows.

[`src/types.rs`](src/types.rs): Shared config types used by both this crate and its host adapters.

## Change Guidance For Agents

- Do not move host-specific runtime policy into this crate unless it is genuinely reusable.
- Do not add process or filesystem side effects beyond config/property IO already owned here.
- Preserve these precedence rules unless the user explicitly asks to change behavior:
  - instance startup config wins when the field is present
  - runtime config is the fallback when the instance field is absent
  - app defaults are only used for default memory fallback cases
- Preserve JVM arg order unless the user explicitly asks otherwise.
  - memory and encoding args first
  - injected `ActiveProcessorCount` before default JVM args
  - preset args, default args, then user args

## Validation Checklist

- Run this crate's tests if startup precedence, JVM arg synthesis, CPU policy parsing, or config IO changed.
- Re-check any host crate tests that depend on startup preview or launch assembly if public behavior changed.
