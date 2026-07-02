import {
  clearSettingsRecord,
  buildPluginSettingsForm,
  findDependentPlugins,
} from "./pluginSettingsShared";
import { serializeSettingsRecord } from "./pluginSettingsShared";
import type { PluginInfo } from "@type/plugin";
import type { PluginSettingsRecord } from "./pluginSettingsShared";

interface UsePluginDependentSettingsOptions {
  pluginId: () => string;
  plugins: () => PluginInfo[];
  showDependents: () => boolean;
  hasPlugin: () => boolean;
  getPluginSettings: (pluginId: string) => Promise<Record<string, unknown>>;
  dependentPlugins: { value: PluginInfo[] };
  dependentSettingsForms: Record<string, PluginSettingsRecord>;
  dependentSettingsSnapshots: Record<string, string>;
}

export function usePluginDependentSettings(options: UsePluginDependentSettingsOptions) {
  async function loadDependentPlugins() {
    if (!options.hasPlugin() || !options.showDependents()) {
      options.dependentPlugins.value = [];
      clearSettingsRecord(options.dependentSettingsForms);
      clearSettingsRecord(options.dependentSettingsSnapshots);
      return;
    }

    const candidates = findDependentPlugins(options.plugins(), options.pluginId());
    const results = await Promise.all(
      candidates
        .filter((candidate) => candidate.manifest.settings?.length)
        .map(async (candidate) => ({
          plugin: candidate,
          form: buildPluginSettingsForm(
            candidate.manifest.settings,
            await options.getPluginSettings(candidate.manifest.id),
          ),
        })),
    );

    options.dependentPlugins.value = results.map((item) => item.plugin);
    clearSettingsRecord(options.dependentSettingsForms);
    clearSettingsRecord(options.dependentSettingsSnapshots);

    for (const { plugin, form } of results) {
      options.dependentSettingsForms[plugin.manifest.id] = form;
      options.dependentSettingsSnapshots[plugin.manifest.id] = serializeSettingsRecord(form);
    }
  }

  return {
    loadDependentPlugins,
  };
}
