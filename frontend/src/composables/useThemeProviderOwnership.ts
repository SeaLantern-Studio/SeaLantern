import { computed, onMounted } from "vue";
import { usePluginStore } from "@stores/pluginStore";

export function useThemeProviderOwnership() {
  const pluginStore = usePluginStore();

  const themeProviderPlugin = computed(() => {
    return pluginStore.plugins.find(
      (plugin) =>
        plugin.state === "enabled" &&
        pluginStore.hasCapability(plugin.manifest.id, "theme-provider"),
    );
  });

  const isThemeProviderActive = computed(() => Boolean(themeProviderPlugin.value));
  const themeProviderPluginName = computed(() => themeProviderPlugin.value?.manifest.name || "");

  onMounted(() => {
    if (!pluginStore.plugins.length && !pluginStore.loading) {
      void pluginStore.loadPlugins();
    }
  });

  return {
    themeProviderPlugin,
    isThemeProviderActive,
    themeProviderPluginName,
  };
}
