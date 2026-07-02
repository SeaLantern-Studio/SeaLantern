import { ref } from "vue";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { usePluginStore } from "@stores/pluginStore";
import { usePluginDependencies } from "@components/views/plugins/usePluginDependencies";
import { usePluginFeedback } from "@components/views/plugins/usePluginFeedback";
import { usePluginSelection } from "@components/views/plugins/usePluginSelection";
import { usePluginSettingsDialog } from "@components/views/plugins/usePluginSettingsDialog";
import { usePluginsInstaller } from "@components/plugin/installer/usePluginsInstaller";

export function usePluginsViewState() {
  const pluginStore = usePluginStore();
  const searchQuery = ref("");

  const installer = usePluginsInstaller();
  const selection = usePluginSelection();
  const dependencies = usePluginDependencies(() => pluginStore.plugins);
  const feedback = usePluginFeedback();

  const settingsDialog = usePluginSettingsDialog({
    getPluginSettings: pluginStore.getPluginSettings,
    setPluginSettings: pluginStore.setPluginSettings,
    logError: (message, details) => {
      pluginLogger.error("PluginsView", message, details);
    },
  });

  return {
    pluginStore,
    searchQuery,
    installer,
    selection,
    dependencies,
    feedback,
    settingsDialog,
  };
}
