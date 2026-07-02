import { computed } from "vue";
import { i18n } from "@language";
import { useShellRuntimeStatus } from "@composables/useShellRuntimeStatus";

export function useUiShellPanel() {
  const runtime = useShellRuntimeStatus({ logScope: "UiShellPanel" });

  const shellFacts = computed(() => [
    {
      label: i18n.t("settings.next.shell.current_label"),
      value: runtime.currentShellName.value,
    },
    {
      label: i18n.t("settings.next.shell.startup_label"),
      value: i18n.t("settings.next.shell.startup_value"),
    },
    {
      label: i18n.t("settings.next.shell.status_label"),
      value: runtime.safeMode.value
        ? i18n.t("settings.next.shell.safe_mode_on")
        : i18n.t("settings.next.shell.safe_mode_off"),
    },
  ]);

  return {
    ...runtime,
    shellFacts,
  };
}
