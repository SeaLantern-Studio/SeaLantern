---
module-name: sea-lantern-server-plugin-core
update-time: 2026-06-06
description: Shared Minecraft server-plugin JAR parsing and plugin-config listing logic for SeaLantern.
tag: ["plugin-jar", "plugin-list", "plugin-config"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Server Plugin Core

`sea-lantern-server-plugin-core` handles Minecraft server-plugin file inspection.

This crate should own:

- scanning the `plugins/` folder
- parsing `plugin.yml` and `bungee.yml` from plugin JARs
- fallback metadata for invalid or descriptor-less plugin JARs
- recursive scanning of plugin config files with supported extensions

This crate should not own SeaLantern's own Lua plugin system.

## Module Entry

- `server-plugin-core/`
  - `src/`
    - `lib.rs`
  - `Cargo.toml`

## Module Info

[`backend/server-plugin-core/src/lib.rs`](../server-plugin-core/src/lib.rs): Shared plugin JAR and config-folder inspection logic. | Main crate API.
- `get_plugins` -> `fn(&str) -> Result<Vec<m_PluginInfo>, String>`: Best-effort plugin scan that keeps broken plugin JARs visible with fallback metadata.
- `get_plugins_checked` -> `fn(&str) -> Result<Vec<m_PluginInfo>, String>`: Strict plugin scan that surfaces invalid JAR parsing failures.
- `get_plugin_config_files` -> `fn(&str, &str) -> Result<Vec<m_PluginConfigFile>, String>`: Recursively scans supported plugin config files.
- `m_PluginInfo` -> `struct`: Output model for one detected server plugin.
- `m_PluginConfigFile` -> `struct`: Output model for one scanned plugin config file.

## Change Guidance For Agents

- Do not mix SeaLantern host-plugin logic into this crate.
- Preserve fallback visibility for malformed plugin JARs in non-strict mode.
- Keep supported config-file extensions explicit and easy to audit.
