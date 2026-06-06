---
module-name: sea-lantern-starter-links-core
update-time: 2026-06-06
description: Shared cached starter-installer link loading, validation, and URL resolution logic for SeaLantern.
tag: ["starter-links", "cache", "installer-url", "json-validate"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Starter Links Core

`sea-lantern-starter-links-core` owns cached starter-installer link handling.

This crate should own:

- loading or refreshing cached starter-link JSON
- validating the starter-link JSON payload shape
- resolving the installer URL for a given core type and MC version
- version-aware matching and installer-file preference rules

This crate should not own UI logic or server-model persistence.

## Module Entry

- `starter-links-core/`
  - `src/`
    - `lib.rs`
    - `cache.rs`
    - `parser.rs`
    - `version.rs`
  - `Cargo.toml`

## Module Info

[`backend/starter-links-core/src/lib.rs`](../starter-links-core/src/lib.rs): Shared cached starter-link resolution surface. | Main crate API.
- `load_or_refresh_starter_links_json` -> `fn(...) -> Result<..., String>`: Loads or refreshes cached starter-link JSON.
- `resolve_installer_url_from_cache_file` -> `fn(...) -> Result<(String, Option<String>), String>`: Resolves the matching installer URL from a cache file.

[`backend/starter-links-core/src/cache.rs`](../starter-links-core/src/cache.rs): Cache-file refresh and reuse logic. | Shared cache behavior.

[`backend/starter-links-core/src/parser.rs`](../starter-links-core/src/parser.rs): Payload validation and nested installer-URL resolution. | Shared payload rules.

## Change Guidance For Agents

- Keep payload validation strict.
- Preserve core-type and version normalization behavior.
- Preserve preference for the most specific matching installer JAR unless the user asks otherwise.
