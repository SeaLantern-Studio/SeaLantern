# Frontend Maintainer Notes

The frontend is a Vue and TypeScript app under `frontend/`.

## Entry Points

- Runtime entries: `src/main.ts`, `src/main.next.ts`, `src/NextRoot.vue`.
- Tauri command wrappers: `src/api/`.
- Routes and page composition: `src/router/`, `src/pages/`, `src/views/`.
- Shared UI pieces: `src/components/`, `src/layout/`, `src/shell/`.
- Page logic and reusable flows: `src/features/`, `src/composables/`, `src/services/`.
- State: `src/stores/`.
- Types and contracts: `src/types/`, `src/contracts/`.
- Styles and themes: `src/styles/`, `src/themes/`.
- Language resources: `src/language/`.
- Dev helpers: `scripts/`.

## Contracts

- Treat `src/api/*.ts` as the first compatibility surface for backend command changes.
- Keep command names, payload shape, and response typing stable unless the task explicitly changes
  the API.
- Plugin-facing UI snapshots, context menus, sidebars, component mirror data, and permission logs are
  public behavior for plugin flows.
- Minecraft server plugins and SeaLantern Lua plugins are separate product areas; do not merge their
  API wrappers or UI assumptions.

## UI Work

- Follow existing component, store, and CSS patterns before adding new structure.
- Keep operational pages dense, scannable, and predictable.
- Avoid large unrelated rewrites while changing a page flow.
- Check text fit and responsive layout when touching visible UI.
- Use existing theme, language, and message utilities instead of hardcoded one-off behavior.

## Validation

- `pnpm --dir frontend run lint`
- `pnpm --dir frontend run build:check`
- `pnpm --dir frontend run fmt:check`
- Use `pnpm --dir frontend run tauri:dev` for local desktop integration checks.

## Important Details

- Tauri invoke wrappers are the contract edge
- i18n goes through `@language`
- theme definitions live under `src/themes`
- plugin UI runtime has snapshot-based host communication
- `main.next.ts` is the current next-shell entry, not an old `next-src` tree
