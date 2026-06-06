---
module-name: sea-lantern-server-local-setup-core
update-time: 2026-06-06
description: Shared local-server setup, path resolution, startup-mode inference, Java-env, and launch-preview helpers for SeaLantern.
tag: ["local-setup", "startup-path", "launch-preview", "java-env", "fallback"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Server Local Setup Core

`sea-lantern-server-local-setup-core` owns shared local-server setup helpers.

This crate should own:

- local folder inspection for attach and create flows
- startup-mode inference from file paths and folder contents
- preferred local JAR resolution for direct launch
- Java path splitting and launch PATH/JAVA_HOME helpers
- local launch-target resolution and command preview helpers
- stable fallback-message formatting for direct-jar fallback flows

This crate should not own actual process spawning or Tauri command handling.

## Module Entry

- `server-local-setup-core/`
  - `src/`
    - `lib.rs`
  - `Cargo.toml`

## Module Info

[`backend/server-local-setup-core/src/lib.rs`](../server-local-setup-core/src/lib.rs): Shared local setup and local launch helper surface. | Main crate API.
- `inspect_local_folder_checked` -> `fn(&Path) -> Result<LocalFolderInspection, String>`: Inspects a local server folder for attachability and inferred metadata.
- `resolve_local_preferred_jar_path_checked` -> `fn(...) -> Result<Option<String>, String>`: Resolves the preferred direct-launch JAR.
- `resolve_direct_jar_launch_target` -> `fn(&str, &str) -> String`: Resolves the launch target path used for direct JAR launch.
- `resolve_local_launch_target` -> `fn(...) -> String`: Computes the human-facing launch target for previews.
- `resolve_java_paths` -> `fn(&str) -> Result<(String, String), String>`: Splits a Java executable path into bin and home paths.
- `preview_command` -> `fn(&Command) -> String`: Renders a command preview string.
- `build_primary_jar_fallback_info` -> `fn(...) -> LocalLaunchFallbackInfo`: Builds stable direct-launch fallback metadata.

## Change Guidance For Agents

- Keep this crate host-agnostic and reusable.
- Put shared path, launch-target, and fallback-text logic here instead of duplicating it in host crates.
- Do not add actual runtime spawning here.
- Preserve the current distinction between best-effort wrappers and checked variants.
