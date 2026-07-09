---
module-name: sealantern-docs-agents-guide
update-time: 2026-06-06
description: Repo-level agent guide for SeaLantern, focused on module discovery, backend crate boundaries, documentation entrypoints, and common safe edit paths.
tag: ["repo-guide", "agents", "docs", "architecture", "workflow"]
---

IF ANY AGENT FIND THESE FILES(Included Example Files) ARE OUTDATE, YOU CAN UPDATE AND REPORT TO USER.

## SeaLantern Repo Agent Guide

This file is the repo-level `Agents.md` for SeaLantern.

Read this first when you need fast orientation across the whole repository instead of one specific backend crate.

Use this file for:

- finding the right module before editing
- understanding the current backend multi-crate layout
- understanding where frontend, host, and shared backend rules live
- identifying the safest entrypoints for common changes

Do not use this file as the only source of truth for one module.

When work becomes module-specific, switch to that module's own `Agents.md` at the crate root.

## Module Entry

- `docs/`
  - `Agents.md`
  - `Agents-Example-Module.md`
  - `STRUCTURE.md`
  - `CONTRIBUTING.md`
  - `cli-server-runtime-guide.md`
  - `language-system.md`
  - `README/`
    - `README-en.md`

## Module Info

[`docs/Agents.md`](./Agents.md): Repo-level guide for agents working in SeaLantern. | Start here for repo-wide orientation before switching to module-local `Agents.md` files.

[`docs/Agents-Example-Module.md`](./Agents-Example-Module.md): Schema example for writing `Agents.md` files. | Follow this format when creating or updating module entry docs.

[`docs/STRUCTURE.md`](./STRUCTURE.md): Human-readable project structure overview. | Use it to understand top-level layout and current backend crate composition.

[`docs/CONTRIBUTING.md`](./CONTRIBUTING.md): Contributor workflow and collaboration expectations. | Read before changing contributor-facing workflows or contribution instructions.

[`docs/cli-server-runtime-guide.md`](./cli-server-runtime-guide.md): Guide for the `sealantern server ...` CLI runtime flows. | Use when editing CLI server lifecycle, local runtime, or Docker runtime flows.

[`docs/language-system.md`](./language-system.md): i18n workflow and language resource guidance. | Use when editing language resources or locale-loading behavior.

[`docs/README/README-en.md`](./README/README-en.md): English README entry now stored under docs. | Update together with root `README.md` when cross-language README navigation or relative links change.

## Current Backend Shape

SeaLantern backend is not a single-crate backend anymore.

Important crate roots include:

- `backend/tauri-host/`: main desktop and host crate
- `backend/runtime-core/`: shared runtime helpers
- `backend/server-config-core/`: startup config, JVM arg, CPU policy, and server.properties rules
- `backend/docker-core/`: Docker launch spec, preview, and Docker env synthesis rules
- `backend/server-local-setup-core/`: local setup, launch target, Java env, and fallback helpers
- `backend/server-installer-core/`: archive, server-core detection, and MC-version inference
- `backend/server-startup-scan-core/`: startup candidate scanning
- `backend/java-installer-core/`: Java runtime download and install flow
- `backend/update-core/`: update check, download, pending install, and install planning
- `backend/server-plugin-core/`: Minecraft server plugin JAR and config scanning
- `backend/server-download-links-core/`: server download link parsing
- `backend/starter-links-core/`: starter installer link cache and resolution
- `backend/docker-entry/`: headless Docker executable entrypoint

Each of these crate roots should have its own `Agents.md`.

## Safe Edit Routing

Use these routing rules before editing:

- Startup config precedence, JVM args, CPU policy parsing:
  go to `backend/server-config-core/Agents.md`
- Docker launch preview, Docker launch args, Docker JVM env synthesis:
  go to `backend/docker-core/Agents.md`
- Local startup path resolution, direct-jar fallback, Java env path helpers:
  go to `backend/server-local-setup-core/Agents.md`
- Tauri commands, app services, plugin runtime, host orchestration:
  go to `backend/tauri-host/Agents.md`
- Headless HTTP, logging, path helpers, status parsing, Tokio setup:
  go to `backend/runtime-core/Agents.md`
- README or docs navigation changes:
  update both the target file and any relative links affected by the move

## Common Change Patterns

[`frontend/package.json`](../frontend/package.json): Frontend version source and frontend scripts. | Update together with Tauri host version files when bumping application version.

[`backend/tauri-host/Cargo.toml`](../backend/tauri-host/Cargo.toml): Main host crate version and dependency wiring. | Update together with frontend version and Tauri config version when changing app version.

[`backend/tauri-host/tauri.conf.json`](../backend/tauri-host/tauri.conf.json): Tauri app metadata and desktop build config. | Keep version and bundle-facing metadata aligned with the main release version.

[`README.md`](../README.md): Root Chinese README. | Update when public-facing navigation, docs entrypoints, or major repo positioning changes.

[`docs/README/README-en.md`](./README/README-en.md): English README. | Keep relative links valid from its current `docs/README/` location.

## Agent Rules For This Repo

- Prefer module-local `Agents.md` over this repo-level file when work becomes specific.
- Prefer shared backend crates over re-implementing logic inside `tauri-host`.
- When moving docs, always re-check relative paths inside the moved file and from files linking to it.
- When asked to commit in a dirty worktree, stage only the files that belong to the current task.
- Keep docs concise and factual; avoid marketing-style language in agent-facing docs.

## Validation Hints

- If you changed only docs, validate links and file paths.
- If you changed crate-local behavior, read that crate's `Agents.md` and run the most relevant checks for that crate.
- If you changed shared rules used by previews and runtime execution, verify both consumption paths still align.
