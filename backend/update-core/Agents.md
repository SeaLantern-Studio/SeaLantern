---
module-name: sea-lantern-update-core
update-time: 2026-06-06
description: Shared update-check, download, pending-install, and install-launch planning logic for SeaLantern.
tag: ["update", "release-check", "download", "pending-install", "windows-install"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Update Core

`sea-lantern-update-core` owns shared application-update backend logic.

This crate should own:

- release-check logic
- platform-aware source selection for update metadata
- update file downloading
- pending-update persistence and cleanup
- install-launch planning and Windows elevated install helpers

This crate should not own frontend update UI.

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
  - `Cargo.toml`

## Module Info

[`backend/update-core/src/lib.rs`](../update-core/src/lib.rs): Shared update-check entry surface. | Main crate API.
- `check_update` -> `async fn(&str) -> Result<UpdateInfo, String>`: Checks for updates using platform-aware source logic.

[`backend/update-core/src/types.rs`](../update-core/src/types.rs): Shared update data models. | Typed update payloads.
- `UpdateInfo` -> `struct`: Main update-check result model.
- `DownloadProgress` -> `struct`: Download progress payload.
- `PendingUpdate` -> `struct`: Pending-install metadata.

[`backend/update-core/src/download.rs`](../update-core/src/download.rs): Shared update file download flow. | Download helper.

[`backend/update-core/src/pending.rs`](../update-core/src/pending.rs): Pending-update persistence helpers. | Pending install state.

[`backend/update-core/src/install_support.rs`](../update-core/src/install_support.rs): Install-launch planning and cache-path helpers. | Install execution planning.

[`backend/update-core/src/version.rs`](../update-core/src/version.rs): Version comparison and release-tag normalization. | Shared version logic.

[`backend/update-core/src/windows_install.rs`](../update-core/src/windows_install.rs): Windows elevated install support. | Platform-specific install helper.

## Change Guidance For Agents

- Keep update source selection explicit and testable.
- Preserve the distinction between checking, downloading, pending state, and install launching.
- Avoid pushing UI-specific policy into this crate.
