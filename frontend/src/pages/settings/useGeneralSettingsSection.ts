import { computed, reactive, ref, watch } from "vue";
import { systemApi, type DesktopWebStatus } from "@api/system";
import { useGlobalMessage } from "@composables/useMessage";
import { i18n } from "@language";
import { useI18nStore } from "@stores/i18nStore";
import { useSettingsStore } from "@stores/settingsStore";
import { useUpdateStore } from "@stores/updateStore";
import { isBrowserEnv } from "@api/tauri";
import { normalizeAppError } from "@utils/appError";

type CloseAction = "ask" | "minimize" | "close";
type GeneralSettingField =
  | "language"
  | "closeAction"
  | "closeServersOnExit"
  | "closeServersOnUpdate"
  | "autoAcceptEula"
  | "autoCheckUpdate"
  | "enableDesktopWebUi";

function resolveCloseAction(value: string | undefined): CloseAction {
  if (value === "minimize" || value === "close") {
    return value;
  }

  return "ask";
}

export function useGeneralSettingsSection() {
  const settingsStore = useSettingsStore();
  const i18nStore = useI18nStore();
  const updateStore = useUpdateStore();
  const globalMessage = useGlobalMessage();

  const bootstrapping = computed(() => !settingsStore.isLoaded || settingsStore.isLoading);
  const showDesktopWebToggle = computed(() => !isBrowserEnv());
  const desktopWebStatus = ref<DesktopWebStatus | null>(null);
  const desktopWebStatusLoading = ref(false);
  const desktopWebStatusError = ref(false);

  const state = reactive({
    language: "zh-CN",
    closeAction: "ask" as CloseAction,
    closeServersOnExit: true,
    closeServersOnUpdate: true,
    autoAcceptEula: false,
    autoCheckUpdate: true,
    enableDesktopWebUi: false,
  });

  const pending = reactive<Record<GeneralSettingField, boolean>>({
    language: false,
    closeAction: false,
    closeServersOnExit: false,
    closeServersOnUpdate: false,
    autoAcceptEula: false,
    autoCheckUpdate: false,
    enableDesktopWebUi: false,
  });

  const closeActionOptions = computed(() => [
    { label: i18n.t("settings.close_action_ask"), value: "ask" },
    { label: i18n.t("settings.close_action_minimize"), value: "minimize" },
    { label: i18n.t("settings.close_action_close"), value: "close" },
  ]);
  const languageOptions = computed(() => {
    return i18nStore.localeOptions.map((option) => ({
      label: option.label,
      value: option.code,
    }));
  });
  const desktopWebStatusLabel = computed(() => {
    if (desktopWebStatusLoading.value && !desktopWebStatus.value) {
      return i18n.t("settings.desktop_web_ui_status_loading");
    }

    if (desktopWebStatusError.value && !desktopWebStatus.value) {
      return i18n.t("settings.desktop_web_ui_status_error");
    }

    return desktopWebStatus.value?.running
      ? i18n.t("settings.desktop_web_ui_status_running")
      : i18n.t("settings.desktop_web_ui_status_stopped");
  });
  const desktopWebUrl = computed(() => desktopWebStatus.value?.url ?? "");
  const canCopyDesktopWebUrl = computed(() => desktopWebUrl.value.length > 0);
  const desktopWebStaticDirMissing = computed(
    () => !!desktopWebStatus.value && !desktopWebStatus.value.static_dir_available,
  );

  watch(
    () =>
      [
        settingsStore.settings.language,
        settingsStore.settings.close_action,
        settingsStore.settings.close_servers_on_exit,
        settingsStore.settings.close_servers_on_update,
        settingsStore.settings.auto_accept_eula,
        settingsStore.settings.auto_check_update,
        settingsStore.settings.enable_desktop_web_ui,
      ] as const,
    ([
      language,
      closeAction,
      closeServersOnExit,
      closeServersOnUpdate,
      autoAcceptEula,
      autoCheckUpdate,
      enableDesktopWebUi,
    ]) => {
      if (!pending.language) {
        state.language = language || i18nStore.currentLocale;
      }

      if (!pending.closeAction) {
        state.closeAction = resolveCloseAction(closeAction);
      }

      if (!pending.closeServersOnExit) {
        state.closeServersOnExit = closeServersOnExit;
      }

      if (!pending.closeServersOnUpdate) {
        state.closeServersOnUpdate = closeServersOnUpdate;
      }

      if (!pending.autoAcceptEula) {
        state.autoAcceptEula = autoAcceptEula;
      }

      if (!pending.autoCheckUpdate) {
        state.autoCheckUpdate = autoCheckUpdate;
      }

      if (!pending.enableDesktopWebUi) {
        state.enableDesktopWebUi = enableDesktopWebUi;
      }
    },
    { immediate: true },
  );

  watch(
    () => [showDesktopWebToggle.value, settingsStore.isLoaded] as const,
    ([visible, loaded]) => {
      if (!visible || !loaded) {
        return;
      }

      void refreshDesktopWebStatus(true);
    },
    { immediate: true },
  );

  async function refreshDesktopWebStatus(silent = true): Promise<void> {
    if (!showDesktopWebToggle.value) {
      return;
    }

    desktopWebStatusLoading.value = true;
    desktopWebStatusError.value = false;

    try {
      desktopWebStatus.value = await systemApi.getDesktopWebStatus();
    } catch (error) {
      desktopWebStatusError.value = true;

      if (!silent) {
        const normalized = normalizeAppError(error);
        globalMessage.error(normalized.message || i18n.t("common.message_unknown_error"));
      }
    } finally {
      desktopWebStatusLoading.value = false;
    }
  }

  async function copyDesktopWebUrl(): Promise<void> {
    const url = desktopWebUrl.value;
    if (!url) {
      return;
    }

    try {
      await navigator.clipboard.writeText(url);
      globalMessage.success(i18n.t("settings.desktop_web_ui_url_copied"));
    } catch (error) {
      const normalized = normalizeAppError(error);
      globalMessage.error(normalized.message || i18n.t("common.message_unknown_error"));
    }
  }

  async function saveField(
    field: GeneralSettingField,
    nextValue: boolean | CloseAction,
    applyLocalValue: () => void,
    rollbackLocalValue: () => void,
  ): Promise<void> {
    if (pending[field]) {
      return;
    }

    applyLocalValue();
    pending[field] = true;

    try {
      switch (field) {
        case "closeAction":
          await settingsStore.updatePartial({ close_action: nextValue as CloseAction });
          return;
        case "closeServersOnExit":
          await settingsStore.updatePartial({ close_servers_on_exit: nextValue as boolean });
          return;
        case "closeServersOnUpdate":
          await settingsStore.updatePartial({ close_servers_on_update: nextValue as boolean });
          return;
        case "autoAcceptEula":
          await settingsStore.updatePartial({ auto_accept_eula: nextValue as boolean });
          return;
        case "autoCheckUpdate":
          await settingsStore.updatePartial({ auto_check_update: nextValue as boolean });
          return;
        case "enableDesktopWebUi":
          await settingsStore.updatePartial({ enable_desktop_web_ui: nextValue as boolean });
          await refreshDesktopWebStatus(true);
          return;
      }
    } catch (error) {
      rollbackLocalValue();
      const normalized = normalizeAppError(error);
      globalMessage.error(normalized.message || i18n.t("common.message_unknown_error"));
    } finally {
      pending[field] = false;
    }
  }

  async function updateLanguage(nextValue: string | number): Promise<void> {
    const resolved = String(nextValue);
    if (pending.language || resolved === state.language) {
      return;
    }

    const previousValue = state.language;
    pending.language = true;

    try {
      const applied = await i18nStore.setLocale(resolved);
      if (!applied) {
        throw new Error(`Unsupported locale: ${resolved}`);
      }

      state.language = resolved;
    } catch (error) {
      state.language = previousValue;
      const normalized = normalizeAppError(error);
      globalMessage.error(normalized.message || i18n.t("common.message_unknown_error"));
    } finally {
      pending.language = false;
    }
  }

  function updateCloseAction(nextValue: string | number): void {
    const resolved = resolveCloseAction(String(nextValue));
    const previousValue = state.closeAction;

    void saveField(
      "closeAction",
      resolved,
      () => {
        state.closeAction = resolved;
      },
      () => {
        state.closeAction = previousValue;
      },
    );
  }

  function updateCloseServersOnExit(nextValue: boolean): void {
    const previousValue = state.closeServersOnExit;

    void saveField(
      "closeServersOnExit",
      nextValue,
      () => {
        state.closeServersOnExit = nextValue;
      },
      () => {
        state.closeServersOnExit = previousValue;
      },
    );
  }

  function updateCloseServersOnUpdate(nextValue: boolean): void {
    const previousValue = state.closeServersOnUpdate;

    void saveField(
      "closeServersOnUpdate",
      nextValue,
      () => {
        state.closeServersOnUpdate = nextValue;
      },
      () => {
        state.closeServersOnUpdate = previousValue;
      },
    );
  }

  function updateAutoAcceptEula(nextValue: boolean): void {
    const previousValue = state.autoAcceptEula;

    void saveField(
      "autoAcceptEula",
      nextValue,
      () => {
        state.autoAcceptEula = nextValue;
      },
      () => {
        state.autoAcceptEula = previousValue;
      },
    );
  }

  function updateAutoCheckUpdate(nextValue: boolean): void {
    const previousValue = state.autoCheckUpdate;

    void saveField(
      "autoCheckUpdate",
      nextValue,
      () => {
        state.autoCheckUpdate = nextValue;
      },
      () => {
        state.autoCheckUpdate = previousValue;
      },
    );
  }

  function updateEnableDesktopWebUi(nextValue: boolean): void {
    const previousValue = state.enableDesktopWebUi;

    void saveField(
      "enableDesktopWebUi",
      nextValue,
      () => {
        state.enableDesktopWebUi = nextValue;
      },
      () => {
        state.enableDesktopWebUi = previousValue;
      },
    );
  }

  async function checkForUpdate(): Promise<void> {
    try {
      await updateStore.checkForUpdate();
    } catch (error) {
      const normalized = normalizeAppError(error);
      globalMessage.error(normalized.message || i18n.t("common.message_unknown_error"));
    }
  }

  return {
    bootstrapping,
    showDesktopWebToggle,
    pending,
    state,
    languageOptions,
    closeActionOptions,
    desktopWebStatusLoading,
    desktopWebStatusError,
    desktopWebStatusLabel,
    desktopWebUrl,
    canCopyDesktopWebUrl,
    desktopWebStaticDirMissing,
    updateStore,
    updateLanguage,
    updateCloseAction,
    updateCloseServersOnExit,
    updateCloseServersOnUpdate,
    updateAutoAcceptEula,
    updateAutoCheckUpdate,
    updateEnableDesktopWebUi,
    copyDesktopWebUrl,
    refreshDesktopWebStatus,
    checkForUpdate,
  };
}
