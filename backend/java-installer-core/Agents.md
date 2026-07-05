---
module-name: sea-lantern-java-installer-core
update-time: 2026-07-05
description: Shared Java runtime download, archive extraction, and installation logic for SeaLantern.
tag: ["java-runtime", "installer", "download", "archive"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`sea-lantern-java-installer-core` owns shared Java runtime installation logic.

This crate should own:

- downloading Java runtime archives
- progress reporting for download and extraction
- temporary install directory management
- archive extraction and install-source resolution
- locating the final Java executable inside the installed runtime

This crate should not own:

- Tauri commands
- UI state
- server-model persistence
- machine-wide Java detection

## Module Entry

- `java-installer-core/`
  - `src/`
    - `lib.rs`
    - `archive.rs`
    - `shared.rs`
  - `Cargo.toml`

## Key Files

[`src/lib.rs`](src/lib.rs): Main shared Java installation flow.
- `download_and_install_java` -> `async fn(...)-> Result<PathBuf, String>`: Downloads a runtime archive, extracts it, and returns the installed Java binary path.
- `JavaInstallProgress` -> `struct`: Progress payload for downloading and extracting states.

[`src/archive.rs`](src/archive.rs): Platform-aware archive extraction helpers.
- Owns zip/tar extraction behavior and archive-specific install handling.

[`src/shared.rs`](src/shared.rs): Shared constants and install-path helpers.
- Owns install-source resolution, Java binary path helpers, and retry-related constants.

## Stable Boundaries

- Keep this crate reusable and host-agnostic.
- Prefer exposing progress-state changes instead of UI-specific messages or flows.
- Archive download, extraction, and final binary resolution should stay coordinated here rather than being split across host crates.

## Change Guidance For Agents

- Preserve cancellation behavior and temp-dir cleanup.
- Keep platform-specific archive handling behind the shared extraction helpers.
- If install behavior changes, update both happy-path and failure-path tests.

## Validation Checklist

- Run this crate's tests if download flow, extraction behavior, install-source resolution, or progress reporting changed.
- Re-check host-side Java install flows if the returned Java binary path or cancellation semantics changed.
