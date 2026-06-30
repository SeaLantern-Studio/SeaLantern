import type { Component } from "vue";
import type { RouteLocationRaw } from "vue-router";
import type { NextProtectedPageKind } from "./page";

export interface NextShellNavItem {
  id: NextProtectedPageKind;
  label: string;
  icon: Component;
  to?: RouteLocationRaw;
  active?: boolean;
  disabled?: boolean;
}
