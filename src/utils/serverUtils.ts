import { ref, computed } from "vue";
import { useServerStore } from "@stores/serverStore";
import { useConsoleStore } from "@stores/consoleStore";
import { serverApi } from "@api/server";
import { i18n } from "@language";
import type { ServerInstance } from "@type/server";

// 服务器管理相关的响应式数据
const actionLoading = ref<Record<string, boolean>>({});
const actionError = ref<string | null>(null);

const editingServerId = ref<string | null>(null);
const editName = ref("");
const editLoading = ref(false);

// 服务器删除确认相关
const deletingServerId = ref<string | null>(null);
const deleteServerName = ref("");
const inputServerName = ref("");
const deleteError = ref<string | null>(null);
const isClosing = ref(false);

const showDeleteConfirm = ref(false);

// 存储
const store = useServerStore();
const consoleStore = useConsoleStore();

/**
 * 格式化字节数
 * @param bytes 字节数
 * @returns 格式化后的字符串
 */
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

/**
 * 格式化服务器路径
 * @param path 完整路径
 * @returns 格式化后的路径（仅显示uuid文件夹名称）
 */
function formatServerPath(path: string): string {
  // 提取路径最后一部分（uuid文件夹）
  const normalizedPath = path.replace(/\\/g, "/");
  const parts = normalizedPath.split("/").filter(Boolean);
  // 返回最后两部分（父文件夹/uuid文件夹）
  if (parts.length >= 2) {
    return parts.slice(-2).join("/");
  }
  return parts.length > 0 ? parts[parts.length - 1] : path;
}

/**
 * 获取服务器状态变体
 * @param status 服务器状态
 * @returns 状态变体
 */
function getStatusVariant(status: string | undefined) {
  switch (status) {
    case "Running":
      return "success" as const;
    case "Starting":
    case "Stopping":
      return "warning" as const;
    case "Error":
      return "error" as const;
    default:
      return "neutral" as const;
  }
}

/**
 * 获取服务器状态文本
 * @param status 服务器状态
 * @returns 状态文本
 */
function getStatusText(status: string | undefined): string {
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

/**
 * 处理服务器启动
 * @param id 服务器ID
 * @returns Promise<void>
 */
async function handleStart(id: string) {
  actionLoading.value[id] = true;
  actionError.value = null;
  try {
    await serverApi.start(id);
    await store.refreshStatus(id);
  } catch (e) {
    actionError.value = String(e);
  } finally {
    actionLoading.value[id] = false;
  }
}

/**
 * 处理服务器停止
 * @param id 服务器ID
 * @returns Promise<void>
 */
async function handleStop(id: string) {
  actionLoading.value[id] = true;
  actionError.value = null;
  try {
    await serverApi.stop(id);
    await store.refreshStatus(id);
  } catch (e) {
    actionError.value = String(e);
  } finally {
    actionLoading.value[id] = false;
  }
}

/**
 * 开始就地编辑服务器名称
 * @param server 服务器实例
 */
function startEditServerName(server: ServerInstance) {
  editingServerId.value = server.id;
  editName.value = server.name;
}

/**
 * 保存服务器名称更改
 * @param serverId 服务器ID
 * @returns Promise<void>
 */
async function saveServerName(serverId: string) {
  if (!serverId || !editName.value.trim()) return;

  editLoading.value = true;
  actionError.value = null;

  try {
    await serverApi.updateServerName(serverId, editName.value.trim());
    // 直接更新本地 store 中的服务器名称，避免刷新整个列表
    const server = store.servers.find((s) => s.id === serverId);
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

/**
 * 取消编辑服务器名称
 */
function cancelEdit() {
  editingServerId.value = null;
  editName.value = "";
}

/**
 * 显示/收回删除确认输入框
 * @param server 服务器实例
 */
function showDeleteConfirmInput(server: ServerInstance) {
  // 如果当前服务器的删除确认输入框已显示，则收回
  if (deletingServerId.value === server.id) {
    cancelDelete();
  } else {
    // 否则显示删除确认输入框
    deletingServerId.value = server.id;
    deleteServerName.value = server.name;
    inputServerName.value = "";
    deleteError.value = null;
  }
}

/**
 * 验证并执行删除
 * @returns Promise<void>
 */
async function confirmDelete() {
  if (!deletingServerId.value) return;

  if (inputServerName.value.trim() !== deleteServerName.value.trim()) {
    deleteError.value = i18n.t("home.delete_error");
    return;
  }

  const serverIdToDelete = deletingServerId.value;

  try {
    await serverApi.deleteServer(serverIdToDelete);
    // 先重置删除状态
    deletingServerId.value = null;
    deleteServerName.value = "";
    inputServerName.value = "";
    deleteError.value = null;
    isClosing.value = false;
    // 然后刷新列表
    await store.refreshList();
  } catch (e) {
    actionError.value = String(e);
  }
}

/**
 * 取消删除
 */
function cancelDelete() {
  if (!deletingServerId.value) return;

  // 添加关闭动画类
  isClosing.value = true;

  // 动画结束后重置状态
  setTimeout(() => {
    deletingServerId.value = null;
    deleteServerName.value = "";
    inputServerName.value = "";
    deleteError.value = null;
    isClosing.value = false;
  }, 300);
}

/**
 * 处理动画结束事件
 * @param event 动画事件
 */
function handleAnimationEnd(event: AnimationEvent) {
  if (event.animationName === "deleteInputCollapse") {
    deletingServerId.value = null;
    deleteServerName.value = "";
    inputServerName.value = "";
    deleteError.value = null;
    isClosing.value = false;
  }
}

/**
 * 处理点击外部区域
 * @param event 鼠标事件
 */
function handleClickOutside(event: MouseEvent) {
  if (!deletingServerId.value) return;

  const target = event.target as HTMLElement;
  // 检查是否点击了删除确认区域或删除按钮
  const isDeleteConfirmArea = target.closest(".delete-confirm-area");
  const isDeleteButton = target.closest(".server-card-actions")?.querySelector("button");

  // 如果没有点击这些元素，则收回输入框
  if (!isDeleteConfirmArea && !isDeleteButton) {
    cancelDelete();
  }
}

/**
 * 最近的警报
 */
const recentAlerts = computed(() => {
  const alerts: { server: string; line: string }[] = [];
  for (const [sid, logs] of Object.entries(consoleStore.logs)) {
    const serverName = store.servers.find((s) => s.id === sid)?.name || sid.substring(0, 8);
    const filtered = logs
      .filter(
        (l) =>
          l.includes("[ERROR]") ||
          l.includes("[WARN]") ||
          l.includes("FATAL") ||
          l.includes("[STDERR]"),
      )
      .slice(-5);
    for (const line of filtered) {
      alerts.push({ server: serverName, line });
    }
  }
  return alerts.slice(-10);
});

/**
 * 关闭删除确认对话框
 */
function closeDeleteConfirm() {
  showDeleteConfirm.value = false;
  deletingServerId.value = null;
  deleteServerName.value = "";
  inputServerName.value = "";
  deleteError.value = null;
}

export {
  // 响应式数据
  actionLoading,
  actionError,
  editingServerId,
  editName,
  editLoading,
  deletingServerId,
  deleteServerName,
  inputServerName,
  deleteError,
  isClosing,
  showDeleteConfirm,

  // 计算属性
  recentAlerts,

  // 工具函数
  formatBytes,
  formatServerPath,
  getStatusVariant,
  getStatusText,

  // 服务器操作函数
  handleStart,
  handleStop,
  startEditServerName,
  saveServerName,
  cancelEdit,

  // 服务器删除函数
  showDeleteConfirmInput,
  confirmDelete,
  cancelDelete,
  handleAnimationEnd,
  closeDeleteConfirm,

  // 其他函数
  handleClickOutside,
};
