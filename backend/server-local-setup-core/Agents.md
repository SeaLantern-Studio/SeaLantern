---
module-name: sea-lantern-server-local-setup-core
update-time: 2026-07-05
description: Shared local-server setup, path resolution, startup-mode inference, Java-env, and launch-preview helpers for SeaLantern.
tag: ["local-setup", "startup-path", "launch-preview", "java-env", "fallback"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`sea-lantern-server-local-setup-core` owns shared local-server setup helpers.

This crate should own:

- local folder inspection for attach and create flows
- startup-mode inference from file paths and folder contents
- preferred local JAR resolution for direct launch
- Java path splitting and launch `PATH` / `JAVA_HOME` helpers
- local launch-target resolution and command preview helpers
- stable fallback-message formatting for direct-jar fallback flows

This crate should not own:

- actual process spawning
- Tauri command handling

## Module Entry

- `server-local-setup-core/`
  - `src/`
    - `lib.rs`
  - `Cargo.toml`

## Key Files

[`src/lib.rs`](src/lib.rs): Shared local setup and local launch helper surface.
- `inspect_local_folder_checked` -> `fn(&Path) -> Result<LocalFolderInspection, String>`: Inspects a local server folder for attachability and inferred metadata.
- `resolve_local_preferred_jar_path_checked` -> `fn(...) -> Result<Option<String>, String>`: Resolves the preferred direct-launch JAR.
- `resolve_direct_jar_launch_target` -> `fn(&str, &str) -> String`: Resolves the launch target path used for direct JAR launch.
- `resolve_local_launch_target` -> `fn(...) -> String`: Computes the human-facing launch target for previews.
- `resolve_java_paths` -> `fn(&str) -> Result<(String, String), String>`: Splits a Java executable path into bin and home paths.
- `preview_command` -> `fn(&Command) -> String`: Renders a command preview string.
- `build_primary_jar_fallback_info` -> `fn(...) -> LocalLaunchFallbackInfo`: Builds stable direct-launch fallback metadata.
- `ensure_supported_script_java_major_version` / `detect_java_major_version`: Shared Java-version guardrails for script startup flows.

## Stable Boundaries

- Keep this crate host-agnostic and reusable.
- Put shared path, launch-target, Java-env, and fallback-text logic here instead of duplicating it in host crates.
- Preserve the current distinction between best-effort wrappers and checked variants.

## Change Guidance For Agents

- Do not add actual runtime spawning here.
- If startup-mode inference changes, verify attach, local-create, and preview callers together.
- Keep filesystem/path validation centralized here when it affects local server setup behavior.

## Validation Checklist

- Run this crate's tests if startup-mode inference, path resolution, Java-env helpers, or fallback formatting changed.
- Re-check `tauri-host` startup preview and local launch flows if public helper behavior changed.
