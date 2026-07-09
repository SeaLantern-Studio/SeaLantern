import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { i18n } from "@language";
import {
  NEXT_PLUGIN_CATEGORY_ROUTE_NAME,
  NEXT_PLUGIN_DETAIL_ROUTE_NAME,
  NEXT_PLUGIN_MARKET_ROUTE_NAME,
  NEXT_PLUGINS_ROUTE_NAME,
} from "@src/router/pageMeta";

type PluginsWorkspaceSectionId = "manage" | "market" | "config";

export function usePluginsWorkspace() {
  const route = useRoute();
  const router = useRouter();

  const pluginId = computed(() => {
    return typeof route.params.pluginId === "string" ? route.params.pluginId : "";
  });

  const pluginsContext = computed(() => {
    return route.query.pluginsContext === "market" ? "market" : "manage";
  });

  const isConfigRoute = computed(() => {
    return (
      route.name === NEXT_PLUGIN_DETAIL_ROUTE_NAME || route.name === NEXT_PLUGIN_CATEGORY_ROUTE_NAME
    );
  });

  const activeSectionId = computed<PluginsWorkspaceSectionId>(() => {
    if (route.name === NEXT_PLUGIN_MARKET_ROUTE_NAME) {
      return "market";
    }

    if (isConfigRoute.value) {
      return "config";
    }

    return "manage";
  });

  const sectionItems = computed(() => {
    if (isConfigRoute.value) {
      return [
        {
          id: "config",
          label: i18n.t("plugins.plugin_settings"),
          description: i18n.t("plugins.next.workspace.config_description"),
        },
      ] as const;
    }

    return [
      {
        id: "manage",
        label: i18n.t("plugins.next.workspace.manage_label"),
        description: i18n.t("plugins.next.workspace.manage_description"),
      },
      {
        id: "market",
        label: i18n.t("plugins.next.workspace.market_label"),
        description: i18n.t("plugins.next.workspace.market_description"),
      },
    ] as const;
  });

  const currentSectionTitle = computed(() => {
    if (activeSectionId.value === "market") {
      return i18n.t("plugins.next.workspace.market_label");
    }

    if (activeSectionId.value === "config") {
      return i18n.t("plugins.plugin_settings");
    }

    return i18n.t("plugins.next.workspace.manage_label");
  });

  const currentSectionDescription = computed(() => {
    if (activeSectionId.value === "market") {
      return i18n.t("plugins.next.workspace.market_description");
    }

    if (activeSectionId.value === "config") {
      return i18n.t("plugins.next.workspace.config_description");
    }

    return i18n.t("plugins.next.workspace.manage_description");
  });

  const backRoute = computed(() => {
    if (pluginsContext.value === "market") {
      return {
        name: NEXT_PLUGIN_MARKET_ROUTE_NAME,
        query: typeof route.query.tag === "string" ? { tag: route.query.tag } : undefined,
      };
    }

    return { name: NEXT_PLUGINS_ROUTE_NAME };
  });

  async function selectSection(sectionId: string): Promise<void> {
    if (sectionId === "market") {
      await router.push({ name: NEXT_PLUGIN_MARKET_ROUTE_NAME });
      return;
    }

    if (sectionId === "manage") {
      await router.push({ name: NEXT_PLUGINS_ROUTE_NAME });
      return;
    }

    if (sectionId === "config" && pluginId.value) {
      const targetRouteName =
        route.name === NEXT_PLUGIN_CATEGORY_ROUTE_NAME
          ? NEXT_PLUGIN_CATEGORY_ROUTE_NAME
          : NEXT_PLUGIN_DETAIL_ROUTE_NAME;

      await router.push({
        name: targetRouteName,
        params: { pluginId: pluginId.value },
        query: route.query,
      });
    }
  }

  async function goBack(): Promise<void> {
    await router.push(backRoute.value);
  }

  return {
    isConfigRoute,
    activeSectionId,
    sectionItems,
    currentSectionTitle,
    currentSectionDescription,
    selectSection,
    goBack,
  };
}
