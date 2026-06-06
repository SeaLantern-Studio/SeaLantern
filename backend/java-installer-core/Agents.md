---
module-name: sea-lantern-java-installer-core
update-time: 2026-06-06
description: Shared Java runtime download, archive extraction, and installation logic for SeaLantern.
tag: ["java-runtime", "installer", "download", "archive"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Java Installer Core

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

## Module Info

[`backend/java-installer-core/src/lib.rs`](../java-installer-core/src/lib.rs): Main shared Java installation flow. | Download, extract, and finalize.
- `download_and_install_java` -> `async fn(...)-> Result<PathBuf, String>`: Downloads a runtime archive, extracts it, and returns the installed Java binary path.
- `JavaInstallProgress` -> `struct`: Progress payload for downloading and extracting states.

[`backend/java-installer-core/src/archive.rs`](../java-installer-core/src/archive.rs): Platform-aware archive extraction helpers. | Internal archive handling.

[`backend/java-installer-core/src/shared.rs`](../java-installer-core/src/shared.rs): Shared constants and install-path helpers. | Internal path and retry policy support.

## Change Guidance For Agents

- Keep this crate reusable and host-agnostic.
- Prefer emitting explicit progress-state changes instead of UI-specific messages.
- Preserve cancellation behavior and temp-dir cleanup.
- If install behavior changes, update both happy-path and failure-path tests.
