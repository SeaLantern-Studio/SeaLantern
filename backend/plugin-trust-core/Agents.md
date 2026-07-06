---
module-name: sea-lantern-plugin-trust-core
update-time: 2026-07-05
description: Shared plugin permission metadata, trust assessment, install metadata, trusted catalog parsing, and enable-policy logic for SeaLantern.
tag: ["plugin-trust-core", "permissions", "trust", "catalog", "policy"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`sea-lantern-plugin-trust-core` is the shared plugin trust and permission-policy crate.

This crate should own:

- canonical plugin permission metadata and alias normalization
- permission risk grouping and standard-sandbox ceiling rules
- trusted catalog parsing and trust assessment helpers
- plugin install metadata read/write helpers
- plugin enable-confirmation and trust-policy decisions
- plugin tree hashing and integrity checks

This crate should not own:

- plugin UI flows
- Tauri command handlers
- plugin runtime execution
- unrelated plugin package parsing that does not affect trust or permission policy

## Module Entry

- `plugin-trust-core/`
  - `src/`
    - `lib.rs`
    - `permissions.rs`
    - `trust.rs`
  - `Cargo.toml`

## Key Files

[`src/lib.rs`](src/lib.rs): Public re-export surface for permission and trust logic.
- Re-exports permission helpers and trust-assessment types.

[`src/permissions.rs`](src/permissions.rs): Canonical permission metadata and permission-risk helpers.
- `PluginPermissionInfo`: Shared metadata entry loaded from `shared/plugin-permissions.json`.
- `normalize_permission_id` / `normalize_permissions`: Canonical permission normalization.
- `permission_risk_group`: Shared risk grouping for standard, escalated, and trusted-only capabilities.
- `requires_explicit_consent` / `requests_trusted_capabilities`: Shared enable-policy signals.

[`src/trust.rs`](src/trust.rs): Trust catalog, integrity, install metadata, and runtime trust decisions.
- `TrustedCatalogSnapshot` / `TrustedCatalogEntry`: Shared trusted-catalog model.
- `PluginTrustAssessment` / `PluginRuntimeTrustState`: Shared trust-evaluation outputs.
- `PluginEnableRequirement`: Shared result for enable confirmation and blocking rules.
- `read_install_metadata` / `write_install_metadata`: Stable install metadata persistence.
- `compute_plugin_tree_sha256`: Shared integrity-hash helper.

## Stable Boundaries

- Keep permission semantics and trust-policy decisions centralized here.
- If the frontend or host needs a new trust display state, add the underlying shared classification here first.
- The JSON files under `shared/` are part of the effective policy contract; keep parser expectations aligned with them.

## Change Guidance For Agents

- Do not duplicate permission normalization or trust classification logic in `tauri-host`.
- Treat trust-level enums and enable-policy outcomes as shared API contracts.
- If you change install metadata shape, verify both read and write callers together.

## Validation Checklist

- Run this crate's tests if permission metadata, risk grouping, install metadata, or trust classification changed.
- Re-check plugin install and enable flows in `tauri-host` if trust enums, messages, or policy decisions changed.
