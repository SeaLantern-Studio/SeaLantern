import { computed, onBeforeUnmount, onMounted, shallowRef, unref, type Ref } from "vue";
import { onBeforeRouteLeave, useRoute } from "vue-router";
import type { NextProtectedPageKind } from "../contracts/page";
import type { NextShellNavigationDirection } from "../contracts/shell";
import {
  getNextProtectedRouteByName,
  resolveNextProtectedRouteDirection,
} from "../router/pageMeta";

const NAVIGATION_HOLD_DURATION_MS = 260;
const NEXT_SHELL_TRANSITION_DIRECTIONS = new Set<NextShellNavigationDirection>(["up", "down"]);
const navigationDirection = shallowRef<NextShellNavigationDirection | null>(null);
const navigationHold = shallowRef(false);
const pendingNavigation = shallowRef<{
  from: NextProtectedPageKind;
  to: NextProtectedPageKind;
  direction: NextShellNavigationDirection;
} | null>(null);
const pointerInsideRail = shallowRef(false);
const focusWithinRail = shallowRef(false);
const prefersReducedMotion = shallowRef(false);

let holdTimer: ReturnType<typeof setTimeout> | null = null;
let reducedMotionMediaQuery: MediaQueryList | null = null;
let removeReducedMotionListener: (() => void) | null = null;

function clearNavigationHoldTimer(): void {
  if (holdTimer === null) {
    return;
  }

  clearTimeout(holdTimer);
  holdTimer = null;
}

function syncReducedMotionPreference(mediaQuery: MediaQueryList | MediaQueryListEvent): void {
  prefersReducedMotion.value = mediaQuery.matches;
}

function ensureReducedMotionListener(): void {
  if (typeof window === "undefined" || reducedMotionMediaQuery) {
    return;
  }

  reducedMotionMediaQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
  syncReducedMotionPreference(reducedMotionMediaQuery);

  const handler = (event: MediaQueryListEvent) => {
    syncReducedMotionPreference(event);
  };

  if (typeof reducedMotionMediaQuery.addEventListener === "function") {
    reducedMotionMediaQuery.addEventListener("change", handler);
    removeReducedMotionListener = () =>
      reducedMotionMediaQuery?.removeEventListener("change", handler);
    return;
  }

  reducedMotionMediaQuery.addListener(handler);
  removeReducedMotionListener = () => reducedMotionMediaQuery?.removeListener(handler);
}

function releaseNavigationHold(): void {
  clearNavigationHoldTimer();
  navigationHold.value = false;
}

function scheduleNavigationHoldRelease(): void {
  clearNavigationHoldTimer();
  holdTimer = setTimeout(
    () => {
      navigationHold.value = false;
      holdTimer = null;
    },
    prefersReducedMotion.value ? 0 : NAVIGATION_HOLD_DURATION_MS,
  );
}

function prepareNextShellNavigation(from: NextProtectedPageKind, to: NextProtectedPageKind): void {
  const direction = resolveNextProtectedRouteDirection(from, to);
  navigationDirection.value = direction;
  navigationHold.value = direction !== null;

  pendingNavigation.value =
    direction === null
      ? null
      : {
          from,
          to,
          direction,
        };

  if (!navigationHold.value) {
    releaseNavigationHold();
  }
}

function setPointerInsideRail(value: boolean): void {
  pointerInsideRail.value = value;
}

function setFocusWithinRail(value: boolean): void {
  focusWithinRail.value = value;
}

export function useNextShellNavigationTransition(
  currentPageKind: NextProtectedPageKind | Readonly<Ref<NextProtectedPageKind>>,
) {
  const route = useRoute();

  ensureReducedMotionListener();

  onBeforeRouteLeave((to) => {
    const nextProtectedRoute = getNextProtectedRouteByName(to.name);
    if (!nextProtectedRoute) {
      pendingNavigation.value = null;
      navigationDirection.value = null;
      releaseNavigationHold();
      return;
    }

    prepareNextShellNavigation(unref(currentPageKind), nextProtectedRoute.meta.pageKind);
  });

  onMounted(() => {
    const nextProtectedRoute = getNextProtectedRouteByName(route.name);
    const mountedPageKind = nextProtectedRoute?.meta.pageKind ?? unref(currentPageKind);
    const pending = pendingNavigation.value;

    if (navigationHold.value && pending?.to === mountedPageKind) {
      navigationDirection.value = pending.direction;
      scheduleNavigationHoldRelease();
    } else {
      navigationDirection.value = null;
      pendingNavigation.value = null;
      releaseNavigationHold();
    }
  });

  onBeforeUnmount(() => {
    clearNavigationHoldTimer();
    if (removeReducedMotionListener) {
      removeReducedMotionListener();
      removeReducedMotionListener = null;
      reducedMotionMediaQuery = null;
    }
  });

  const railExpanded = computed(
    () => pointerInsideRail.value || focusWithinRail.value || navigationHold.value,
  );
  const pageTransitionDirection = computed(() => navigationDirection.value);
  const pageTransitionClass = computed(() => {
    if (prefersReducedMotion.value || pageTransitionDirection.value === null) {
      return null;
    }

    return "next-shell-frame__page-body--fade-in";
  });

  function handlePageTransitionSettled(): void {
    if (!NEXT_SHELL_TRANSITION_DIRECTIONS.has(pageTransitionDirection.value ?? "down")) {
      return;
    }

    pendingNavigation.value = null;
    navigationDirection.value = null;
    releaseNavigationHold();
  }

  return {
    pageTransitionClass,
    railExpanded,
    setFocusWithinRail,
    setPointerInsideRail,
    handlePageTransitionSettled,
  };
}
