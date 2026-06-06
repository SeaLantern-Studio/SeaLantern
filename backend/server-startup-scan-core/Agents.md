---
module-name: sea-lantern-server-startup-scan-core
update-time: 2026-06-06
description: Shared startup-candidate scanning logic for folders and archives in SeaLantern.
tag: ["startup-scan", "archive", "folder-scan", "candidate-detection"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Server Startup Scan Core

`sea-lantern-server-startup-scan-core` scans local folders or archives and produces startup candidates.

This crate should own:

- startup-candidate discovery for folders and archives
- candidate ranking and recommendation order
- starter-vs-jar distinction based on parsed main class
- MC-version option carry-through for startup scan results

This crate should not own UI state or archive persistence.

## Module Entry

- `server-startup-scan-core/`
  - `src/`
    - `lib.rs`
  - `Cargo.toml`

## Module Info

[`backend/server-startup-scan-core/src/lib.rs`](../server-startup-scan-core/src/lib.rs): Shared startup-candidate scanning surface. | Main crate API.
- `scan_startup_candidates` -> `fn(&str, &str, &[&str]) -> Result<StartupScanResult, String>`: Scans a folder or archive and returns ranked startup candidates.
- `StartupScanResult` -> `struct`: Typed result model for startup-scan consumers.
- `StartupCandidateItem` -> `struct`: One startup candidate with mode, label, detail, path, and recommendation score.

## Change Guidance For Agents

- Keep candidate ranking deterministic.
- If startup-mode recommendation behavior changes, update tests that assert candidate order.
- Prefer surfacing strict scan failures instead of silently downgrading broken archive or JAR input.
