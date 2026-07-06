---
module-name: sea-lantern-event-core
update-time: 2026-07-05
description: Shared app and server event envelopes, subscriptions, event manager behavior, and consumer registry DTOs for SeaLantern.
tag: ["event-core", "events", "subscriptions", "registry", "backend-contract"]
---

If this file becomes outdated, update it in the same change and tell the user what changed.

## Module Purpose

`sea-lantern-event-core` is the shared event-contract crate.

This crate should own:

- shared app-level and server-level event envelope types
- event subscription filters and normalization rules
- the reusable event manager used by host-side consumers
- named event-consumer registration metadata and registry DTOs

This crate should not own:

- Tauri command handlers
- transport-specific websocket or HTTP adapters
- server process lifecycle orchestration
- frontend-only view models

## Module Entry

- `event-core/`
  - `src/`
    - `lib.rs`
    - `events.rs`
    - `registry.rs`
  - `Cargo.toml`

## Key Files

[`src/lib.rs`](src/lib.rs): Public re-export surface for the event-core crate.
- Re-exports the shared event envelopes, subscriptions, manager, and registry types.

[`src/events.rs`](src/events.rs): Core event types and reusable manager behavior.
- `AppEventEnvelope` / `ServerEventEnvelope`: Canonical event payload shapes.
- `AppEventPayload` / `ServerEventPayload`: Shared typed payload variants.
- `EventConsumer` / `EventConsumerRegistration`: Shared consumer registration surface.
- `EventManager`: Shared in-process event manager for publishing and subscribing.
- `EventConsumerMetadata` / `NamedEventConsumerState`: Stable metadata and state for named consumers.

[`src/registry.rs`](src/registry.rs): DTO and service layer for inspecting and updating named consumers.
- `EventConsumerRegistry`: Read and mutate named consumer state through a stable DTO surface.
- `EventConsumerRegistry*Dto`: Serialized registry views and update requests.

## Stable Boundaries

- Keep this crate transport-agnostic. It should describe events and manage subscriptions, not decide how events are exposed to the outside world.
- If a new host feature needs event filtering or metadata, add the shared rule here first instead of re-encoding it in `tauri-host`.
- Preserve backward compatibility of serialized event fields unless the frontend and host adapters are updated together.

## Change Guidance For Agents

- Prefer extending shared envelope or subscription types here before patching one consumer in isolation.
- Keep app events and server events clearly separated; do not blur their scopes.
- If you change registry DTOs, verify all callers that serialize or deserialize them.

## Validation Checklist

- Run this crate's tests if event payloads, subscription filtering, or registry DTO behavior changed.
- Re-check `tauri-host` consumers if event names, payload fields, or enable/disable behavior changed.
