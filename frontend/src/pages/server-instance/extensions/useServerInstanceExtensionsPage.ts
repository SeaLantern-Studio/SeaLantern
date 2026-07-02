import { computed, shallowRef, watch } from "vue";
import { i18n } from "@language";
import { formatBytes } from "@utils/formatters";
import { useNextInstanceWorkspaceContext } from "@src/composables/useNextInstanceWorkspace";
import {
  serverExtensionsApi,
  type ServerExtensionEntrySummary,
  type ServerExtensionsSummary,
} from "@src/api/serverExtensions";

function createEmptySummary(): ServerExtensionsSummary {
  return {
    has_plugins_dir: false,
    has_mods_dir: false,
    plugin_entries: [],
    mod_entries: [],
    other_entries: [],
  };
}

function normalizeExtensionLabel(name: string): string | null {
  const lowered = name.toLowerCase();
  if (lowered.endsWith(".jar.disabled")) return "JAR.disabled";
  if (lowered.endsWith(".zip.disabled")) return "ZIP.disabled";

  const segments = lowered.split(".");
  if (segments.length < 2) {
    return null;
  }

  return segments.at(-1)?.toUpperCase() ?? null;
}

function formatModifiedAtLabel(value: string | null): string {
  if (!value) {
    return i18n.t("servers.next.instance.extensions.unknown_modified_at");
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat(undefined, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date);
}

export function useServerInstanceExtensionsPage() {
  const workspace = useNextInstanceWorkspaceContext();

  const loading = shallowRef(false);
  const refreshing = shallowRef(false);
  const errorMessage = shallowRef("");
  const loadedOnce = shallowRef(false);
  const summary = shallowRef<ServerExtensionsSummary>(createEmptySummary());

  const serverPath = computed(() => workspace.server.value?.path ?? "");
  const totalEntries = computed(
    () =>
      summary.value.plugin_entries.length +
      summary.value.mod_entries.length +
      summary.value.other_entries.length,
  );
  const hasAnyDirectory = computed(
    () => summary.value.has_plugins_dir || summary.value.has_mods_dir,
  );
  const hasAnyEntries = computed(() => totalEntries.value > 0);
  const hasNoDirectories = computed(() => loadedOnce.value && !hasAnyDirectory.value);
  const hasEmptyDirectories = computed(
    () => loadedOnce.value && hasAnyDirectory.value && !hasAnyEntries.value,
  );
  const isMissingServer = computed(
    () => loadedOnce.value && !serverPath.value && !errorMessage.value,
  );

  async function refresh(manual = false): Promise<void> {
    errorMessage.value = "";

    if (manual) {
      refreshing.value = true;
    } else {
      loading.value = true;
    }

    try {
      await workspace.refreshServerContext();

      if (!workspace.server.value?.path) {
        summary.value = createEmptySummary();
        loadedOnce.value = true;
        return;
      }

      summary.value = await serverExtensionsApi.getServerExtensionsSummary(
        workspace.server.value.path,
      );
      loadedOnce.value = true;
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : String(error);
      loadedOnce.value = true;
    } finally {
      loading.value = false;
      refreshing.value = false;
    }
  }

  function resolveTypeLabel(entry: ServerExtensionEntrySummary): string {
    const extensionLabel = normalizeExtensionLabel(entry.name);

    if (entry.kind === "plugin") {
      return extensionLabel
        ? i18n.t("servers.next.instance.extensions.type_plugin_with_ext", { ext: extensionLabel })
        : i18n.t("servers.next.instance.extensions.type_plugin");
    }

    if (entry.kind === "mod") {
      return extensionLabel
        ? i18n.t("servers.next.instance.extensions.type_mod_with_ext", { ext: extensionLabel })
        : i18n.t("servers.next.instance.extensions.type_mod");
    }

    return extensionLabel
      ? i18n.t("servers.next.instance.extensions.type_other_with_ext", { ext: extensionLabel })
      : i18n.t("servers.next.instance.extensions.type_other");
  }

  watch(
    () => workspace.serverId.value,
    () => {
      void refresh(false);
    },
    { immediate: true },
  );

  return {
    loading,
    refreshing,
    errorMessage,
    summary,
    totalEntries,
    hasAnyDirectory,
    hasAnyEntries,
    hasNoDirectories,
    hasEmptyDirectories,
    isMissingServer,
    refresh,
    formatSize: formatBytes,
    formatModifiedAt: formatModifiedAtLabel,
    resolveTypeLabel,
    pluginsTitle: computed(() => i18n.t("servers.next.instance.extensions.plugins_title")),
    modsTitle: computed(() => i18n.t("servers.next.instance.extensions.mods_title")),
    otherTitle: computed(() => i18n.t("servers.next.instance.extensions.other_title")),
  };
}
