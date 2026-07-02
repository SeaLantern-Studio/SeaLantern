import type { RouteLocationNormalizedLoaded, RouteLocationRaw } from "vue-router";
import {
  NEXT_SERVER_CREATE_ROUTE_NAME,
  NEXT_SERVER_IMPORT_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_CONFIG_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_EXTENSIONS_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_PLAYERS_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_WORLD_ROUTE_NAME,
  NEXT_SERVERS_ROUTE_NAME,
} from "@next-src/router/pageMeta";

type ServerCreationRouteContext = Pick<RouteLocationNormalizedLoaded, "name" | "path">;

export type ServerCreationShell = "classic" | "next";
export type ServerCreationInstanceSection = "players" | "extensions" | "config" | "world";

function isNextCreateRoute(route: ServerCreationRouteContext): boolean {
  if (
    route.name === NEXT_SERVER_CREATE_ROUTE_NAME ||
    route.name === NEXT_SERVER_IMPORT_ROUTE_NAME ||
    route.name === NEXT_SERVERS_ROUTE_NAME ||
    route.name === NEXT_SERVER_INSTANCE_ROUTE_NAME ||
    route.name === NEXT_SERVER_INSTANCE_PLAYERS_ROUTE_NAME ||
    route.name === NEXT_SERVER_INSTANCE_EXTENSIONS_ROUTE_NAME ||
    route.name === NEXT_SERVER_INSTANCE_CONFIG_ROUTE_NAME ||
    route.name === NEXT_SERVER_INSTANCE_WORLD_ROUTE_NAME
  ) {
    return true;
  }

  return route.path === "/servers/create" || route.path === "/servers/import";
}

function resolveInstanceRouteName(section: ServerCreationInstanceSection): string {
  switch (section) {
    case "extensions":
      return NEXT_SERVER_INSTANCE_EXTENSIONS_ROUTE_NAME;
    case "config":
      return NEXT_SERVER_INSTANCE_CONFIG_ROUTE_NAME;
    case "world":
      return NEXT_SERVER_INSTANCE_WORLD_ROUTE_NAME;
    default:
      return NEXT_SERVER_INSTANCE_PLAYERS_ROUTE_NAME;
  }
}

export function resolveServerCreationShell(route: ServerCreationRouteContext): ServerCreationShell {
  return isNextCreateRoute(route) ? "next" : "classic";
}

export function resolveServerCreationEntryRoute(
  route: ServerCreationRouteContext,
): RouteLocationRaw {
  if (resolveServerCreationShell(route) === "next") {
    return { name: NEXT_SERVER_CREATE_ROUTE_NAME };
  }

  return { name: "create-server" };
}

export function resolveServerCreationCancelRoute(
  route: ServerCreationRouteContext,
): RouteLocationRaw {
  if (resolveServerCreationShell(route) === "next") {
    return { name: NEXT_SERVERS_ROUTE_NAME };
  }

  return { name: "home" };
}

export function resolveCreatedServerRoute(
  route: ServerCreationRouteContext,
  serverId: string,
  section: ServerCreationInstanceSection = "players",
): RouteLocationRaw {
  if (resolveServerCreationShell(route) === "next") {
    return {
      name: resolveInstanceRouteName(section),
      params: { serverId },
    };
  }

  return {
    name: "console",
    params: { id: serverId },
  };
}
