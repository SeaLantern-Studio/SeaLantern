---
module-name: sea-lantern-server-installer-core
update-time: 2026-07-05
description: Shared server archive extraction, core-type detection, and MC-version inference logic for SeaLantern.
tag: ["server-installer", "core-detection", "archive", "mc-version"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`sea-lantern-server-installer-core` owns shared server-package inspection logic.

This crate should own:

- archive extraction for modpacks and server bundles
- finding the server JAR in extracted content
- core-type detection from filenames and JAR main classes
- MC-version inference from mods content

New integrations should treat canonical server core keys as the default contract.

This crate should not own:

- Tauri commands
- application persistence

## Module Entry

- `server-installer-core/`
  - `src/`
    - `lib.rs`
    - `archive.rs`
    - `core_type.rs`
    - `mc_version.rs`
    - `parser.rs`
  - `Cargo.toml`

## Key Files

[`src/lib.rs`](src/lib.rs): Public detection and extraction entry surface.
- `detect_core_key` -> `fn(&str) -> String`: Best-effort canonical core-key detection.
- `detect_core_key_checked` -> `fn(&str) -> Result<String, String>`: Strict canonical core-key detection.
- `resolve_imported_server_core_key` -> `fn(&str, &str) -> String`: Resolves the persisted imported-server core key from startup mode and resolved startup target path.
- `should_copy_modpack_source_as_native_server_binary` -> `fn(&Path) -> bool`: Detects modpack inputs that should be copied directly as native server binaries instead of extracted as archives.
- `resolve_starter_core_key` -> `fn(Option<&str>, Option<&str>) -> Option<String>`: Resolves the canonical starter-install core key from explicit server metadata or a launch target path.
- `detect_core_type` -> `fn(&str) -> String`: Legacy display-style core detection kept for compatibility.
- `detect_core_type_checked` -> `fn(&str) -> Result<String, String>`: Strict legacy display-style core detection.
- `detect_mc_version_from_mods_checked` -> `fn(...) -> Result<(Option<String>, bool), String>`: Strict MC-version inference.

[`src/archive.rs`](src/archive.rs): Archive extraction and server-JAR discovery helpers.
- `extract_modpack_archive` -> `fn(...) -> Result<(), String>`: Extracts a modpack archive.
- `find_server_jar_checked` -> `fn(...) -> Result<String, String>`: Finds a launchable server JAR.

[`src/parser.rs`](src/parser.rs): Parsed JAR/core metadata model and parsing entry.
- `parse_server_core_key` -> `fn(&str) -> Result<ParsedServerCoreInfo, String>`: Parses JAR or folder into typed core info using canonical core keys.
- `parse_server_core_type` -> `fn(&str) -> Result<ParsedServerCoreInfo, String>`: Legacy display-style parser kept for compatibility.

[`src/core_type.rs`](src/core_type.rs): Canonical server-core taxonomy.
- `CoreType::docker_type_resolution` -> `fn(&str) -> Option<DockerTypeResolution>`: Resolves canonical API core key plus Docker `TYPE` env value from any supported alias input.
- `CoreType::starter_install_args` -> `fn(&str) -> Option<StarterInstallArgs>`: Resolves the starter installer CLI argument contract for the normalized core family.

## Stable Boundaries

- Keep detection logic deterministic and reusable across import, scan, and startup flows.
- Prefer canonical `*_core_key` helpers for persistence and cross-crate integration boundaries.
- Treat `*_core_type` helpers as compatibility wrappers unless a caller explicitly needs display-style output.

## Change Guidance For Agents

- Prefer checked variants for new host integrations when failure should be surfaced.
- Preserve the distinction between best-effort helpers and strict checked helpers.
- If archive or parser behavior changes, verify downstream startup-scan and local-setup consumers too.

## Validation Checklist

- Run this crate's tests if archive extraction, core detection, taxonomy mapping, or MC-version inference changed.
- Re-check `server-startup-scan-core`, `server-local-setup-core`, and `tauri-host` flows if public detection behavior changed.
