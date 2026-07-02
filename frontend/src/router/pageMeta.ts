import { AUTH_ROUTE_NAME } from "@router/authRoute";
import {
  isNextPageKind,
  type NextPageKind,
  type NextProtectedPageKind,
  type NextRoutePageContract,
} from "@src/contracts/page";
import type { NextShellNavigationDirection } from "@src/contracts/shell";

export type NextTitleSource = "route.meta.titleKey";

export interface NextRoutePageMeta extends NextRoutePageContract, Record<string, unknown> {
  titleKey: string;
  titleSource: NextTitleSource;
  authRequired: boolean;
  public: boolean;
  navLabelKey?: string;
}

export interface NextProtectedRoutePageMeta extends NextRoutePageMeta {
  pageKind: NextProtectedPageKind;
  authRequired: true;
  public: false;
  navLabelKey: string;
}

export interface NextRouteDefinition {
  name: string;
  path: string;
  meta: NextRoutePageMeta;
}

export interface NextProtectedRouteDefinition {
  name: string;
  path: string;
  meta: NextProtectedRoutePageMeta;
}

const NEXT_TITLE_SOURCE: NextTitleSource = "route.meta.titleKey";

function definePublicPageMeta(input: { pageKind: "auth"; titleKey: string }): NextRoutePageMeta {
  return {
    pageKind: input.pageKind,
    titleKey: input.titleKey,
    titleSource: NEXT_TITLE_SOURCE,
    authRequired: false,
    public: true,
  };
}

function defineProtectedPageMeta(input: {
  pageKind: NextProtectedPageKind;
  titleKey: string;
  navLabelKey: string;
}): NextProtectedRoutePageMeta {
  return {
    pageKind: input.pageKind,
    titleKey: input.titleKey,
    titleSource: NEXT_TITLE_SOURCE,
    authRequired: true,
    public: false,
    navLabelKey: input.navLabelKey,
  };
}

export function createNextProtectedPageMeta(input: {
  pageKind: NextProtectedPageKind;
  titleKey: string;
  navLabelKey: string;
}): NextProtectedRoutePageMeta {
  return defineProtectedPageMeta(input);
}

export const NEXT_HOME_ROUTE_NAME = "next-home";
export const NEXT_SERVERS_ROUTE_NAME = "next-servers";
export const NEXT_DOWNLOADS_ROUTE_NAME = "next-downloads";
export const NEXT_TUNNEL_ROUTE_NAME = "next-tunnel";
export const NEXT_PLUGINS_ROUTE_NAME = "next-plugins";
export const NEXT_PLUGIN_MARKET_ROUTE_NAME = "next-plugin-market";
export const NEXT_PLUGIN_DETAIL_ROUTE_NAME = "next-plugin-detail";
export const NEXT_PLUGIN_CATEGORY_ROUTE_NAME = "next-plugin-category";
export const NEXT_PAINT_ROUTE_NAME = "next-paint";
export const NEXT_DEVELOPER_ROUTE_NAME = "next-developer";
export const NEXT_SETTINGS_ROUTE_NAME = "next-settings";
export const NEXT_ABOUT_ROUTE_NAME = "next-about";
export const NEXT_SERVER_CREATE_ROUTE_NAME = "next-server-create";
export const NEXT_SERVER_IMPORT_ROUTE_NAME = "next-server-import";
export const NEXT_SERVER_INSTANCE_ROUTE_NAME = "next-server-instance";
export const NEXT_SERVER_INSTANCE_PLAYERS_ROUTE_NAME = "next-server-instance-players";
export const NEXT_SERVER_INSTANCE_EXTENSIONS_ROUTE_NAME = "next-server-instance-extensions";
export const NEXT_SERVER_INSTANCE_CONFIG_ROUTE_NAME = "next-server-instance-config";
export const NEXT_SERVER_INSTANCE_WORLD_ROUTE_NAME = "next-server-instance-world";

export const NEXT_LEGACY_CREATE_COMPAT_PATH = "/create";
export const NEXT_LEGACY_ADD_EXISTING_COMPAT_PATH = "/add-existing";
export const NEXT_LEGACY_DOWNLOAD_COMPAT_PATH = "/download";
export const NEXT_LEGACY_CONFIG_COMPAT_PATH = "/config/:serverId?";
export const NEXT_LEGACY_PLAYERS_COMPAT_PATH = "/players/:serverId?";

export const NEXT_HOME_ROUTE: NextProtectedRouteDefinition = {
  path: "/",
  name: NEXT_HOME_ROUTE_NAME,
  meta: defineProtectedPageMeta({
    pageKind: "home",
    titleKey: "shell.page_title",
    navLabelKey: "shell.nav_home",
  }),
};

export const NEXT_SERVERS_ROUTE: NextProtectedRouteDefinition = {
  path: "/servers",
  name: NEXT_SERVERS_ROUTE_NAME,
  meta: defineProtectedPageMeta({
    pageKind: "servers",
    titleKey: "shell.nav_servers",
    navLabelKey: "shell.nav_servers",
  }),
};

export const NEXT_DOWNLOADS_ROUTE: NextProtectedRouteDefinition = {
  path: "/downloads",
  name: NEXT_DOWNLOADS_ROUTE_NAME,
  meta: defineProtectedPageMeta({
    pageKind: "downloads",
    titleKey: "shell.nav_downloads",
    navLabelKey: "shell.nav_downloads",
  }),
};

export const NEXT_TUNNEL_ROUTE: NextProtectedRouteDefinition = {
  path: "/tunnel",
  name: NEXT_TUNNEL_ROUTE_NAME,
  meta: defineProtectedPageMeta({
    pageKind: "tunnel",
    titleKey: "common.tunnel",
    navLabelKey: "shell.nav_tunnel",
  }),
};

export const NEXT_PLUGINS_ROUTE: NextProtectedRouteDefinition = {
  path: "/plugins",
  name: NEXT_PLUGINS_ROUTE_NAME,
  meta: defineProtectedPageMeta({
    pageKind: "plugins",
    titleKey: "shell.nav_plugins",
    navLabelKey: "shell.nav_plugins",
  }),
};

export const NEXT_PAINT_ROUTE: NextProtectedRouteDefinition = {
  path: "/paint",
  name: NEXT_PAINT_ROUTE_NAME,
  meta: defineProtectedPageMeta({
    pageKind: "paint",
    titleKey: "common.personalize",
    navLabelKey: "shell.nav_paint",
  }),
};

export const NEXT_PLUGIN_MARKET_ROUTE: NextProtectedRouteDefinition = {
  path: "/market",
  name: NEXT_PLUGIN_MARKET_ROUTE_NAME,
  meta: defineProtectedPageMeta({
    pageKind: "plugins",
    titleKey: "market.title",
    navLabelKey: "shell.nav_plugins",
  }),
};

export const NEXT_PLUGIN_DETAIL_ROUTE: NextProtectedRouteDefinition = {
  path: "/plugin/:pluginId",
  name: NEXT_PLUGIN_DETAIL_ROUTE_NAME,
  meta: defineProtectedPageMeta({
    pageKind: "plugins",
    titleKey: "plugins.plugin_settings",
    navLabelKey: "shell.nav_plugins",
  }),
};

export const NEXT_PLUGIN_CATEGORY_ROUTE: NextProtectedRouteDefinition = {
  path: "/plugin-category/:pluginId",
  name: NEXT_PLUGIN_CATEGORY_ROUTE_NAME,
  meta: defineProtectedPageMeta({
    pageKind: "plugins",
    titleKey: "plugins.plugin_category",
    navLabelKey: "shell.nav_plugins",
  }),
};

export const NEXT_DEVELOPER_ROUTE: NextProtectedRouteDefinition = {
  path: "/developer",
  name: NEXT_DEVELOPER_ROUTE_NAME,
  meta: defineProtectedPageMeta({
    pageKind: "developer",
    titleKey: "shell.nav_developer",
    navLabelKey: "shell.nav_developer",
  }),
};

export const NEXT_SETTINGS_ROUTE: NextProtectedRouteDefinition = {
  path: "/settings",
  name: NEXT_SETTINGS_ROUTE_NAME,
  meta: defineProtectedPageMeta({
    pageKind: "settings",
    titleKey: "shell.nav_settings",
    navLabelKey: "shell.nav_settings",
  }),
};

export const NEXT_ABOUT_ROUTE: NextProtectedRouteDefinition = {
  path: "/about",
  name: NEXT_ABOUT_ROUTE_NAME,
  meta: defineProtectedPageMeta({
    pageKind: "about",
    titleKey: "common.about",
    navLabelKey: "common.about",
  }),
};

export const NEXT_AUTH_ROUTE: NextRouteDefinition = {
  path: "/auth",
  name: AUTH_ROUTE_NAME,
  meta: definePublicPageMeta({
    pageKind: "auth",
    titleKey: "auth.heading",
  }),
};

export const NEXT_PROTECTED_ROUTES: NextProtectedRouteDefinition[] = [
  NEXT_HOME_ROUTE,
  NEXT_SERVERS_ROUTE,
  NEXT_DOWNLOADS_ROUTE,
  NEXT_TUNNEL_ROUTE,
  NEXT_PLUGINS_ROUTE,
  NEXT_PAINT_ROUTE,
  NEXT_DEVELOPER_ROUTE,
  NEXT_SETTINGS_ROUTE,
  NEXT_ABOUT_ROUTE,
];

export const NEXT_SERVER_CREATE_ROUTE: NextProtectedRouteDefinition = {
  path: "/servers/create",
  name: NEXT_SERVER_CREATE_ROUTE_NAME,
  meta: defineProtectedPageMeta({
    pageKind: "servers",
    titleKey: "common.create_server",
    navLabelKey: "shell.nav_servers",
  }),
};

export const NEXT_SERVER_IMPORT_ROUTE: NextProtectedRouteDefinition = {
  path: "/servers/import",
  name: NEXT_SERVER_IMPORT_ROUTE_NAME,
  meta: defineProtectedPageMeta({
    pageKind: "servers",
    titleKey: "common.add_existing_server",
    navLabelKey: "shell.nav_servers",
  }),
};

export const NEXT_AUXILIARY_PROTECTED_ROUTES: NextProtectedRouteDefinition[] = [
  NEXT_SERVER_CREATE_ROUTE,
  NEXT_SERVER_IMPORT_ROUTE,
  NEXT_PLUGIN_MARKET_ROUTE,
  NEXT_PLUGIN_DETAIL_ROUTE,
  NEXT_PLUGIN_CATEGORY_ROUTE,
];

export const NEXT_LEGACY_COMPAT_ENTRY_PATHS = Object.freeze([
  NEXT_LEGACY_CREATE_COMPAT_PATH,
  NEXT_LEGACY_ADD_EXISTING_COMPAT_PATH,
  NEXT_LEGACY_DOWNLOAD_COMPAT_PATH,
]) as readonly string[];

const NEXT_DYNAMIC_SHELL_ENTRY_PATTERNS = Object.freeze([
  /^\/console(?:\/[^/]+)?$/,
  /^\/config(?:\/[^/]+)?$/,
  /^\/players(?:\/[^/]+)?$/,
  /^\/servers\/[^/]+(?:\/(?:players|extensions|config|world))?$/,
  /^\/(?:plugin\/[^/]+|plugin-category\/[^/]+)$/,
]);

export const NEXT_SHELL_ENTRY_PATHS = Object.freeze([
  NEXT_AUTH_ROUTE.path,
  ...NEXT_PROTECTED_ROUTES.map((route) => route.path),
  ...NEXT_AUXILIARY_PROTECTED_ROUTES.map((route) => route.path),
  ...NEXT_LEGACY_COMPAT_ENTRY_PATHS,
]) as readonly string[];

export function isNextShellEntryPath(pathname: string): boolean {
  if (NEXT_SHELL_ENTRY_PATHS.includes(pathname)) {
    return true;
  }

  return NEXT_DYNAMIC_SHELL_ENTRY_PATTERNS.some((pattern) => pattern.test(pathname));
}

const NEXT_ROUTE_META_BY_KIND: Record<NextPageKind, NextRoutePageMeta> = {
  auth: NEXT_AUTH_ROUTE.meta,
  home: NEXT_HOME_ROUTE.meta,
  servers: NEXT_SERVERS_ROUTE.meta,
  downloads: NEXT_DOWNLOADS_ROUTE.meta,
  tunnel: NEXT_TUNNEL_ROUTE.meta,
  plugins: NEXT_PLUGINS_ROUTE.meta,
  paint: NEXT_PAINT_ROUTE.meta,
  developer: NEXT_DEVELOPER_ROUTE.meta,
  settings: NEXT_SETTINGS_ROUTE.meta,
  about: NEXT_ABOUT_ROUTE.meta,
};

const NEXT_PROTECTED_ROUTE_INDEX_BY_KIND: Record<NextProtectedPageKind, number> = {
  home: 0,
  servers: 1,
  downloads: 2,
  tunnel: 3,
  plugins: 4,
  paint: 5,
  developer: 6,
  settings: 7,
  about: 8,
};

export function getNextRoutePageMeta(meta: Record<string, unknown>): NextRoutePageMeta {
  const pageKind = isNextPageKind(meta.pageKind) ? meta.pageKind : "home";
  const fallback = NEXT_ROUTE_META_BY_KIND[pageKind];

  return {
    pageKind,
    titleKey: typeof meta.titleKey === "string" ? meta.titleKey : fallback.titleKey,
    titleSource: meta.titleSource === NEXT_TITLE_SOURCE ? NEXT_TITLE_SOURCE : fallback.titleSource,
    authRequired:
      typeof meta.authRequired === "boolean" ? meta.authRequired : fallback.authRequired,
    public: typeof meta.public === "boolean" ? meta.public : fallback.public,
    navLabelKey: typeof meta.navLabelKey === "string" ? meta.navLabelKey : fallback.navLabelKey,
  };
}

export function getNextProtectedRouteByName(name: unknown): NextProtectedRouteDefinition | null {
  return (
    [...NEXT_PROTECTED_ROUTES, ...NEXT_AUXILIARY_PROTECTED_ROUTES].find(
      (route) => route.name === name,
    ) ?? null
  );
}

export function getNextProtectedRouteOrderIndex(pageKind: NextProtectedPageKind): number {
  return NEXT_PROTECTED_ROUTE_INDEX_BY_KIND[pageKind];
}

export function resolveNextProtectedRouteDirection(
  from: NextProtectedPageKind,
  to: NextProtectedPageKind,
): NextShellNavigationDirection | null {
  const fromIndex = getNextProtectedRouteOrderIndex(from);
  const toIndex = getNextProtectedRouteOrderIndex(to);

  if (fromIndex === toIndex) {
    return null;
  }

  return toIndex > fromIndex ? "down" : "up";
}
