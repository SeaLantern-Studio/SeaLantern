import { computed, onMounted, reactive, shallowRef, watch } from "vue";
import { useGlobalMessage } from "@composables/useMessage";
import { i18n } from "@language";
import { useSettingsStore } from "@stores/settingsStore";
import { normalizeAppError } from "@utils/appError";

type CloseAction = "ask" | "minimize" | "close";
type GeneralSettingField =
  | "closeAction"
  | "closeServersOnExit"
  | "closeServersOnUpdate"
  | "autoAcceptEula";

function resolveCloseAction(value: string | undefined): CloseAction {
  if (value === "minimize" || value === "close") {
    return value;
  }

  return "ask";
}

export function useGeneralSettingsSection() {
  const settingsStore = useSettingsStore();
  const globalMessage = useGlobalMessage();

  const bootstrapping = shallowRef(!settingsStore.isLoaded);

  const state = reactive({
    closeAction: "ask" as CloseAction,
    closeServersOnExit: true,
    closeServersOnUpdate: true,
    autoAcceptEula: false,
  });

  const pending = reactive<Record<GeneralSettingField, boolean>>({
    closeAction: false,
    closeServersOnExit: false,
    closeServersOnUpdate: false,
    autoAcceptEula: false,
  });

  const closeActionOptions = computed(() => [
    { label: i18n.t("settings.close_action_ask"), value: "ask" },
    { label: i18n.t("settings.close_action_minimize"), value: "minimize" },
    { label: i18n.t("settings.close_action_close"), value: "close" },
  ]);

  watch(
    () =>
      [
        settingsStore.settings.close_action,
        settingsStore.settings.close_servers_on_exit,
        settingsStore.settings.close_servers_on_update,
        settingsStore.settings.auto_accept_eula,
      ] as const,
    ([closeAction, closeServersOnExit, closeServersOnUpdate, autoAcceptEula]) => {
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
    },
    { immediate: true },
  );

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
      }
    } catch (error) {
      rollbackLocalValue();
      const normalized = normalizeAppError(error);
      globalMessage.error(normalized.message || i18n.t("common.message_unknown_error"));
    } finally {
      pending[field] = false;
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

  onMounted(async () => {
    try {
      await settingsStore.ensureLoaded();
    } finally {
      bootstrapping.value = false;
    }
  });

  return {
    bootstrapping,
    pending,
    state,
    closeActionOptions,
    updateCloseAction,
    updateCloseServersOnExit,
    updateCloseServersOnUpdate,
    updateAutoAcceptEula,
  };
}
