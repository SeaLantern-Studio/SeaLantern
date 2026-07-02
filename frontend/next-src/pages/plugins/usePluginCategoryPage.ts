import { computed } from "vue";
import { useRouter, useRoute } from "vue-router";
import { NEXT_PLUGIN_DETAIL_ROUTE_NAME } from "@next-src/router/pageMeta";
import { usePluginStore } from "@stores/pluginStore";
import { usePluginCategorySettings } from "@views/plugins/usePluginCategorySettings";

export function usePluginCategoryPage() {
  const route = useRoute();
  const router = useRouter();
  const pluginStore = usePluginStore();

  const pluginId = computed(() => {
    return typeof route.params.pluginId === "string" ? route.params.pluginId : "";
  });

  const categorySettings = usePluginCategorySettings({
    pluginId: () => pluginId.value,
  });

  const iconUrl = computed(() => {
    return pluginId.value ? (pluginStore.icons[pluginId.value] ?? null) : null;
  });

  async function openDetailPage(): Promise<void> {
    if (!pluginId.value) {
      return;
    }

    await router.push({
      name: NEXT_PLUGIN_DETAIL_ROUTE_NAME,
      params: { pluginId: pluginId.value },
    });
  }

  function updateMainField(key: string, value: string | number | boolean) {
    categorySettings.updateSettingsField(categorySettings.settingsForm, key, value);
  }

  function updateDependentField(
    pluginIdValue: string,
    key: string,
    value: string | number | boolean,
  ) {
    const form = categorySettings.dependentSettingsForms[pluginIdValue];
    if (!form) {
      return;
    }

    categorySettings.updateSettingsField(form, key, value);
  }

  return {
    iconUrl,
    categorySettings,
    openDetailPage,
    updateMainField,
    updateDependentField,
  };
}
