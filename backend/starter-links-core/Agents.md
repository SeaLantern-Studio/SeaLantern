---
module-name: sea-lantern-starter-links-core
update-time: 2026-07-05
description: Shared cached starter-installer link loading, validation, and URL resolution logic for SeaLantern.
tag: ["starter-links", "cache", "installer-url", "json-validate"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`sea-lantern-starter-links-core` owns cached starter-installer link handling.

This crate should own:

- loading or refreshing cached starter-link JSON
- validating the starter-link JSON payload shape
- resolving the installer URL for a given core type and MC version
- version-aware matching and installer-file preference rules

This crate should not own:

- UI logic
- server-model persistence

## Module Entry

- `starter-links-core/`
  - `src/`
    - `lib.rs`
    - `cache.rs`
    - `parser.rs`
    - `version.rs`
  - `Cargo.toml`

## Key Files

[`src/lib.rs`](src/lib.rs): Shared cached starter-link resolution surface.
- `load_or_refresh_starter_links_json` -> `fn(...) -> Result<..., String>`: Loads or refreshes cached starter-link JSON.
- `resolve_installer_url_from_cache_file` -> `fn(...) -> Result<(String, Option<String>), String>`: Resolves the matching installer URL from a cache file.

[`src/cache.rs`](src/cache.rs): Cache-file refresh and reuse logic.
- Owns cache freshness, refresh, and reuse behavior.

[`src/parser.rs`](src/parser.rs): Payload validation and nested installer-URL resolution.
- Owns payload validation and nested matching rules.

[`src/version.rs`](src/version.rs): Version matching helpers.
- Keeps version normalization and matching behavior consistent.

## Stable Boundaries

- Keep payload validation strict.
- Preserve core-type and version normalization behavior.
- Preserve preference for the most specific matching installer JAR unless the user asks otherwise.

## Change Guidance For Agents

- Prefer changing version resolution or file preference rules here instead of layering ad hoc host-side fallback logic.
- Treat cache-file format expectations as shared contracts.
- If refresh behavior changes, verify both cold-cache and warm-cache paths.

## Validation Checklist

- Run this crate's tests if cache refresh, payload validation, version matching, or installer selection changed.
- Re-check starter install flows in `tauri-host` if resolved installer URLs or error semantics changed.
