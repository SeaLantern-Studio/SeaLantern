import { computed, ref, type ComputedRef, type Ref } from "vue";
import { i18n } from "@language";

type PendingReloadSide = "current" | "compare" | null;

interface ReloadCompareContext {
  compareMode: Ref<boolean>;
  compareTargetServerName: ComputedRef<string>;
  compareTargetDraftValues: Ref<Record<string, string>>;
  compareTargetLoadedValues: Ref<Record<string, string>>;
  compareTargetSourceDraftText: Ref<string>;
  compareTargetLoadedSourceText: Ref<string>;
  loadCompareProperties: () => Promise<void>;
}

interface UseConfigPropertiesReloadGuardOptions {
  currentServerName: ComputedRef<string>;
  sourceDraftText: Ref<string>;
  loadedSourceText: Ref<string>;
  editValues: Ref<Record<string, string>>;
  loadedValues: Ref<Record<string, string>>;
  getCompareContext: () => ReloadCompareContext | null;
  loadCurrentPropertiesOnly: () => Promise<void>;
}

function areMapValuesEqual(a: Record<string, string>, b: Record<string, string>) {
  const aKeys = Object.keys(a);
  const bKeys = Object.keys(b);

  if (aKeys.length !== bKeys.length) {
    return false;
  }

  for (const key of aKeys) {
    if (a[key] !== b[key]) {
      return false;
    }
  }

  return true;
}

export function useConfigPropertiesReloadGuard(options: UseConfigPropertiesReloadGuardOptions) {
  const showDiscardConfirm = ref(false);
  const pendingReloadSide = ref<PendingReloadSide>(null);

  const currentSideDirty = computed(
    () =>
      options.sourceDraftText.value !== options.loadedSourceText.value ||
      !areMapValuesEqual(options.editValues.value, options.loadedValues.value),
  );

  const compareSideDirty = computed(() => {
    const context = options.getCompareContext();
    if (!context) {
      return false;
    }

    return (
      context.compareTargetSourceDraftText.value !== context.compareTargetLoadedSourceText.value ||
      !areMapValuesEqual(
        context.compareTargetDraftValues.value,
        context.compareTargetLoadedValues.value,
      )
    );
  });

  const discardConfirmTitle = computed(() => {
    if (pendingReloadSide.value === "compare") {
      return "丢弃对照侧修改";
    }
    if (pendingReloadSide.value === "current") {
      return "丢弃当前侧修改";
    }
    return i18n.t("config.discard_title");
  });

  const discardConfirmMessage = computed(() => {
    const context = options.getCompareContext();
    if (pendingReloadSide.value === "compare") {
      return `重新载入将丢弃 ${context?.compareTargetServerName.value || i18n.t("config.compare.target_server")} 的未保存属性修改。`;
    }
    if (pendingReloadSide.value === "current") {
      return `重新载入将丢弃 ${options.currentServerName.value || i18n.t("config.current_server")} 的未保存属性修改。`;
    }
    return i18n.t("config.discard_message");
  });

  function closeDiscardDialog() {
    showDiscardConfirm.value = false;
    pendingReloadSide.value = null;
  }

  async function reloadPropertiesWithGuard() {
    pendingReloadSide.value = "current";
    if (currentSideDirty.value) {
      showDiscardConfirm.value = true;
      return;
    }

    await options.loadCurrentPropertiesOnly();
    pendingReloadSide.value = null;
  }

  async function reloadComparePropertiesWithGuard() {
    const context = options.getCompareContext();
    if (!context?.compareMode.value) {
      return;
    }

    pendingReloadSide.value = "compare";
    if (compareSideDirty.value) {
      showDiscardConfirm.value = true;
      return;
    }

    await context.loadCompareProperties();
    pendingReloadSide.value = null;
  }

  async function confirmReloadDiscard() {
    showDiscardConfirm.value = false;
    const context = options.getCompareContext();
    if (pendingReloadSide.value === "compare" && context) {
      await context.loadCompareProperties();
    } else {
      await options.loadCurrentPropertiesOnly();
    }
    pendingReloadSide.value = null;
  }

  return {
    showDiscardConfirm,
    pendingReloadSide,
    discardConfirmTitle,
    discardConfirmMessage,
    closeDiscardDialog,
    reloadPropertiesWithGuard,
    reloadComparePropertiesWithGuard,
    confirmReloadDiscard,
  };
}
