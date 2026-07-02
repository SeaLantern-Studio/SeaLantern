import { shallowRef, watch } from "vue";
import { useRoute } from "vue-router";
import type { RouteLocationNormalizedLoaded } from "vue-router";
import type { NextProtectedPageKind } from "@src/contracts/page";
import {
  getNextProtectedRouteByName,
  resolveNextProtectedRouteDirection,
} from "@src/router/pageMeta";

type RouteViewTransitionName =
  | "next-route-fade"
  | "next-route-forward"
  | "next-route-backward";

function resolveProtectedPageKind(
  route: Pick<RouteLocationNormalizedLoaded, "name" | "meta">,
): NextProtectedPageKind | null {
  const protectedRoute = getNextProtectedRouteByName(route.name);
  if (protectedRoute) {
    return protectedRoute.meta.pageKind;
  }

  const metaPageKind = route.meta?.pageKind;
  if (
    metaPageKind === "home" ||
    metaPageKind === "servers" ||
    metaPageKind === "downloads" ||
    metaPageKind === "tunnel" ||
    metaPageKind === "plugins" ||
    metaPageKind === "paint" ||
    metaPageKind === "developer" ||
    metaPageKind === "settings" ||
    metaPageKind === "about"
  ) {
    return metaPageKind;
  }

  return null;
}

export function buildNextRouteViewKey(
  route: Pick<RouteLocationNormalizedLoaded, "path" | "params">,
): string {
  return `${route.path}::${JSON.stringify(route.params)}`;
}

export function useNextRouteViewTransition() {
  const route = useRoute();
  const transitionName = shallowRef<RouteViewTransitionName>("next-route-fade");
  const previousRouteState = shallowRef({
    name: route.name,
    pageKind: resolveProtectedPageKind(route),
  });

  watch(
    () => ({
      name: route.name,
      pageKind: resolveProtectedPageKind(route),
    }),
    (nextState, previousState) => {
      const fromPageKind = previousState.pageKind ?? previousRouteState.value.pageKind;
      const toPageKind = nextState.pageKind;

      if (!toPageKind || !fromPageKind) {
        transitionName.value = "next-route-fade";
        previousRouteState.value = nextState;
        return;
      }

      const direction = resolveNextProtectedRouteDirection(fromPageKind, toPageKind);
      if (direction === "down") {
        transitionName.value = "next-route-forward";
        previousRouteState.value = nextState;
        return;
      }

      if (direction === "up") {
        transitionName.value = "next-route-backward";
        previousRouteState.value = nextState;
        return;
      }

      transitionName.value = "next-route-fade";
      previousRouteState.value = nextState;
    },
  );
  return {
    transitionName,
  };
}
