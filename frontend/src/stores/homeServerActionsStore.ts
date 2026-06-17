import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { serverApi } from "@api/server";
import { systemApi } from "@api/system";
import { useConsoleStore } from "@stores/consoleStore";
import { useServerStore } from "@stores/serverStore";
import { i18n } from "@language";
import type { ServerInstance } from "@type/server";

function getHomeServerStatusText(status: string | undefined): string {
  switch (status) {
    case "Running":
      return i18n.t("home.running");
    case "Starting":
      return i18n.t("home.starting");
    case "Stopping":
      return i18n.t("home.stopping");
    case "Error":
      return i18n.t("home.error");
    default:
      return i18n.t("home.stopped");
  }
}

export const useHomeServerActionsStore = defineStore("homeServerActions", () => {
  const actionLoading = ref<Record<string, boolean>>({});
  const actionError = ref<string | null>(null);

  const editingServerId = ref<string | null>(null);
  const editName = ref("");
  const editLoading = ref(false);

  const deletingServerId = ref<string | null>(null);
  const deleteServerName = ref("");
  const showDeleteConfirm = ref(false);

  const changingPathServerId = ref<string | null>(null);
  const changePathModalVisible = ref(false);
  const changePathLoading = ref(false);
  const changePathValidationResult = ref<{
    valid: boolean;
    message: string;
    messageKey: string | null;
    jarPath: string | null;
    startupMode: string | null;
  } | null>(null);
  const selectedNewPath = ref("");

  const serverStore = useServerStore();
  const consoleStore = useConsoleStore();

  const recentAlerts = computed(() => {
    const alerts: { server: string; line: string }[] = [];
    for (const [sid, logs] of Object.entries(consoleStore.logs)) {
      const serverName =
        serverStore.servers.find((server) => server.id === sid)?.name || sid.substring(0, 8);
      const filtered = logs
        .filter(
          (line) =>
            line.includes("[ERROR]") ||
            line.includes("[WARN]") ||
            line.includes("FATAL") ||
            line.includes("[STDERR]"),
        )
        .slice(-5);

      for (const line of filtered) {
        alerts.push({ server: serverName, line });
      }
    }

    return alerts.slice(-10);
  });

  async function handleStart(id: string) {
    actionLoading.value[id] = true;
    actionError.value = null;
    try {
      await serverApi.start(id);
      await serverStore.refreshStatus(id);
    } catch (e) {
      actionError.value = String(e);
    } finally {
      actionLoading.value[id] = false;
    }
  }

  async function handleStop(id: string) {
    actionLoading.value[id] = true;
    actionError.value = null;
    try {
      await serverApi.stop(id);
      await serverStore.refreshStatus(id);
    } catch (e) {
      actionError.value = String(e);
    } finally {
      actionLoading.value[id] = false;
    }
  }

  function startEditServerName(server: ServerInstance) {
    editingServerId.value = server.id;
    editName.value = server.name;
  }

  async function saveServerName(serverId: string) {
    if (!serverId || !editName.value.trim()) return;

    editLoading.value = true;
    actionError.value = null;

    try {
      await serverApi.updateServerName(serverId, editName.value.trim());
      const server = serverStore.servers.find((item) => item.id === serverId);
      if (server) {
        server.name = editName.value.trim();
      }
      editingServerId.value = null;
    } catch (e) {
      actionError.value = String(e);
    } finally {
      editLoading.value = false;
    }
  }

  function cancelEdit() {
    editingServerId.value = null;
    editName.value = "";
  }

  function showDeleteConfirmInput(server: ServerInstance) {
    deletingServerId.value = server.id;
    deleteServerName.value = server.name;
    showDeleteConfirm.value = true;
  }

  async function confirmDelete() {
    if (!deletingServerId.value) return;

    try {
      await serverApi.deleteServer(deletingServerId.value);
      closeDeleteConfirm();
      await serverStore.refreshList();
    } catch (e) {
      actionError.value = String(e);
    }
  }

  function cancelDelete() {
    closeDeleteConfirm();
  }

  function closeDeleteConfirm() {
    showDeleteConfirm.value = false;
    deletingServerId.value = null;
    deleteServerName.value = "";
  }

  function showChangePathModal(server: ServerInstance) {
    const status = serverStore.statuses[server.id]?.status;
    if (status === "Running" || status === "Starting") {
      actionError.value = i18n.t("home.change_path_server_running");
      return;
    }

    changingPathServerId.value = server.id;
    selectedNewPath.value = "";
    changePathValidationResult.value = null;
    changePathModalVisible.value = true;
  }

  function closeChangePathModal() {
    changePathModalVisible.value = false;
    changingPathServerId.value = null;
    selectedNewPath.value = "";
    changePathValidationResult.value = null;
  }

  async function validateNewPath() {
    if (!selectedNewPath.value) return;

    changePathLoading.value = true;
    changePathValidationResult.value = null;

    try {
      const result = await serverApi.validateServerPath(selectedNewPath.value);
      changePathValidationResult.value = result;
    } catch (e) {
      changePathValidationResult.value = {
        valid: false,
        message: String(e),
        messageKey: null,
        jarPath: null,
        startupMode: null,
      };
    } finally {
      changePathLoading.value = false;
    }
  }

  async function selectNewPath() {
    try {
      const path = await systemApi.pickFolder();
      if (path) {
        selectedNewPath.value = path;
        await validateNewPath();
      }
    } catch (e) {
      actionError.value = String(e);
    }
  }

  async function confirmChangePath() {
    if (!changingPathServerId.value || !selectedNewPath.value) return;

    if (!changePathValidationResult.value?.valid) {
      await validateNewPath();
      if (!changePathValidationResult.value?.valid) {
        return;
      }
    }

    changePathLoading.value = true;
    try {
      const result = changePathValidationResult.value;
      await serverApi.updateServerPath(
        changingPathServerId.value,
        selectedNewPath.value,
        result?.jarPath || undefined,
        result?.startupMode || undefined,
      );

      await serverStore.refreshList();
      closeChangePathModal();
    } catch (e) {
      actionError.value = String(e);
    } finally {
      changePathLoading.value = false;
    }
  }

  return {
    actionLoading,
    actionError,
    editingServerId,
    editName,
    editLoading,
    deletingServerId,
    deleteServerName,
    showDeleteConfirm,
    changingPathServerId,
    changePathModalVisible,
    changePathLoading,
    changePathValidationResult,
    selectedNewPath,
    recentAlerts,
    handleStart,
    handleStop,
    startEditServerName,
    saveServerName,
    cancelEdit,
    showDeleteConfirmInput,
    confirmDelete,
    cancelDelete,
    closeDeleteConfirm,
    showChangePathModal,
    closeChangePathModal,
    selectNewPath,
    validateNewPath,
    confirmChangePath,
    getStatusText: getHomeServerStatusText,
  };
});
