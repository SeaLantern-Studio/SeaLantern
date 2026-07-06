---
module-name: sea-lantern-server-log-core
update-time: 2026-07-05
description: Shared server log persistence, line ingestion, history reading, structured event mapping, and output-reader hooks for SeaLantern.
tag: ["server-log-core", "logs", "sqlite", "output-reader", "structured-events"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`sea-lantern-server-log-core` is the shared server-log persistence crate.

This crate should own:

- SQLite-backed server log storage
- append and flush behavior for SeaLantern and server log lines
- history reads from the persisted log database
- stdout/stderr reader hooks that decode, parse, and persist live lines
- lightweight structured-event mapping derived from parsed server logs

This crate should not own:

- server process spawning
- event-bus publication policy outside of reader hooks
- Tauri command handlers
- frontend log rendering

## Module Entry

- `server-log-core/`
  - `src/`
    - `lib.rs`
    - `db.rs`
    - `output_reader.rs`
    - `reader.rs`
    - `state.rs`
    - `writer.rs`
  - `Cargo.toml`

## Key Files

[`src/lib.rs`](src/lib.rs): Public re-export surface and structured-event mapping helpers.
- `append_log` / `append_server_log` / `append_sealantern_log`: Shared append entrypoints.
- `read_logs`: Shared history-read entrypoint.
- `spawn_server_output_reader`: Shared live output ingestion entrypoint.
- `map_domain_event`: Maps parsed domain events into lightweight structured log fields.

[`src/output_reader.rs`](src/output_reader.rs): Live stdout/stderr ingestion path.
- `OutputReaderHooks`: Shared hook bundle for line emit, ready notification, and error reporting.
- `spawn_server_output_reader`: Reads console lines, decodes bytes, persists logs, parses structured events, and triggers hooks.

[`src/reader.rs`](src/reader.rs): History-read path over the log database.
- `read_logs`: Reads ordered log lines with optional recent-window limiting.

[`src/writer.rs`](src/writer.rs): Buffered log writer and flush lifecycle.
- `init_db`: Ensures the log database exists.
- `shutdown_writer`: Flushes and tears down a server-specific writer.
- `append_log`: Queues a log line and manages writer lifecycle.

[`src/state.rs`](src/state.rs): Shared log state, constants, and helper types.
- Owns writer registry state, file naming, batching constants, and byte-decoding helpers.

[`src/db.rs`](src/db.rs): SQLite schema and open-or-create helpers.
- Owns the storage bootstrap boundary.

## Stable Boundaries

- Keep log persistence and live output ingestion together in this crate.
- Preserve append ordering and history-read ordering semantics unless the caller contract is explicitly changed.
- Keep structured-event mapping lightweight and derived from parsed log input, not from host-specific assumptions.

## Change Guidance For Agents

- If live output parsing changes, verify both persisted history and hook emission behavior.
- Do not move SQLite writer lifecycle logic into `tauri-host`.
- Treat decoding and parsing behavior as shared runtime infrastructure used by multiple host flows.

## Validation Checklist

- Run this crate's tests if persistence, output reading, structured-event mapping, or read-window behavior changed.
- Re-check host-side log consumers if line ordering, ready-event detection, or structured fields changed.
