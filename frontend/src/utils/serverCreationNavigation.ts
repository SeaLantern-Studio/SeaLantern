import type { RouteLocationNormalizedLoaded, RouteLocationRaw } from "vue-router";
import {
  NEXT_SERVER_CREATE_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_CONFIG_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_EXTENSIONS_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_PLAYERS_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_WORLD_ROUTE_NAME,
  NEXT_SERVERS_ROUTE_NAME,
} from "@next-src/router/pageMeta";

type ServerCreationRouteContext = Pick<RouteLocationNormalizedLoaded, "name" | "path">;

export type ServerCreationInstanceSection = "players" | "extensions" | "config" | "world";

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

export function resolveServerCreationEntryRoute(
  _route?: ServerCreationRouteContext,
): RouteLocationRaw {
  return { name: NEXT_SERVER_CREATE_ROUTE_NAME };
}

export function resolveServerCreationCancelRoute(
  _route?: ServerCreationRouteContext,
): RouteLocationRaw {
  return { name: NEXT_SERVERS_ROUTE_NAME };
}

export function resolveCreatedServerRoute(
  _route: ServerCreationRouteContext,
  serverId: string,
  section: ServerCreationInstanceSection = "players",
): RouteLocationRaw {
  return {
    name: resolveInstanceRouteName(section),
    params: { serverId },
  };
}
