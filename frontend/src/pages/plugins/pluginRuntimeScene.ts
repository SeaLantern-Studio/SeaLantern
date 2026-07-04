import type { HostBuildFlavor, HostCapabilities } from "@api/system";
import { i18n } from "@language";

export interface PluginRuntimeSceneState {
  kind: "none" | "metadata-only" | "runtime-off";
  buildFlavor: HostBuildFlavor | null;
  bannerTitle: string | null;
  bannerDescription: string | null;
  tagLabel: string | null;
  toggleUnavailableMessage: string | null;
  emptyDescription: string | null;
}

function translateRuntimeCopy(
  key: string,
  fallback: string,
  args?: Record<string, string>,
): string {
  const fullKey = `plugins.next.runtime.${key}`;
  const translated = i18n.t(fullKey, args);
  return translated === fullKey ? fallback : translated;
}

function buildMetadataOnlyScene(buildFlavor: HostBuildFlavor): PluginRuntimeSceneState {
  return {
    kind: "metadata-only",
    buildFlavor,
    bannerTitle: translateRuntimeCopy(
      "metadata_only_title",
      "This is a metadata-only desktop build.",
    ),
    bannerDescription: translateRuntimeCopy(
      "metadata_only_description",
      `This ${buildFlavor} build can show plugin metadata, trust, and permissions, but it cannot start local plugin runtime or enable plugins.`,
      { flavor: buildFlavor },
    ),
    tagLabel: translateRuntimeCopy(
      "metadata_only_tag",
      `${buildFlavor} · metadata only`,
      { flavor: buildFlavor },
    ),
    toggleUnavailableMessage: translateRuntimeCopy(
      "metadata_only_toggle_unavailable",
      "Local plugin runtime is not included in this build, so enable and disable actions are unavailable here.",
    ),
    emptyDescription: translateRuntimeCopy(
      "metadata_only_empty_description",
      `This ${buildFlavor} build can still rescan plugin metadata and inspect trust details, but it does not run local plugin runtime.`,
      { flavor: buildFlavor },
    ),
  };
}

function buildRuntimeOffScene(buildFlavor: HostBuildFlavor | null): PluginRuntimeSceneState {
  const flavor = buildFlavor ?? "custom";
  return {
    kind: "runtime-off",
    buildFlavor,
    bannerTitle: translateRuntimeCopy(
      "runtime_off_title",
      "Plugin runtime is unavailable in this build.",
    ),
    bannerDescription: translateRuntimeCopy(
      "runtime_off_description",
      `This ${flavor} host can still show plugin metadata, trust, and permissions, but local plugin runtime features are unavailable right now.`,
      { flavor },
    ),
    tagLabel: translateRuntimeCopy(
      "runtime_off_tag",
      `${flavor} · runtime unavailable`,
      { flavor },
    ),
    toggleUnavailableMessage: translateRuntimeCopy(
      "runtime_off_toggle_unavailable",
      "Plugin runtime is unavailable in this host build, so enable and disable actions are unavailable here.",
    ),
    emptyDescription: null,
  };
}

export function derivePluginRuntimeScene(
  capabilities: HostCapabilities | null,
): PluginRuntimeSceneState {
  if (!capabilities) {
    return {
      kind: "none",
      buildFlavor: null,
      bannerTitle: null,
      bannerDescription: null,
      tagLabel: null,
      toggleUnavailableMessage: null,
      emptyDescription: null,
    };
  }

  if (capabilities.build_flavor === "desktop-min") {
    return buildMetadataOnlyScene(capabilities.build_flavor);
  }

  if (!capabilities.plugin_runtime.available) {
    return buildRuntimeOffScene(capabilities.build_flavor);
  }

  return {
    kind: "none",
    buildFlavor: capabilities.build_flavor,
    bannerTitle: null,
    bannerDescription: null,
    tagLabel: null,
    toggleUnavailableMessage: null,
    emptyDescription: null,
  };
}
