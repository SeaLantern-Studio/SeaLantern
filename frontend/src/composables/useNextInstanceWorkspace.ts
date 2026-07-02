import { computed, inject, provide, type ComputedRef, type InjectionKey } from "vue";
import { useRoute, useRouter } from "vue-router";
import { i18n } from "@language";
import { useServerStore } from "@stores/serverStore";
import type { ServerStatusInfo } from "@api/server";
import type { ServerInstance } from "@type/server";
import type {
  NextInstancePlaceholderContent,
  NextInstanceSection,
} from "@src/contracts/instance";
import {
  NEXT_SERVER_INSTANCE_CONFIG_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_EXTENSIONS_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_PLAYERS_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_WORLD_ROUTE_NAME,
  NEXT_SERVERS_ROUTE_NAME,
} from "@src/router/pageMeta";

export interface NextInstanceWorkspaceContextValue {
  serverId: ComputedRef<string>;
  server: ComputedRef<ServerInstance | null>;
  status: ComputedRef<ServerStatusInfo | null>;
  statusLabel: ComputedRef<string>;
  section: ComputedRef<NextInstanceSection>;
  pageTitle: ComputedRef<string>;
  pageSubtitle: ComputedRef<string>;
  placeholderContent: ComputedRef<NextInstancePlaceholderContent | null>;
  refreshServerContext: () => Promise<void>;
}

const NEXT_INSTANCE_WORKSPACE_CONTEXT_KEY: InjectionKey<NextInstanceWorkspaceContextValue> = Symbol(
  "next-instance-workspace-context",
);

function formatStatusLabel(status: ServerStatusInfo["status"] | undefined): string {
  if (status === "Running") return i18n.t("home.running");
  if (status === "Starting") return i18n.t("home.starting");
  if (status === "Stopping") return i18n.t("home.stopping");
  if (status === "Error") return i18n.t("home.error");
  if (status === "Stopped") return i18n.t("home.stopped");
  return i18n.t("common.loading");
}

function buildPlaceholderContent(
  section: NextInstanceSection,
): NextInstancePlaceholderContent | null {
  if (section === "players") {
    return null;
  }

  const titleKeyBySection: Record<Exclude<NextInstanceSection, "players">, string> = {
    extensions: "servers.next.instance.sections.extensions",
    config: "servers.next.instance.sections.config",
    world: "servers.next.instance.sections.world",
  };

  const sectionTitleKey = titleKeyBySection[section as Exclude<NextInstanceSection, "players">];

  return {
    eyebrow: i18n.t("servers.next.instance.placeholder.eyebrow"),
    summary: i18n.t(sectionTitleKey),
    description: i18n.t(`servers.next.instance.placeholder.${section}.description`),
    tracks: [
      {
        title: i18n.t("servers.next.instance.placeholder.track.workspace_title"),
        description: i18n.t(`servers.next.instance.placeholder.${section}.track_workspace`),
      },
      {
        title: i18n.t("servers.next.instance.placeholder.track.boundary_title"),
        description: i18n.t(`servers.next.instance.placeholder.${section}.track_boundary`),
      },
    ],
  };
}

export function useProvideNextInstanceWorkspace() {
  const route = useRoute();
  const router = useRouter();
  const serverStore = useServerStore();

  const serverId = computed(() => {
    const rawId = route.params.serverId;
    return typeof rawId === "string" ? rawId : "";
  });

  const section = computed<NextInstanceSection>(() => {
    if (route.name === NEXT_SERVER_INSTANCE_EXTENSIONS_ROUTE_NAME) return "extensions";
    if (route.name === NEXT_SERVER_INSTANCE_CONFIG_ROUTE_NAME) return "config";
    if (route.name === NEXT_SERVER_INSTANCE_WORLD_ROUTE_NAME) return "world";
    return "players";
  });

  const server = computed(() => serverStore.getServerById(serverId.value));
  const status = computed(() => serverStore.statuses[serverId.value] ?? null);
  const statusLabel = computed(() => formatStatusLabel(status.value?.status));
  const pageTitle = computed(() => {
    switch (section.value) {
      case "extensions":
        return i18n.t("servers.next.instance.sections.extensions");
      case "config":
        return i18n.t("servers.next.instance.sections.config");
      case "world":
        return i18n.t("servers.next.instance.sections.world");
      default:
        return i18n.t("servers.next.instance.sections.players");
    }
  });

  const pageSubtitle = computed(() => {
    if (!server.value) {
      return i18n.t("servers.next.instance.not_found_description");
    }

    return i18n.t("servers.next.instance.page_subtitle", {
      server: server.value.name,
      status: statusLabel.value,
    });
  });

  const placeholderContent = computed(() => buildPlaceholderContent(section.value));

  async function refreshServerContext(): Promise<void> {
    if (!serverStore.servers.some((item) => item.id === serverId.value)) {
      await serverStore.refreshList();
    }

    if (!serverId.value) {
      return;
    }

    serverStore.setCurrentServer(serverId.value);
    await serverStore.refreshStatus(serverId.value);
  }

  const context: NextInstanceWorkspaceContextValue = {
    serverId,
    server,
    status,
    statusLabel,
    section,
    pageTitle,
    pageSubtitle,
    placeholderContent,
    refreshServerContext,
  };

  provide(NEXT_INSTANCE_WORKSPACE_CONTEXT_KEY, context);

  return {
    ...context,
    router,
    serverStore,
    backToServersRoute: { name: NEXT_SERVERS_ROUTE_NAME },
    playersRoute: computed(() => ({
      name: NEXT_SERVER_INSTANCE_PLAYERS_ROUTE_NAME,
      params: { serverId: serverId.value },
    })),
    extensionsRoute: computed(() => ({
      name: NEXT_SERVER_INSTANCE_EXTENSIONS_ROUTE_NAME,
      params: { serverId: serverId.value },
    })),
    configRoute: computed(() => ({
      name: NEXT_SERVER_INSTANCE_CONFIG_ROUTE_NAME,
      params: { serverId: serverId.value },
    })),
    worldRoute: computed(() => ({
      name: NEXT_SERVER_INSTANCE_WORLD_ROUTE_NAME,
      params: { serverId: serverId.value },
    })),
  };
}

export function useNextInstanceWorkspaceContext(): NextInstanceWorkspaceContextValue {
  const context = inject(NEXT_INSTANCE_WORKSPACE_CONTEXT_KEY, null);
  if (!context) {
    throw new Error("Next instance workspace context is not available");
  }
  return context;
}
