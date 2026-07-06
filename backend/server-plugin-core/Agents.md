---
module-name: sea-lantern-server-plugin-core
update-time: 2026-07-05
description: Shared Minecraft server-plugin JAR parsing, extension target resolution, and plugin-config listing logic for SeaLantern.
tag: ["plugin-jar", "plugin-list", "plugin-config"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`sea-lantern-server-plugin-core` handles Minecraft server-plugin and extension file inspection.

This crate should own:

- scanning server extension directories such as `plugins/` and other supported extension targets
- parsing `plugin.yml` and `bungee.yml` from plugin JARs
- fallback metadata for invalid or descriptor-less plugin JARs
- recursive scanning of plugin config files with supported extensions
- extension target-directory resolution based on core/runtime/startup context

This crate should not own:

- SeaLantern's own Lua plugin system
- Tauri command handlers
- frontend plugin views

## Module Entry

- `server-plugin-core/`
  - `src/`
    - `lib.rs`
  - `Cargo.toml`

## Key Files

[`src/lib.rs`](src/lib.rs): Shared plugin JAR and config-folder inspection logic.
- `get_plugins` -> `fn(&str) -> Result<Vec<m_PluginInfo>, String>`: Best-effort plugin scan that keeps broken plugin JARs visible with fallback metadata.
- `get_plugins_checked` -> `fn(&str) -> Result<Vec<m_PluginInfo>, String>`: Strict plugin scan that surfaces invalid JAR parsing failures.
- `get_plugin_config_files` -> `fn(&str, &str) -> Result<Vec<m_PluginConfigFile>, String>`: Recursively scans supported plugin config files.
- `resolve_extension_relative_dir` / `resolve_extension_target_dir`: Resolve extension directories from core/runtime/startup context.
- `ensure_extension_target_dir` -> `fn(...) -> Result<PathBuf, String>`: Ensures the resolved extension directory exists.
- `m_PluginInfo` -> `struct`: Output model for one detected server plugin.
- `m_PluginConfigFile` -> `struct`: Output model for one scanned plugin config file.

## Stable Boundaries

- Keep plugin JAR parsing, extension-target resolution, and config-file scanning centralized here.
- Preserve fallback visibility for malformed plugin JARs in non-strict mode.
- Keep supported config-file extensions and extension-target mapping explicit and easy to audit.

## Change Guidance For Agents

- Do not mix SeaLantern host-plugin logic into this crate.
- If runtime/core-specific extension placement changes, verify the taxonomy-driven resolution path here instead of hardcoding new folders in host code.
- Treat plugin parsing output as shared data consumed by multiple host flows.

## Validation Checklist

- Run this crate's tests if plugin JAR parsing, extension-target resolution, or config-file discovery changed.
- Re-check host-side extension install/listing flows if returned paths, metadata, or fallback behavior changed.
