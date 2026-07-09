---
module-name: sea-lantern-server-download-links-core
update-time: 2026-07-05
description: Shared parser for SeaLantern server-download link payloads.
tag: ["download-links", "parser", "server-links"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`sea-lantern-server-download-links-core` parses structured download-link payloads for Minecraft server distributions.

This crate should own:

- JSON parsing for the server-download payload shape
- case-insensitive type lookup behavior
- version-to-link selection rules
- installer-jar preference rules when multiple files exist for one version

This crate should not own:

- network fetching
- UI logic
- host-specific persistence

## Module Entry

- `server-download-links-core/`
  - `src/`
    - `lib.rs`
  - `Cargo.toml`

## Key Files

[`src/lib.rs`](src/lib.rs): Shared payload parser and typed result model.
- `parse_base_download_links` -> `fn(&str) -> Result<BaseDownloadLinks, String>`: Parses raw JSON into typed link groups.
- `BaseDownloadLinks` -> `struct`: Top-level parsed payload.
- `TypeDownloadLinks` -> `struct`: Per-server-type version and link group.
- `DownloadLink` -> `struct`: One downloadable file for one version.

## Stable Boundaries

- Keep parsing strict and deterministic.
- Preserve case-insensitive type matching and current installer-file priority behavior unless the user explicitly asks for a new selection rule.
- Keep this crate pure; fetching and persistence belong elsewhere.

## Change Guidance For Agents

- Prefer changing typed parsing rules here instead of post-processing raw JSON in host crates.
- Keep error messages actionable because callers surface them directly.
- Treat payload shape changes as shared contract changes.

## Validation Checklist

- Run this crate's tests if payload parsing, version matching, or installer-file selection changed.
- Re-check any host flow that imports server-download metadata if result ordering or errors changed.
