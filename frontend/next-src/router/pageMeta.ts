import { AUTH_ROUTE_NAME } from "@router/authRoute";
import {
  isNextPageKind,
  type NextPageKind,
  type NextProtectedPageKind,
  type NextRoutePageContract,
} from "../contracts/page";

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

export const NEXT_HOME_ROUTE_NAME = "next-home";
export const NEXT_SERVERS_ROUTE_NAME = "next-servers";
export const NEXT_PLUGINS_ROUTE_NAME = "next-plugins";
export const NEXT_SETTINGS_ROUTE_NAME = "next-settings";

export const NEXT_AUTH_ROUTE: NextRouteDefinition = {
  path: "/auth",
  name: AUTH_ROUTE_NAME,
  meta: definePublicPageMeta({
    pageKind: "auth",
    titleKey: "auth.heading",
  }),
};

export const NEXT_PROTECTED_ROUTES: NextProtectedRouteDefinition[] = [
  {
    path: "/",
    name: NEXT_HOME_ROUTE_NAME,
    meta: defineProtectedPageMeta({
      pageKind: "home",
      titleKey: "shell.page_title",
      navLabelKey: "shell.nav_home",
    }),
  },
  {
    path: "/servers",
    name: NEXT_SERVERS_ROUTE_NAME,
    meta: defineProtectedPageMeta({
      pageKind: "servers",
      titleKey: "shell.nav_servers",
      navLabelKey: "shell.nav_servers",
    }),
  },
  {
    path: "/plugins",
    name: NEXT_PLUGINS_ROUTE_NAME,
    meta: defineProtectedPageMeta({
      pageKind: "plugins",
      titleKey: "shell.nav_plugins",
      navLabelKey: "shell.nav_plugins",
    }),
  },
  {
    path: "/settings",
    name: NEXT_SETTINGS_ROUTE_NAME,
    meta: defineProtectedPageMeta({
      pageKind: "settings",
      titleKey: "shell.nav_settings",
      navLabelKey: "shell.nav_settings",
    }),
  },
];

export const NEXT_SHELL_ENTRY_PATHS = Object.freeze([
  NEXT_AUTH_ROUTE.path,
  ...NEXT_PROTECTED_ROUTES.map((route) => route.path),
]) as readonly string[];

const NEXT_ROUTE_META_BY_KIND: Record<NextPageKind, NextRoutePageMeta> = {
  auth: NEXT_AUTH_ROUTE.meta,
  home: NEXT_PROTECTED_ROUTES[0].meta,
  servers: NEXT_PROTECTED_ROUTES[1].meta,
  plugins: NEXT_PROTECTED_ROUTES[2].meta,
  settings: NEXT_PROTECTED_ROUTES[3].meta,
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
  return NEXT_PROTECTED_ROUTES.find((route) => route.name === name) ?? null;
}
