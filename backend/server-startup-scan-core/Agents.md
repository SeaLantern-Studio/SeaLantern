---
module-name: sea-lantern-server-startup-scan-core
update-time: 2026-07-05
description: Shared startup-candidate scanning logic for folders and archives in SeaLantern.
tag: ["startup-scan", "archive", "folder-scan", "candidate-detection"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`sea-lantern-server-startup-scan-core` scans local folders or archives and produces startup candidates.

This crate should own:

- startup-candidate discovery for folders and archives
- candidate ranking and recommendation order
- starter-vs-jar distinction based on parsed main class
- MC-version option carry-through for startup scan results

This crate should not own:

- UI state
- archive persistence
- Tauri command handlers

## Module Entry

- `server-startup-scan-core/`
  - `src/`
    - `lib.rs`
  - `Cargo.toml`

## Key Files

[`src/lib.rs`](src/lib.rs): Shared startup-candidate scanning surface.
- `scan_startup_candidates` -> `fn(&str, &str, &[&str]) -> Result<StartupScanResult, String>`: Scans a folder or archive and returns ranked startup candidates.
- `StartupScanResult` -> `struct`: Typed result model for startup-scan consumers.
- `StartupCandidateItem` -> `struct`: One startup candidate with mode, label, detail, path, and recommendation score.

## Stable Boundaries

- Keep candidate ranking deterministic.
- Folder and archive scans should produce the same shared result shape.
- Core parsing and MC-version inference should stay delegated to the appropriate shared crates rather than being duplicated here.

## Change Guidance For Agents

- If startup-mode recommendation behavior changes, update tests that assert candidate order.
- Prefer surfacing strict scan failures instead of silently downgrading broken archive or JAR input.
- Treat candidate labels and recommendation ordering as shared behavior consumed by multiple host flows.

## Validation Checklist

- Run this crate's tests if startup candidate discovery, archive extraction paths, or recommendation ordering changed.
- Re-check local import/attach flows in `tauri-host` if candidate IDs, modes, or labels changed.
