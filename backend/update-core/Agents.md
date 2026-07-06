---
module-name: sea-lantern-update-core
update-time: 2026-07-05
description: Shared update-check, download, pending-install, and install-launch planning logic for SeaLantern.
tag: ["update", "release-check", "download", "pending-install", "windows-install"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`sea-lantern-update-core` owns shared application-update backend logic.

This crate should own:

- release-check logic
- platform-aware source selection for update metadata
- update file downloading
- pending-update persistence and cleanup
- install-launch planning and Windows elevated install helpers

This crate should not own:

- frontend update UI

## Module Entry

- `update-core/`
  - `src/`
    - `lib.rs`
    - `types.rs`
    - `download.rs`
    - `pending.rs`
    - `install_support.rs`
    - `version.rs`
    - `windows_install.rs`
    - `arch.rs`
    - `cnb.rs`
    - `constants.rs`
  - `Cargo.toml`

## Key Files

[`src/lib.rs`](src/lib.rs): Shared update-check entry surface.
- `check_update` -> `async fn(&str) -> Result<UpdateInfo, String>`: Checks for updates using platform-aware source logic.

[`src/types.rs`](src/types.rs): Shared update data models.
- `UpdateInfo` -> `struct`: Main update-check result model.
- `DownloadProgress` -> `struct`: Download progress payload.
- `PendingUpdate` -> `struct`: Pending-install metadata.

[`src/download.rs`](src/download.rs): Shared update file download flow.
- Owns download and checksum-related transfer behavior.

[`src/pending.rs`](src/pending.rs): Pending-update persistence helpers.
- Owns pending install state and cleanup behavior.

[`src/install_support.rs`](src/install_support.rs): Install-launch planning and cache-path helpers.
- Owns install execution planning.

[`src/version.rs`](src/version.rs): Version comparison and release-tag normalization.
- Shared version logic.

[`src/windows_install.rs`](src/windows_install.rs): Windows elevated install support.
- Platform-specific install helper.

[`src/arch.rs`](src/arch.rs), [`src/cnb.rs`](src/cnb.rs), [`src/constants.rs`](src/constants.rs): Source-specific update helpers and constants.
- Keep platform/source-specific update policy in the core crate, not in UI code.

## Stable Boundaries

- Keep update source selection explicit and testable.
- Preserve the distinction between checking, downloading, pending state, and install launching.
- Debug/dev-mode behavior and production update-source behavior should stay clearly separated.

## Change Guidance For Agents

- Avoid pushing UI-specific policy into this crate.
- If source selection changes, verify Linux vs non-Linux behavior and debug vs release behavior separately.
- Treat update result shapes and pending-install metadata as shared contracts.

## Validation Checklist

- Run this crate's tests if release checking, download flow, pending state, version normalization, or Windows install behavior changed.
- Re-check host-side update flows if public payload fields or source-selection behavior changed.
