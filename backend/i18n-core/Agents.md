---
module-name: sea-lantern-i18n-core
update-time: 2026-07-05
description: Shared backend i18n service, embedded locale assets, locale callbacks, and plugin translation registration for SeaLantern.
tag: ["i18n-core", "locales", "translations", "plugin-i18n", "backend-service"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`sea-lantern-i18n-core` is the shared backend internationalization crate.

This crate should own:

- embedded locale-table loading
- locale selection and fallback behavior
- translation lookup helpers
- locale-change callback registration
- plugin locale registration and plugin translation merging

This crate should not own:

- frontend i18n bundling or page-local copy decisions
- plugin UI rendering
- Tauri command exposure
- unrelated persistence or config logic

## Module Entry

- `i18n-core/`
  - `src/`
    - `lib.rs`
    - `embedded.rs`
    - `service.rs`
  - `locales/`
  - `Cargo.toml`

## Key Files

[`src/lib.rs`](src/lib.rs): Public re-export surface for backend i18n.
- Re-exports `embedded_table`, `SUPPORTED_LOCALES`, `I18nService`, and `LocaleCallbackToken`.

[`src/embedded.rs`](src/embedded.rs): Embedded locale asset loading.
- `SUPPORTED_LOCALES`: Canonical built-in locale list.
- `embedded_table`: Loads and parses the embedded JSON table for a locale, with fallback behavior.

[`src/service.rs`](src/service.rs): Shared runtime i18n service.
- `I18nService`: Owns translations, active locale, callbacks, plugin locale ownership, and plugin translations.
- `t` / `t_for_locale` / `t_with_options`: Shared translation lookup helpers.
- `register_locale` / `add_plugin_translations` / `remove_plugin_translations`: Plugin i18n integration boundary.
- `on_locale_change` / `remove_locale_callback`: Locale-change notification surface.

## Stable Boundaries

- Keep locale fallback and translation merging behavior centralized here.
- Plugin-provided locale entries should stay additive to the shared service instead of bypassing it.
- Avoid duplicating locale parsing or translation-merging rules in host crates.

## Change Guidance For Agents

- If translation lookup semantics change, update both direct lookups and plugin-translation merge paths together.
- Preserve stable locale IDs unless the rest of the app is updated in the same change.
- Treat locale callback behavior as a shared runtime contract used by multiple host components.

## Validation Checklist

- Run this crate's tests if locale fallback, translation lookup, plugin locale registration, or callback behavior changed.
- Re-check host-side locale consumers if available locale lists or merged translation output changed.
