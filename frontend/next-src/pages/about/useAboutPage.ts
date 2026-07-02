import { computed, onMounted, shallowRef } from "vue";
import { i18n } from "@language";
import type { WorkbenchFactItem } from "@next-src/components/workbench/WorkbenchFactGrid.vue";
import { getAppVersion } from "@utils/version";

export function useAboutPage() {
  const version = shallowRef(i18n.t("common.loading"));

  const summaryFacts = computed<WorkbenchFactItem[]>(() => [
    {
      label: i18n.t("about.version"),
      value: version.value,
    },
    {
      label: i18n.t("about.frontend"),
      value: "Vue 3 + TypeScript",
    },
    {
      label: i18n.t("about.backend"),
      value: "Rust + Tauri 2",
    },
    {
      label: i18n.t("about.license"),
      value: "GPLv3",
    },
  ]);

  onMounted(async () => {
    version.value = await getAppVersion();
  });

  return {
    version,
    summaryFacts,
  };
}
