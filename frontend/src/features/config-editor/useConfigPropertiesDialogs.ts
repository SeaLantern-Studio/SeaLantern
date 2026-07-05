import { computed } from "vue";
import { i18n } from "@language";
import type { useConfigPropertiesEditor } from "@src/features/config-editor/useConfigPropertiesEditor";

type PropertiesEditorState = ReturnType<typeof useConfigPropertiesEditor>;

interface UseConfigPropertiesDialogsOptions {
  propertiesEditor: PropertiesEditorState;
  modalWidth: string;
}

export function useConfigPropertiesDialogs(options: UseConfigPropertiesDialogsOptions) {
  const discardDialog = computed(() => ({
    visible: options.propertiesEditor.showDiscardConfirm.value,
    title: options.propertiesEditor.discardConfirmTitle.value,
    message: options.propertiesEditor.discardConfirmMessage.value,
    confirmText: i18n.t("config.discard_confirm"),
    cancelText: i18n.t("common.cancel"),
    confirmVariant: "danger" as const,
  }));

  const saveDiffDialog = computed(() => ({
    visible: options.propertiesEditor.showSaveDiffModal.value,
    title: i18n.t("config.diff_modal_title"),
    width: options.modalWidth,
    closeOnOverlay: !options.propertiesEditor.saving.value,
    saving: options.propertiesEditor.saving.value,
    items: options.propertiesEditor.pendingSaveItemsWithStats.value,
    confirmText: i18n.t("config.confirm_save"),
    cancelText: i18n.t("common.cancel"),
    originalLabel: i18n.t("config.diff_original"),
    savedLabel: i18n.t("config.diff_after_save"),
  }));

  function closeDiscardDialog() {
    options.propertiesEditor.closeDiscardDialog();
  }

  return {
    discardDialog,
    saveDiffDialog,
    confirmReloadDiscard: options.propertiesEditor.confirmReloadDiscard,
    closeDiscardDialog,
    confirmSaveProperties: options.propertiesEditor.confirmSaveProperties,
    closeSaveDiffModal: options.propertiesEditor.closeSaveDiffModal,
  };
}
