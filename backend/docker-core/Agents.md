---
module-name: sea-lantern-docker-core
update-time: 2026-06-05
description: Shared pure Docker launch-shape, preview, JVM-env synthesis, and Docker command interpretation rules for SeaLantern.
tag: ["docker", "launch-preview", "jvm-env", "cpu-policy"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`sea-lantern-docker-core` is the pure shared Docker rules crate.

This crate should own:

- Docker runtime config types reused by host code
- Docker launch spec generation from effective startup config
- `docker run` argument assembly from a launch spec
- Docker launch preview / detail rendering for UI and CLI consumers
- Docker JVM env synthesis, including `JVM_OPTS`, `JVM_XX_OPTS`, memory envs, and `ActiveProcessorCount`
- Docker image-reference parsing and lightweight error classification helpers

This crate should not own:

- actual Docker process execution
- Tauri commands
- server-manager state transitions
- container lifecycle polling
- host-specific filesystem preparation

## Module Entry

- `docker-core/`
  - `src/`
    - `lib.rs`
  - `Cargo.toml`

## Stable Boundaries

- Keep this crate pure and reusable by both runtime execution and preview surfaces.
- Runtime launch and preview must stay derived from the same shared launch-spec logic.
- If Docker JVM env behavior changes here, both actual `docker run` execution and UI preview will change together.
- Do not duplicate Docker launch synthesis in host crates; adapt host models into this crate instead.

## Key Files

[`src/lib.rs`](src/lib.rs): Entire shared Docker rules surface.
- `DockerEffectiveLaunchConfig`: Effective startup inputs already resolved by host code.
- `DockerLaunchSpec`: Shared normalized launch shape used by both execution and preview.
- `DockerLaunchDetail`: UI/CLI-facing preview structure.
- `build_docker_launch_spec` -> `DockerLaunchSpec`: Computes cpuset, effective env, and JVM synthesis metadata.
- `build_docker_run_args` -> `Vec<String>`: Builds final `docker run` arguments from shared spec.
- `build_docker_launch_detail` -> `DockerLaunchDetail`: Builds previewable launch detail from shared spec.
- `build_docker_effective_env` -> `(Vec<(String, String)>, DockerJvmSynthesisMeta)`: Synthesizes `TYPE`, `VERSION`, memory envs, `JVM_OPTS`, and `JVM_XX_OPTS`.
- `resolve_docker_cpuset` / `resolve_docker_active_processor_count`: Reuse server-config-core CPU policy logic for Docker.
- `classify_docker_command_failure`: Maps raw Docker failures into operator-facing categories.

## Change Guidance For Agents

- Prefer adding new Docker launch behavior in `build_docker_launch_spec` and friends instead of patching host runtime code.
- Keep preview output honest: `DockerLaunchDetail` should describe what `build_docker_run_args` would actually execute.
- Preserve sensitive-value redaction in preview surfaces.
- Preserve runtime-env takeover rules:
  - if runtime env already sets `JVM_OPTS`, shared synthesis must not overwrite it
  - if runtime env already sets `JVM_XX_OPTS`, shared synthesis must not overwrite it
- Preserve shared CPU-policy semantics with `server-config-core` unless the user explicitly asks for Docker-only behavior.

## Complexity Notes

- This crate is intentionally central and currently high-complexity.
- If it grows further, prefer file-level splits inside this crate instead of moving rules back into host code.
- Good future split seams are:
  - image-reference parsing
  - launch env synthesis
  - preview rendering
  - Docker command failure interpretation

## Validation Checklist

- Run this crate's tests if launch spec, preview rendering, Docker env synthesis, or Docker error classification changed.
- Re-check host-side Docker runtime tests if public launch behavior changed.
