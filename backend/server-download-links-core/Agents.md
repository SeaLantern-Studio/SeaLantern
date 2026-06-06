---
module-name: sea-lantern-server-download-links-core
update-time: 2026-06-06
description: Shared parser for SeaLantern server-download link payloads.
tag: ["download-links", "parser", "server-links"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Server Download Links Core

`sea-lantern-server-download-links-core` parses structured download-link payloads for Minecraft server distributions.

This crate should own:

- JSON parsing for the server-download payload shape
- case-insensitive type lookup behavior
- version-to-link selection rules
- installer-jar preference rules when multiple files exist for one version

This crate should not own network fetching, UI, or host-specific persistence.

## Module Entry

- `server-download-links-core/`
  - `src/`
    - `lib.rs`
  - `Cargo.toml`

## Module Info

[`backend/server-download-links-core/src/lib.rs`](../server-download-links-core/src/lib.rs): Shared payload parser and typed result model. | Main crate surface.
- `parse_base_download_links` -> `fn(&str) -> Result<BaseDownloadLinks, String>`: Parses raw JSON into typed link groups.
- `BaseDownloadLinks` -> `struct`: Top-level parsed payload.
- `TypeDownloadLinks` -> `struct`: Per-server-type version and link group.
- `DownloadLink` -> `struct`: One downloadable file for one version.

## Change Guidance For Agents

- Keep parsing strict and error messages actionable.
- Preserve case-insensitive type matching.
- Preserve file-priority behavior unless the user explicitly wants a different selection rule.
