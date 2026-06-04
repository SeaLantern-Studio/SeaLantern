import { ref } from "vue";
import { normalizeAppError, resolveErrorMessage } from "@utils/appError";

export function usePluginFeedback() {
  const alertDialog = ref({
    show: false,
    title: "",
    message: "",
  });

  const permissionWarning = ref({
    show: false,
    pluginId: "",
    pluginName: "",
    permissions: [] as string[],
  });

  function showAlert(title: string, message: string) {
    alertDialog.value = {
      show: true,
      title,
      message,
    };
  }

  function closeAlertDialog() {
    alertDialog.value.show = false;
  }

  function openPermissionWarning(pluginId: string, pluginName: string, permissions: string[]) {
    permissionWarning.value = {
      show: true,
      pluginId,
      pluginName,
      permissions,
    };
  }

  function closePermissionWarning() {
    permissionWarning.value.show = false;
  }

  function getErrorMessage(error: unknown): string {
    const normalized = normalizeAppError(error);
    return normalized.message || resolveErrorMessage(normalized.code, normalized.args);
  }

  return {
    alertDialog,
    permissionWarning,
    showAlert,
    closeAlertDialog,
    openPermissionWarning,
    closePermissionWarning,
    getErrorMessage,
  };
}
