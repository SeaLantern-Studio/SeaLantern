import { computed, onMounted, shallowRef } from "vue";
import { i18n } from "@language";
import type { WorkbenchFactItem } from "@src/components/workbench/WorkbenchFactGrid.vue";
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
      value: i18n.t("about.frontend_stack"),
    },
    {
      label: i18n.t("about.backend"),
      value: i18n.t("about.backend_stack"),
    },
    {
      label: i18n.t("about.license"),
      value: i18n.t("about.license_full"),
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
