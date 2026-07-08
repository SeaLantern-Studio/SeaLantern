# Backend Maintainer Notes

The backend is a Rust workspace rooted at the repository `Cargo.toml`.

## Ownership

- `tauri-host`: desktop/headless host, Tauri commands, services, persistence, plugin runtime,
  command catalog, and OS integration.
- `docker-entry`: thin Docker/headless executable entrypoint.
- `runtime-core`: logging, headless HTTP helpers, path utilities, status parsing, panic reporting,
  Tokio/runtime setup.
- `event-core`: app/server event envelopes, subscriptions, registry DTOs, and in-process event
  manager.
- `i18n-core`: backend locale service, embedded locale assets, plugin translation registration.
- `server-config-core`: startup config, JVM args, CPU policy, and `server.properties` rules.
- `docker-core`: Docker launch shape, run args, preview detail, JVM env synthesis, and Docker error
  classification.
- `server-local-setup-core`: local setup, startup-mode inference, Java env/path handling, and launch
  preview helpers.
- `server-installer-core`: archive extraction, server core detection, taxonomy mapping, and Minecraft
  version inference.
- `server-startup-scan-core`: startup candidates from folders and archives.
- `java-installer-core`: Java runtime download, extraction, and install finalization.
- `server-log-core`: SQLite log persistence, history reads, output reader hooks, and structured log
  mapping.
- `server-plugin-core`: Minecraft server plugin jar/config scanning and extension target resolution.
- `plugin-trust-core`: plugin permission metadata, trust assessment, trusted catalog parsing, and
  enable policy.
- `lua-runtime-core`: shared trait boundary for Lua runtime integration.
- `server-download-links-core`: server download link payload parsing.
- `starter-links-core`: cached starter installer link loading and URL resolution.
- `update-core`: update checking, download, pending install, and install launch planning.

## Boundaries

- Put pure reusable rules in a `*-core` crate.
- Put command exposure, manager orchestration, plugin runtime execution, persistence, and host state
  in `tauri-host`.
- Keep `docker-entry` and binary `main.rs` files thin.
- If preview and real execution must match, make both consume the same normalized core shape.
- Do not copy Docker, startup, Java, log, event, or plugin trust rules into `tauri-host` when a core
  crate already owns them.

## Common Contract Paths

- Frontend caller: `frontend/src/api/*.ts`.
- Command catalog: `backend/tauri-host/src/runtime/command_catalog.rs`.
- Tauri commands: `backend/tauri-host/src/commands/`.
- Host services: `backend/tauri-host/src/services/`.
- Plugin runtime: `backend/tauri-host/src/plugins/`.

Check caller, registration, and implementation together for frontend-backend contract changes.

## Validation

- Workspace baseline:
  - `cargo fmt --all -- --check`
  - `cargo check --workspace`
  - `cargo clippy --workspace -- -D warnings`
- For scoped crate work, a targeted command such as
  `cargo check --manifest-path backend/<crate>/Cargo.toml` is acceptable when it proves the change.
- Re-check `tauri-host` when a core crate change affects command payloads, runtime behavior, or
  plugin-visible behavior.

## Core Invariants

- shared core over host duplication
- preview and runtime parity
- command payload stability
- plugin permission and trust boundaries
- log ordering and output-reader lifecycle
- Docker env synthesis and sensitive preview redaction
- startup config precedence and CPU policy consistency

- Docker preview must describe the same `docker run` shape that execution uses.
- Docker JVM env synthesis must not overwrite runtime-provided `JVM_OPTS` or `JVM_XX_OPTS`.
- Docker CPU policy should stay aligned with `server-config-core` unless behavior is explicitly split.
- Startup config precedence is instance config, then runtime config, then app defaults.
- Managed JVM arg ordering matters: memory and encoding args, preset/user args, and injected
  `ActiveProcessorCount` should stay intentional and tested when changed.
- Event DTOs and serialized event fields are frontend-facing contracts.
- Backend locale IDs and locale callback behavior are shared runtime contracts.
- Java runtime install flow should preserve cancellation behavior and temporary directory cleanup.
- Log persistence should preserve append ordering, history ordering, flush lifecycle, and ready-event
  detection semantics.
- Plugin trust and permission normalization should stay centralized in `plugin-trust-core`.
- Lua runtime trait signatures are plugin-runtime contracts and require coordinated host/runtime
  updates when changed.
- Parsers for server download links, starter links, startup candidates, and server-core detection
  should remain deterministic; ordering changes are public behavior for callers.
- Server installer compatibility wrappers should remain compatibility wrappers unless callers are
  moved to strict checked variants.
- Server plugin scanning should not mix SeaLantern Lua plugin logic with Minecraft server plugin
  file logic.
- `tauri-host` launch behavior has separate paths for jar/starter, bat, sh, ps1, and custom starts;
  do not collapse them without an explicit behavior change.
- Update logic should keep checking, downloading, pending state, and install launching separated.
