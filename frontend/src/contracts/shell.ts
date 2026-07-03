import type { Component } from "vue";
import type { RouteLocationRaw } from "vue-router";
import type { NextProtectedPageKind } from "./page";

export type NextShellNavigationDirection = "up" | "down";

export interface NextShellNavItem {
  id: NextProtectedPageKind | string;
  label: string;
  icon: Component;
  to?: RouteLocationRaw;
  active?: boolean;
  disabled?: boolean;
}

export interface NextShellRailPinControl {
  pinned: boolean;
  disabled?: boolean;
  label: string;
}
