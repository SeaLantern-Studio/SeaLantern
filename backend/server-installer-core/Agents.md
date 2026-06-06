---
module-name: sea-lantern-server-installer-core
update-time: 2026-06-06
description: Shared server archive extraction, core-type detection, and MC-version inference logic for SeaLantern.
tag: ["server-installer", "core-detection", "archive", "mc-version"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Server Installer Core

`sea-lantern-server-installer-core` owns shared server-package inspection logic.

This crate should own:

- archive extraction for modpacks and server bundles
- finding the server JAR in extracted content
- core-type detection from filenames and JAR main classes
- MC-version inference from mods content

This crate should not own Tauri commands or application persistence.

## Module Entry

- `server-installer-core/`
  - `src/`
    - `lib.rs`
    - `archive.rs`
    - `core_type.rs`
    - `mc_version.rs`
    - `parser.rs`
  - `Cargo.toml`

## Module Info

[`backend/server-installer-core/src/lib.rs`](../server-installer-core/src/lib.rs): Public detection and extraction entry surface. | Main consumer API.
- `detect_core_type` -> `fn(&str) -> String`: Best-effort core-type detection.
- `detect_core_type_checked` -> `fn(&str) -> Result<String, String>`: Strict core-type detection.
- `detect_mc_version_from_mods_checked` -> `fn(...) -> Result<(Option<String>, bool), String>`: Strict MC-version inference.

[`backend/server-installer-core/src/archive.rs`](../server-installer-core/src/archive.rs): Archive extraction and server-JAR discovery helpers. | Shared archive behavior.
- `extract_modpack_archive` -> `fn(...) -> Result<(), String>`: Extracts a modpack archive.
- `find_server_jar_checked` -> `fn(...) -> Result<String, String>`: Finds a launchable server JAR.

[`backend/server-installer-core/src/parser.rs`](../server-installer-core/src/parser.rs): Parsed JAR/core metadata model and parsing entry. | Main class and core parsing.
- `parse_server_core_type` -> `fn(&str) -> Result<ParsedServerCoreInfo, String>`: Parses JAR or folder into typed core info.

[`backend/server-installer-core/src/core_type.rs`](../server-installer-core/src/core_type.rs): Canonical server-core taxonomy. | Shared type normalization.

## Change Guidance For Agents

- Keep detection logic deterministic and reusable.
- Prefer checked variants for new host integrations when failure should be surfaced.
- Preserve the distinction between best-effort helpers and strict checked helpers.
