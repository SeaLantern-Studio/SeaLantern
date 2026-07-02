import { House, Puzzle, Server, Settings2 } from "@lucide/vue";
import { i18n } from "@language";
import { NEXT_PAGE_KINDS, type NextShellPage } from "../contracts/page";
import type { NextShellNavItem } from "../contracts/shell";

export interface NextWorkbenchShellConfig {
  brand: string;
  railLabel: string;
  logoutLabel: string;
  pageMeta: NextShellPage;
  navItems: NextShellNavItem[];
}

export interface NextWorkbenchShellOptions {
  previewMode: boolean;
}

export function buildNextWorkbenchShellConfig(
  options: NextWorkbenchShellOptions,
): NextWorkbenchShellConfig {
  return {
    brand: "SeaLantern",
    railLabel: i18n.t("shell.rail_label"),
    logoutLabel: i18n.t("shell.logout"),
    pageMeta: {
      kind: NEXT_PAGE_KINDS.home,
      title: i18n.t("shell.page_title"),
      subtitle: options.previewMode
        ? i18n.t("shell.page_subtitle_preview")
        : i18n.t("shell.page_subtitle_live"),
    },
    navItems: [
      {
        id: "home",
        label: i18n.t("shell.nav_home"),
        icon: House,
        active: true,
      },
      {
        id: "servers",
        label: i18n.t("shell.nav_servers"),
        icon: Server,
        disabled: true,
      },
      {
        id: "plugins",
        label: i18n.t("shell.nav_plugins"),
        icon: Puzzle,
        disabled: true,
      },
      {
        id: "settings",
        label: i18n.t("shell.nav_settings"),
        icon: Settings2,
        disabled: true,
      },
    ],
  };
}
