import type { PluginEnableBlockReason, PluginEnableGrantScope, PluginInfo } from "@type/plugin";
import { ref } from "vue";
import { normalizeAppError, resolveErrorMessage } from "@utils/appError";

function getPluginErrorMessage(error: unknown): string {
  const normalized = normalizeAppError(error);
  return normalized.message || resolveErrorMessage(normalized.code, normalized.args);
}

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
    grantScope: "version" as PluginEnableGrantScope,
    title: "",
    message: "",
    confirmText: "",
    confirmVariant: "primary" as "primary" | "danger",
  });

  function buildPermissionWarningContent(
    pluginName: string,
    plugin: PluginInfo | null | undefined,
    grantScope: PluginEnableGrantScope,
    blockReason?: PluginEnableBlockReason,
  ) {
    const isRevoked = blockReason === "revoked" || plugin?.review_status === "revoked" || plugin?.revoked;
    const isIntegrityMismatch = plugin?.integrity_status === "mismatch";
    const isTrusted = plugin?.trust_level_display === "trusted";
    const isUnreviewed = plugin?.trust_level_display === "unreviewed";

    if (isRevoked) {
      return {
        title: "插件信任状态已撤销",
        message: `插件 \"${pluginName}\" 的既有信任状态已被撤销。若继续启用，将按未审查插件处理，并需要你重新承担风险。`,
        confirmText: "仍然启用",
        confirmVariant: "danger" as const,
      };
    }

    if (isIntegrityMismatch) {
      return {
        title: "插件内容已变更",
        message: `插件 \"${pluginName}\" 的当前目录内容与安装/审查时记录不一致，已自动降级为未审查状态。继续启用前请确认你接受该变更。`,
        confirmText: "接受变更并启用",
        confirmVariant: "danger" as const,
      };
    }

    if (isTrusted || grantScope === "hash") {
      return {
        title: "启用受审查插件",
        message: `插件 \"${pluginName}\" 需要基于当前版本与内容哈希再次确认后才能启用。`,
        confirmText: "确认并启用",
        confirmVariant: "primary" as const,
      };
    }

    if (isUnreviewed || grantScope === "version") {
      return {
        title: "启用未审查插件",
        message: `插件 \"${pluginName}\" 当前属于未审查状态。继续启用前，请确认你信任它的来源、内容和所声明权限。`,
        confirmText: "我已知晓并启用",
        confirmVariant: "danger" as const,
      };
    }

    return {
      title: "权限警告",
      message: `插件 \"${pluginName}\" 请求了需要额外确认的权限。`,
      confirmText: "仍然启用",
      confirmVariant: "primary" as const,
    };
  }

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

  function openPermissionWarning(
    pluginId: string,
    pluginName: string,
    permissions: string[],
    grantScope: PluginEnableGrantScope = "version",
    plugin?: PluginInfo | null,
    blockReason?: PluginEnableBlockReason,
  ) {
    const content = buildPermissionWarningContent(pluginName, plugin, grantScope, blockReason);
    permissionWarning.value = {
      show: true,
      pluginId,
      pluginName,
      permissions,
      grantScope,
      title: content.title,
      message: content.message,
      confirmText: content.confirmText,
      confirmVariant: content.confirmVariant,
    };
  }

  function closePermissionWarning() {
    permissionWarning.value.show = false;
  }

  return {
    alertDialog,
    permissionWarning,
    showAlert,
    closeAlertDialog,
    openPermissionWarning,
    closePermissionWarning,
    getErrorMessage: getPluginErrorMessage,
  };
}
