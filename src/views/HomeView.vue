<script setup lang="ts">
import { Menu, Clock, Server, Pencil, Folder } from 'lucide-vue-next';
import { ref, onMounted, onUnmounted, computed } from "vue";
import { useRouter } from "vue-router";
import SLCard from "../components/common/SLCard.vue";
import SLButton from "../components/common/SLButton.vue";
import SLBadge from "../components/common/SLBadge.vue";
import SLProgress from "../components/common/SLProgress.vue";
import { useServerStore } from "../stores/serverStore";
import { useConsoleStore } from "../stores/consoleStore";
import { serverApi } from "../api/server";
import { systemApi, type SystemInfo } from "../api/system";
import { i18n } from "../locales";

const router = useRouter();
const store = useServerStore();
const consoleStore = useConsoleStore();

const actionLoading = ref<Record<string, boolean>>({});
const actionError = ref<string | null>(null);

// 编辑服务器名称相关
const editingServerId = ref<string | null>(null);
const editName = ref("");
const editLoading = ref(false);

// 系统信息
const systemInfo = ref<SystemInfo | null>(null);
const cpuUsage = ref(0);
const memUsage = ref(0);
const diskUsage = ref(0);
const cpuHistory = ref<number[]>([]);
const memHistory = ref<number[]>([]);
const statsViewMode = ref<"detail" | "gauge">("gauge");
const statsLoading = ref(true); // 视图模式
let statsTimer: ReturnType<typeof setInterval> | null = null;
let refreshTimer: ReturnType<typeof setInterval> | null = null;

// 一言 API 相关
interface HitokotoResponse {
  id: number;
  hitokoto: string;
  type: string;
  from: string;
  from_who: string | null;
  creator: string;
  creator_uid: number;
  review_status: number;
  uuid: string;
  created_at: string;
}

const currentQuote = ref<{ text: string; author: string }>({ text: "", author: "" });
const displayText = ref("");
const isTyping = ref(false);
const quoteCache = ref<Array<{ text: string; author: string }>>([]);
let typeTimer: ReturnType<typeof setInterval> | null = null;

function typeWriter(text: string, callback?: () => void) {
  if (typeTimer) clearInterval(typeTimer);
  displayText.value = "";
  isTyping.value = true;
  let index = 0;
  typeTimer = setInterval(() => {
    if (index < text.length) {
      displayText.value += text[index];
      index++;
    } else {
      if (typeTimer) clearInterval(typeTimer);
      isTyping.value = false;
      if (callback) callback();
    }
  }, 50);
}

function typeWriterOut(callback?: () => void) {
  if (typeTimer) clearInterval(typeTimer);
  if (!displayText.value) {
    if (callback) callback();
    return;
  }
  isTyping.value = true;
  let chars = displayText.value.split("");
  typeTimer = setInterval(() => {
    if (chars.length > 0) {
      chars.pop();
      displayText.value = chars.join("");
    } else {
      if (typeTimer) clearInterval(typeTimer);
      isTyping.value = false;
      if (callback) callback();
    }
  }, 30);
}

/**
 * 从一言 API 获取名言
 */
async function fetchHitokoto(): Promise<{ text: string; author: string }> {
  console.log("获取一言，当前缓存数量:", quoteCache.value.length);
  // 优先从缓存中获取
  if (quoteCache.value.length > 0) {
    const quote = quoteCache.value.shift();
    console.log("从缓存中获取一言:", quote);
    console.log("缓存剩余数量:", quoteCache.value.length);
    // 异步补充缓存
    replenishCache();
    return quote!;
  }

  try {
    console.log("缓存为空，从API获取一言");
    const response = await fetch("https://v1.hitokoto.cn/?encode=json");
    if (!response.ok) {
      throw new Error("Failed to fetch hitokoto");
    }
    const data: HitokotoResponse = await response.json();
    const quote = {
      text: data.hitokoto,
      author: data.from_who || data.from || i18n.t("common.unknown"),
    };
    console.log("从API获取一言成功:", quote);
    // 补充缓存
    replenishCache();
    return quote;
  } catch (error) {
    console.error("Error fetching hitokoto:", error);
    // 失败时返回默认名言
    const defaultQuote = { text: i18n.t("common.quote_text"), author: "Sea Lantern" };
    console.log("使用默认一言:", defaultQuote);
    return defaultQuote;
  }
}

/**
 * 检查一言是否已在缓存中
 */
function isQuoteInCache(quote: { text: string; author: string }): boolean {
  return quoteCache.value.some((cachedQuote) => cachedQuote.text === quote.text);
}

/**
 * 补充一言缓存，确保缓存中有至少2个一言，且不重复
 */
async function replenishCache() {
  console.log("开始补充缓存，当前缓存数量:", quoteCache.value.length);
  let attempts = 0;
  const maxAttempts = 10;

  while (quoteCache.value.length < 2 && attempts < maxAttempts) {
    try {
      const response = await fetch("https://v1.hitokoto.cn/?encode=json");
      if (!response.ok) {
        throw new Error("Failed to fetch hitokoto");
      }
      const data: HitokotoResponse = await response.json();
      const newQuote = {
        text: data.hitokoto,
        author: data.from_who || data.from || i18n.t("common.unknown"),
      };

      // 检查是否已在缓存中
      if (!isQuoteInCache(newQuote)) {
        quoteCache.value.push(newQuote);
        console.log("补充缓存成功，当前缓存数量:", quoteCache.value.length);
        console.log("新缓存的一言:", newQuote);
      } else {
        console.log("一言已在缓存中，跳过:", newQuote.text.substring(0, 20) + "...");
        attempts++;
      }
    } catch (error) {
      console.error("Error replenishing quote cache:", error);
      break;
    }
  }

  if (attempts >= maxAttempts) {
    console.log("达到最大尝试次数，停止补充缓存");
  }
}

/**
 * 更新名言
 */
async function updateQuote() {
  console.log("触发更新一言");
  if (isTyping.value) {
    console.log("正在打字中，取消更新");
    return;
  }
  // 先打字消失
  typeWriterOut(async () => {
    try {
      console.log("开始获取新一言");
      const newQuote = await fetchHitokoto();
      console.log("获取新一言成功:", newQuote);
      currentQuote.value = newQuote;
      // 再打字出现
      console.log("开始打字显示新一言");
      typeWriter(newQuote.text);
    } catch (error) {
      console.error("Error updating quote:", error);
    }
  });
}

// 初始化打字机效果
async function initQuote() {
  try {
    // 确保缓存中有足够的一言
    await replenishCache();
    const initialQuote = await fetchHitokoto();
    currentQuote.value = initialQuote;
    typeWriter(initialQuote.text);
  } catch (error) {
    console.error("Error initializing quote:", error);
  }
}

// 初始化获取名言
initQuote();

let quoteTimer: ReturnType<typeof setInterval> | null = null;

// 格式化字节
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

// 格式化服务器路径，只显示 servers 目录及其后面的内容
function formatServerPath(path: string): string {
  const serversIndex = path.indexOf("servers/");
  if (serversIndex !== -1) {
    return path.substring(serversIndex);
  }
  // 尝试使用反斜杠查找
  const serversIndexBackslash = path.indexOf("servers\\");
  if (serversIndexBackslash !== -1) {
    return path.substring(serversIndexBackslash);
  }
  return path;
}

// Recent warning/error logs across all servers
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

onMounted(() => {
  // 异步加载服务器列表，不阻塞页面渲染
  const loadServers = async () => {
    try {
      await store.refreshList();
      // 服务器列表加载完成后，异步加载每个服务器的状态
      for (const s of store.servers) {
        await store.refreshStatus(s.id);
      }
    } catch (e) {
      console.error("Failed to load servers:", e);
    }
  };

  // 获取真实系统信息（异步，不阻塞页面渲染）
  const fetchSystemInfo = async () => {
    try {
      const info = await systemApi.getSystemInfo();
      systemInfo.value = info;
      // clamp CPU usage to 0-100 (sysinfo can sometimes return >100%)
      cpuUsage.value = Math.min(100, Math.max(0, Math.round(info.cpu.usage)));
      memUsage.value = Math.min(100, Math.max(0, Math.round(info.memory.usage)));
      diskUsage.value = Math.min(100, Math.max(0, Math.round(info.disk.usage)));
      cpuHistory.value.push(cpuUsage.value);
      memHistory.value.push(memUsage.value);
      if (cpuHistory.value.length > 30) cpuHistory.value.shift();
      if (memHistory.value.length > 30) memHistory.value.shift();
      statsLoading.value = false;
    } catch (e) {
      console.error("Failed to fetch system info:", e);
      statsLoading.value = false;
    }
  };

  // 启动异步加载
  loadServers();
  fetchSystemInfo();

  // 设置定时任务
  statsTimer = setInterval(fetchSystemInfo, 1000);

  // 名言每30秒更新一次
  quoteTimer = setInterval(updateQuote, 30000);

  // Refresh server statuses
  refreshTimer = setInterval(async () => {
    for (const s of store.servers) {
      await store.refreshStatus(s.id);
    }
  }, 3000);

  // 添加全局点击事件监听器，点击空白区域收回删除确认输入框
  document.addEventListener("click", handleClickOutside);
});

onUnmounted(() => {
  if (statsTimer) clearInterval(statsTimer);
  if (refreshTimer) clearInterval(refreshTimer);
  if (quoteTimer) clearInterval(quoteTimer);
  // 移除全局点击事件监听器
  document.removeEventListener("click", handleClickOutside);
});

// 处理点击空白区域的逻辑
function handleClickOutside(event: MouseEvent) {
  if (!deletingServerId.value) return;

  const target = event.target as HTMLElement;
  // 检查是否点击了删除确认输入框或删除按钮
  const isDeleteConfirmInput = target.closest(".delete-confirm-input");
  const isDeleteButton = target.closest(".server-card-actions")?.querySelector("button");

  // 如果没有点击这些元素，则收回输入框
  if (!isDeleteConfirmInput && !isDeleteButton) {
    cancelDelete();
  }
}

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

// 开始就地编辑服务器名称
function startEditServerName(server: any) {
  editingServerId.value = server.id;
  editName.value = server.name;
}

// 保存服务器名称更改
async function saveServerName(serverId: string) {
  if (!serverId || !editName.value.trim()) return;

  editLoading.value = true;
  actionError.value = null;

  try {
    await serverApi.updateServerName(serverId, editName.value.trim());
    await store.refreshList();
    editingServerId.value = null;
  } catch (e) {
    actionError.value = String(e);
  } finally {
    editLoading.value = false;
  }
}

// 取消编辑服务器名称
function cancelEdit() {
  editingServerId.value = null;
  editName.value = "";
}

// 服务器删除确认相关
const deletingServerId = ref<string | null>(null);
const deleteServerName = ref("");
const inputServerName = ref("");
const deleteError = ref<string | null>(null);
const isClosing = ref(false);

// 显示/收回删除确认输入框
function showDeleteConfirmInput(server: any) {
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

// 验证并执行删除
async function confirmDelete() {
  if (!deletingServerId.value) return;

  if (inputServerName.value.trim() !== deleteServerName.value.trim()) {
    deleteError.value = i18n.t("home.delete_error");
    return;
  }

  try {
    await serverApi.deleteServer(deletingServerId.value);
    await store.refreshList();
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
  } catch (e) {
    actionError.value = String(e);
  }
}

// 取消删除
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

// 处理动画结束事件
function handleAnimationEnd(event: AnimationEvent) {
  if (event.animationName === "deleteInputCollapse") {
    deletingServerId.value = null;
    deleteServerName.value = "";
    inputServerName.value = "";
    deleteError.value = null;
    isClosing.value = false;
  }
}
</script>
<template>
  <div class="home-view animate-fade-in-up">
    <!-- Error Banner -->
    <div v-if="actionError" class="error-banner">
      <span>{{ actionError }}</span>
      <button class="error-close" @click="actionError = null">x</button>
    </div>

    <!-- Top Row: Quick Actions + System Stats -->
    <div class="top-row">
      <SLCard
        :title="i18n.t('home.title')"
        :subtitle="i18n.t('home.create_first')"
        class="quick-start-card"
      >
        <div class="quick-actions">
          <SLButton variant="primary" size="lg" @click="router.push('/create')">
            {{ i18n.t("common.create_server") }}
          </SLButton>
        </div>
        <div class="card-spacer"></div>
        <div class="quote-display" @click="updateQuote" :title="i18n.t('common.click_to_refresh')">
          <span v-if="displayText && !isTyping" class="quote-text">「{{ displayText }}」</span>
          <span v-if="currentQuote && !isTyping" class="quote-author"
            >—— {{ currentQuote.author }}</span
          >
          <span v-if="isTyping" class="quote-text">「{{ displayText }}」</span>
          <span v-if="!displayText && !isTyping" class="quote-loading">加载中...</span>
        </div>
      </SLCard>

      <SLCard class="stats-card">
        <template #header>
          <div class="stats-card-header">
            <span class="card-title">{{ i18n.t("home.system_resources") }}</span>
            <button
              class="view-toggle"
              @click="statsViewMode = statsViewMode === 'gauge' ? 'detail' : 'gauge'"
              :title="
                statsViewMode === 'gauge' ? i18n.t('home.detail_view') : i18n.t('home.gauge_view')
              "
            >
              <Menu v-if="statsViewMode === 'gauge'" :size="14" />
              <Clock v-else :size="14" />
            </button>
          </div>
        </template>
        <!-- 加载状态 -->
        <div v-if="statsLoading" class="stats-loading">
          <div class="spinner"></div>
          <span>{{ i18n.t("common.loading") }}</span>
        </div>
        <!-- 仪表盘视图 -->
        <div v-else-if="statsViewMode === 'gauge'" class="gauge-view">
          <div class="gauge-grid">
            <div class="gauge-item">
              <svg class="gauge-svg" viewBox="0 0 36 36">
                <path
                  class="gauge-bg"
                  d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
                  fill="none"
                  stroke-width="3"
                />
                <path
                  class="gauge-fill gauge-cpu"
                  d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
                  fill="none"
                  stroke-width="3"
                  :stroke-dasharray="`${cpuUsage}, 100`"
                />
              </svg>
              <div class="gauge-text">
                <span class="gauge-value">{{ cpuUsage }}%</span>
                <span class="gauge-label">CPU</span>
              </div>
            </div>
            <div class="gauge-item">
              <svg class="gauge-svg" viewBox="0 0 36 36">
                <path
                  class="gauge-bg"
                  d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
                  fill="none"
                  stroke-width="3"
                />
                <path
                  class="gauge-fill gauge-mem"
                  d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
                  fill="none"
                  stroke-width="3"
                  :stroke-dasharray="`${memUsage}, 100`"
                />
              </svg>
              <div class="gauge-text">
                <span class="gauge-value">{{ memUsage }}%</span>
                <span class="gauge-label">内存</span>
              </div>
            </div>
            <div class="gauge-item">
              <svg class="gauge-svg" viewBox="0 0 36 36">
                <path
                  class="gauge-bg"
                  d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
                  fill="none"
                  stroke-width="3"
                />
                <path
                  class="gauge-fill gauge-disk"
                  d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
                  fill="none"
                  stroke-width="3"
                  :stroke-dasharray="`${diskUsage}, 100`"
                />
              </svg>
              <div class="gauge-text">
                <span class="gauge-value">{{ diskUsage }}%</span>
                <span class="gauge-label">磁盘</span>
              </div>
            </div>
          </div>
          <div v-if="systemInfo" class="gauge-details">
            <div class="gauge-detail-item">
              <span class="detail-label">CPU</span
              ><span class="detail-value">{{ systemInfo.cpu.count }} 核心</span>
            </div>
            <div class="gauge-detail-item">
              <span class="detail-label">内存</span
              ><span class="detail-value"
                >{{ formatBytes(systemInfo.memory.used) }} /
                {{ formatBytes(systemInfo.memory.total) }}</span
              >
            </div>
            <div class="gauge-detail-item">
              <span class="detail-label">磁盘</span
              ><span class="detail-value"
                >{{ formatBytes(systemInfo.disk.used) }} /
                {{ formatBytes(systemInfo.disk.total) }}</span
              >
            </div>
          </div>
        </div>
        <!-- 详细视图 -->
        <div v-else class="stats-grid">
          <div class="stat-item">
            <div class="stat-header">
              <span class="stat-label"
                >CPU<span v-if="systemInfo" class="stat-detail">
                  · {{ systemInfo.cpu.count }} 核心</span
                ></span
              >
              <span class="stat-value">{{ cpuUsage }}%</span>
            </div>
            <div class="mini-chart taskmgr-style">
              <svg viewBox="0 0 300 40" class="chart-svg" preserveAspectRatio="none">
                <!-- 网格线 -->
                <g class="grid-lines" stroke="var(--sl-border)" stroke-width="0.5">
                  <line x1="0" y1="8" x2="300" y2="8" />
                  <line x1="0" y1="16" x2="300" y2="16" />
                  <line x1="0" y1="24" x2="300" y2="24" />
                  <line x1="0" y1="32" x2="300" y2="32" />
                </g>
                <!-- 填充区域 -->
                <polygon
                  :points="
                    '0,40 ' +
                    cpuHistory
                      .map(
                        (v, i) =>
                          (cpuHistory.length > 1 ? (i / (cpuHistory.length - 1)) * 300 : 0) +
                          ',' +
                          (40 - v * 0.4),
                      )
                      .join(' ') +
                    ' 300,40'
                  "
                  fill="var(--sl-primary)"
                  fill-opacity="0.15"
                />
                <!-- 曲线 -->
                <polyline
                  :points="
                    cpuHistory
                      .map(
                        (v, i) =>
                          (cpuHistory.length > 1 ? (i / (cpuHistory.length - 1)) * 300 : 0) +
                          ',' +
                          (40 - v * 0.4),
                      )
                      .join(' ')
                  "
                  fill="none"
                  stroke="var(--sl-primary)"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            </div>
          </div>
          <div class="stat-item">
            <div class="stat-header">
              <span class="stat-label"
                >内存<span v-if="systemInfo" class="stat-detail">
                  · {{ formatBytes(systemInfo.memory.used) }} /
                  {{ formatBytes(systemInfo.memory.total) }}</span
                ></span
              >
              <span class="stat-value">{{ memUsage }}%</span>
            </div>
            <div class="mini-chart taskmgr-style">
              <svg viewBox="0 0 300 40" class="chart-svg" preserveAspectRatio="none">
                <!-- 网格线 -->
                <g class="grid-lines" stroke="var(--sl-border)" stroke-width="0.5">
                  <line x1="0" y1="8" x2="300" y2="8" />
                  <line x1="0" y1="16" x2="300" y2="16" />
                  <line x1="0" y1="24" x2="300" y2="24" />
                  <line x1="0" y1="32" x2="300" y2="32" />
                </g>
                <!-- 填充区域 -->
                <polygon
                  :points="
                    '0,40 ' +
                    memHistory
                      .map(
                        (v, i) =>
                          (memHistory.length > 1 ? (i / (memHistory.length - 1)) * 300 : 0) +
                          ',' +
                          (40 - v * 0.4),
                      )
                      .join(' ') +
                    ' 300,40'
                  "
                  fill="var(--sl-success)"
                  fill-opacity="0.15"
                />
                <!-- 曲线 -->
                <polyline
                  :points="
                    memHistory
                      .map(
                        (v, i) =>
                          (memHistory.length > 1 ? (i / (memHistory.length - 1)) * 300 : 0) +
                          ',' +
                          (40 - v * 0.4),
                      )
                      .join(' ')
                  "
                  fill="none"
                  stroke="var(--sl-success)"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            </div>
          </div>
          <div class="stat-item">
            <div class="stat-header">
              <span class="stat-label"
                >磁盘<span v-if="systemInfo" class="stat-detail">
                  · {{ formatBytes(systemInfo.disk.used) }} /
                  {{ formatBytes(systemInfo.disk.total) }}</span
                ></span
              >
              <span class="stat-value">{{ diskUsage }}%</span>
            </div>
            <SLProgress :value="diskUsage" variant="warning" :showPercent="false" />
          </div>
        </div>
      </SLCard>
    </div>

    <!-- Server List -->
    <div class="section-header">
      <h3 class="section-title">
        {{ i18n.t("home.title") }}
        <span class="server-count">{{ store.servers.length }}</span>
      </h3>
    </div>

    <div v-if="store.loading" class="loading-state">
      <div class="spinner"></div>
      <span>{{ i18n.t("common.loading") }}</span>
    </div>

    <div v-else-if="store.servers.length === 0" class="empty-state">
      <Server :size="64" :color="'var(--sl-text-tertiary)'" stroke-width="1" />
      <p class="text-body">{{ i18n.t("home.no_servers") }}</p>
      <p class="text-caption">{{ i18n.t("home.create_first") }}</p>
    </div>

    <div v-else class="server-grid">
      <div v-for="server in store.servers" :key="server.id" class="server-card glass-card">
        <div class="server-card-header">
          <div class="server-info">
            <div class="server-name-container">
              <template v-if="editingServerId === server.id">
                <div class="inline-edit">
                  <input
                    type="text"
                    v-model="editName"
                    class="server-name-input"
                    @keyup.enter="saveServerName(server.id)"
                    @keyup.esc="cancelEdit"
                    @blur="saveServerName(server.id)"
                    ref="editInput"
                  />
                  <div class="inline-edit-actions">
                    <button
                      class="inline-edit-btn save"
                      @click="saveServerName(server.id)"
                      :disabled="!editName.trim() || editLoading"
                      :class="{ loading: editLoading }"
                    >
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      >
                        <polyline points="20 6 9 17 4 12" />
                      </svg>
                    </button>
                    <button
                      class="inline-edit-btn cancel"
                      @click="cancelEdit"
                      :disabled="editLoading"
                    >
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      >
                        <line x1="18" y1="6" x2="6" y2="18" />
                        <line x1="6" y1="6" x2="18" y2="18" />
                      </svg>
                    </button>
                  </div>
                </div>
              </template>
              <template v-else>
                <h4 class="server-name">{{ server.name }}</h4>
                <button
                  class="edit-server-name"
                  @click="startEditServerName(server)"
                  :title="i18n.t('common.edit_server_name')"
                >
                  <Pencil :size="16" />
                </button>
              </template>
            </div>
            <div class="server-meta">
              <span>{{ server.core_type }}</span>
              <span>端口 {{ server.port }}</span>
              <span>{{ server.max_memory }}MB</span>
            </div>
          </div>
          <SLBadge
            :text="getStatusText(store.statuses[server.id]?.status)"
            :variant="getStatusVariant(store.statuses[server.id]?.status)"
            size="large"
          />
        </div>

        <div
          class="server-card-path text-mono text-caption"
          :title="server.jar_path"
          @click="systemApi.openFolder(server.path)"
        >
          <span class="server-path-text">{{ formatServerPath(server.jar_path) }}</span>
          <Folder class="folder-icon" :size="16" />
        </div>

        <div class="server-card-actions">
          <SLButton
            v-if="
              store.statuses[server.id]?.status === 'Stopped' ||
              store.statuses[server.id]?.status === 'Error' ||
              !store.statuses[server.id]?.status
            "
            variant="primary"
            size="sm"
            :loading="actionLoading[server.id]"
            :disabled="actionLoading[server.id] || store.statuses[server.id]?.status === 'Stopping'"
            @click="handleStart(server.id)"
            >{{ i18n.t("home.start") }}</SLButton
          >
          <SLButton
            v-else
            variant="danger"
            size="sm"
            :loading="actionLoading[server.id]"
            :disabled="actionLoading[server.id] || store.statuses[server.id]?.status === 'Stopping'"
            @click="handleStop(server.id)"
            >{{ i18n.t("home.stop") }}</SLButton
          >
          <SLButton
            variant="ghost"
            size="sm"
            @click="
              store.setCurrentServer(server.id);
              router.push('/console/' + server.id);
            "
          >
            {{ i18n.t("common.console") }}
          </SLButton>
          <SLButton
            variant="ghost"
            size="sm"
            @click="
              store.setCurrentServer(server.id);
              router.push('/config/' + server.id);
            "
          >
            {{ i18n.t("common.config_edit") }}
          </SLButton>
          <SLButton variant="ghost" size="sm" @click="showDeleteConfirmInput(server)">
            {{ i18n.t("home.delete") }}
          </SLButton>
          <!-- 删除确认输入框 -->
          <div
            v-if="deletingServerId === server.id"
            :class="['delete-confirm-input', { closing: isClosing }]"
            @animationend="handleAnimationEnd"
          >
            <p
              class="delete-confirm-message"
              v-html="i18n.t('home.delete_confirm_message', { server: '<strong>' + server.name + '</strong>' })"
            ></p>
            <div class="delete-input-group">
              <input
                type="text"
                v-model="inputServerName"
                class="delete-input"
                :placeholder="i18n.t('home.delete_input_placeholder')"
                @keyup.enter="confirmDelete"
                @keyup.esc="cancelDelete"
                ref="deleteInput"
              />
              <div v-if="deleteError" class="delete-error">{{ deleteError }}</div>
            </div>
            <div class="delete-actions">
              <SLButton variant="ghost" size="sm" @click="cancelDelete">{{
                i18n.t("home.delete_cancel")
              }}</SLButton>
              <SLButton variant="danger" size="sm" @click="confirmDelete">{{
                i18n.t("home.delete_confirm")
              }}</SLButton>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Recent Alerts -->
    <div v-if="recentAlerts.length > 0" class="alerts-section">
      <h3 class="section-title">{{ i18n.t("home.recent_alerts") }}</h3>
      <div class="alerts-list">
        <div
          v-for="(alert, i) in recentAlerts"
          :key="i"
          class="alert-item"
          :class="{
            'alert-error': alert.line.includes('ERROR') || alert.line.includes('FATAL'),
            'alert-warn': alert.line.includes('WARN'),
          }"
        >
          <span class="alert-server">{{ alert.server }}</span>
          <span class="alert-text">{{ alert.line }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.home-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}

.error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: var(--sl-radius-md);
  color: var(--sl-error);
  font-size: 0.875rem;
}
.error-close {
  color: var(--sl-error);
  font-weight: 600;
}

.top-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--sl-space-md);
}

.quick-actions {
  display: flex;
  gap: var(--sl-space-sm);
  margin-top: var(--sl-space-sm);
  flex-wrap: wrap;
}

.card-spacer {
  flex-grow: 1;
}

.quick-start-card .quote-display {
  margin-top: var(--sl-space-md);
  padding-top: var(--sl-space-md);
  border-top: 1px solid var(--sl-border-light);
  text-align: center;
  margin-bottom: 0;
  padding-bottom: 0;
}

.quick-start-card {
  display: flex;
  flex-direction: column;
  height: 280px;
  padding: var(--sl-space-sm);
  background: var(--sl-bg-secondary);
  border: 1px solid var(--sl-border);
  box-shadow: var(--sl-shadow-sm);
  border-radius: var(--sl-radius-lg);
}

.quick-start-card .sl-card__header {
  margin-bottom: var(--sl-space-sm);
}

.quick-start-card .sl-card__title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin-bottom: var(--sl-space-xs);
}

.quick-start-card .sl-card__subtitle {
  font-size: 0.875rem;
  color: var(--sl-text-secondary);
  line-height: 1.4;
}

.gauge-view {
  min-height: 240px;
}

.stats-grid {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-xs) 0;
  min-height: 240px;
}

.stats-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  min-height: 240px;
  color: var(--sl-text-tertiary);
}

.stats-card {
  height: 280px;
  padding: var(--sl-space-sm);
  background: var(--sl-bg-secondary);
  border: 1px solid var(--sl-border);
  box-shadow: var(--sl-shadow-sm);
  border-radius: var(--sl-radius-lg);
  display: flex;
  flex-direction: column;
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.stat-label {
  font-size: 0.8125rem;
  color: var(--sl-text-secondary);
  font-weight: 500;
}
.stat-value {
  font-size: 0.875rem;
  font-weight: 600;
  font-family: var(--sl-font-mono);
}
.stat-detail {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  font-family: var(--sl-font-mono);
  font-weight: 400;
}

.mini-chart {
  width: 100%;
  height: 30px;
}

.mini-chart.taskmgr-style {
  background: var(--sl-bg-secondary);
  border-radius: 4px;
  overflow: hidden;
  width: 100%;
}

.mini-chart.taskmgr-style .chart-svg {
  width: 100%;
  height: 100%;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.section-title {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  font-size: 1.0625rem;
  font-weight: 600;
}

.server-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
  border-radius: var(--sl-radius-full);
  font-size: 0.75rem;
  font-weight: 600;
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-2xl);
  color: var(--sl-text-tertiary);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--sl-space-2xl);
  gap: var(--sl-space-sm);
}

.server-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
  gap: var(--sl-space-lg);
}

.server-card {
  padding: var(--sl-space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
  border-radius: var(--sl-radius-lg);
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
  background: var(--sl-bg-secondary);
  box-shadow: var(--sl-shadow-sm);
}

.server-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--sl-shadow-lg);
  border-color: var(--sl-primary-light);
}

.server-card::before {
  content: "";
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: linear-gradient(90deg, var(--sl-primary), var(--sl-secondary));
  transform: scaleX(0);
  transform-origin: left;
  transition: transform 0.3s ease;
}

.server-card:hover::before {
  transform: scaleX(1);
}

@media (max-width: 768px) {
  .server-grid {
    grid-template-columns: 1fr;
  }
}

.server-card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: var(--sl-space-md);
}

.server-info {
  flex: 1;
  min-width: 0;
}

.server-name-container {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  flex-wrap: wrap;
  margin-bottom: var(--sl-space-xs);
}

.server-name {
  font-size: 1.125rem;
  font-weight: 700;
  color: var(--sl-text-primary);
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-card-header .sl-badge {
  flex-shrink: 0;
}

.edit-server-name {
  opacity: 0;
  background: transparent;
  border: none;
  cursor: pointer;
  font-size: 0.875rem;
  transition: all 0.2s ease;
  padding: 4px;
  border-radius: var(--sl-radius-sm);
  flex-shrink: 0;
}

.server-card:hover .edit-server-name {
  opacity: 1;
}

.edit-server-name:hover {
  background: var(--sl-bg-secondary);
  transform: scale(1.05);
}

.server-meta {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  display: flex;
  flex-wrap: wrap;
  gap: var(--sl-space-xs);
  margin-top: var(--sl-space-xs);
}

.server-meta span {
  background: var(--sl-bg-tertiary);
  padding: 4px 12px;
  border-radius: var(--sl-radius-full);
  white-space: nowrap;
  border: 1px solid var(--sl-border);
  transition: all 0.2s ease;
}

.server-meta span:hover {
  background: var(--sl-bg-secondary);
  border-color: var(--sl-primary-light);
}

/* 就地编辑样式 */
.inline-edit {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.server-name-input {
  flex: 1;
  padding: 4px 8px;
  border: 1px solid var(--sl-primary);
  border-radius: var(--sl-radius-sm);
  background: var(--sl-bg-secondary);
  color: var(--sl-text-primary);
  font-size: 1rem;
  font-weight: 600;
  outline: none;
  transition: all 0.2s ease;
}

.server-name-input:focus {
  box-shadow: 0 0 0 2px var(--sl-primary-bg);
}

.inline-edit-actions {
  display: flex;
  gap: 4px;
  align-items: center;
}

.inline-edit-btn {
  width: 24px;
  height: 24px;
  border-radius: var(--sl-radius-sm);
  border: 1px solid transparent;
  cursor: pointer;
  font-size: 0.875rem;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.inline-edit-btn.save {
  background: var(--sl-primary);
  color: white;
}

.inline-edit-btn.save:hover:not(:disabled) {
  background: var(--sl-primary-dark);
}

.inline-edit-btn.cancel {
  background: var(--sl-bg-secondary);
  color: var(--sl-text-secondary);
  border-color: var(--sl-border);
}

.inline-edit-btn.cancel:hover:not(:disabled) {
  background: var(--sl-bg-tertiary);
}

.inline-edit-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.inline-edit-btn.loading {
  opacity: 0.8;
}

.server-card-path {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-sm);
  font-size: 0.75rem;
  color: var(--sl-text-secondary);
  background: var(--sl-bg-tertiary);
  padding: var(--sl-space-sm) var(--sl-space-md);
  border-radius: var(--sl-radius-md);
  margin: var(--sl-space-xs) 0;
  border: 1px solid var(--sl-border);
  transition: all 0.2s ease;
  cursor: pointer;
  user-select: none;
}

.server-path-text {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.folder-icon {
  flex-shrink: 0;
  opacity: 0.6;
  transition: opacity 0.2s ease;
  color: var(--sl-text-secondary);
}

.server-card-path:hover {
  background: var(--sl-bg-secondary);
  border-top-color: var(--sl-primary-light);
  border-right-color: var(--sl-primary-light);
  border-bottom-color: var(--sl-primary-light);
  color: var(--sl-text-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.server-card-path:hover .folder-icon {
  opacity: 1;
  color: var(--sl-text-primary);
}

.server-card-path:active {
  transform: translateY(1px);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
}

.server-card-actions {
  display: flex;
  gap: var(--sl-space-sm);
  padding-top: var(--sl-space-md);
  border-top: 1px solid var(--sl-border-light);
  flex-wrap: wrap;
  align-items: center;
}

.server-card-actions .sl-button {
  flex: 1;
  min-width: 90px;
  border-radius: var(--sl-radius-md);
  transition: all 0.2s ease;
}

.server-card-actions .sl-button:hover {
  transform: translateY(-1px);
}

.server-card-actions .sl-button:not(.sl-button--variant-primary):not(.sl-button--variant-danger) {
  flex: 0 0 auto;
  min-width: unset;
}

@media (max-width: 480px) {
  .server-card-actions {
    flex-direction: column;
    align-items: stretch;
  }

  .server-card-actions .sl-button {
    flex: 1;
    min-width: unset;
  }
}

/* 删除确认输入框样式 */
.delete-confirm-input {
  width: 100%;
  margin-top: var(--sl-space-sm);
  padding: var(--sl-space-sm);
  background: var(--sl-bg-secondary);
  backdrop-filter: blur(16px) saturate(180%);
  -webkit-backdrop-filter: blur(16px) saturate(180%);
  border-radius: var(--sl-radius-md);
  overflow: hidden;
  animation: deleteInputExpand 0.3s ease forwards;
}

.delete-confirm-input.closing {
  animation: deleteInputCollapse 0.3s ease forwards;
}

@keyframes deleteInputExpand {
  0% {
    opacity: 0;
    transform: translateY(-10px) scaleY(0.2);
    max-height: 0;
    padding: 0 var(--sl-space-sm);
  }
  60% {
    opacity: 1;
    transform: translateY(0) scaleY(1);
    max-height: 200px;
    padding: var(--sl-space-sm);
  }
  100% {
    opacity: 1;
    transform: translateY(0) scaleY(1);
    max-height: 200px;
    padding: var(--sl-space-sm);
  }
}

@keyframes deleteInputCollapse {
  0% {
    opacity: 1;
    transform: translateY(0) scaleY(1);
    max-height: 200px;
    padding: var(--sl-space-sm);
  }
  40% {
    opacity: 0;
    transform: translateY(-5px) scaleY(1);
    max-height: 200px;
    padding: var(--sl-space-sm);
  }
  100% {
    opacity: 0;
    transform: translateY(-10px) scaleY(0.2);
    max-height: 0;
    padding: 0 var(--sl-space-sm);
  }
}

.delete-confirm-message {
  font-size: 0.875rem;
  margin-bottom: var(--sl-space-sm);
  line-height: 1.4;
}

.delete-input-group {
  margin-bottom: var(--sl-space-sm);
}

.delete-input {
  width: 100%;
  padding: var(--sl-space-sm) var(--sl-space-md);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  background: var(--sl-bg-secondary);
  color: var(--sl-text-primary);
  font-size: 0.875rem;
  outline: none;
  transition: all 0.2s ease;
}

.delete-input:focus {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 2px var(--sl-primary-bg);
}

.delete-error {
  margin-top: var(--sl-space-xs);
  font-size: 0.75rem;
  color: var(--sl-error);
}

.delete-actions {
  display: flex;
  gap: var(--sl-space-xs);
  justify-content: flex-end;
  margin-top: var(--sl-space-sm);
}

/* Alerts */
.alerts-section {
  margin-top: var(--sl-space-sm);
}

.alerts-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  max-height: 200px;
  overflow-y: auto;
  background: #1e1e2e;
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-sm);
  margin-top: var(--sl-space-sm);
}

.alert-item {
  display: flex;
  gap: var(--sl-space-sm);
  font-family: var(--sl-font-mono);
  font-size: 0.75rem;
  line-height: 1.6;
  color: #cdd6f4;
}

.alert-error {
  color: #f38ba8;
}
.alert-warn {
  color: #fab387;
}

.alert-server {
  flex-shrink: 0;
  padding: 0 6px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: var(--sl-radius-sm);
  color: #89b4fa;
}

.alert-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.stats-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  margin-bottom: var(--sl-space-sm);
}
.card-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}
.view-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: var(--sl-bg-tertiary);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  color: var(--sl-text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}
.view-toggle:hover {
  background: var(--sl-bg-hover);
  color: var(--sl-text-primary);
  transform: scale(1.05);
}

.gauge-grid {
  display: flex;
  justify-content: space-around;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: 0;
  margin-bottom: 4px;
}
.gauge-item {
  position: relative;
  width: 60px;
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.gauge-svg {
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
}
.gauge-bg {
  stroke: var(--sl-border);
}
.gauge-fill {
  stroke-linecap: round;
  transition: stroke-dasharray 0.3s;
}
.gauge-cpu {
  stroke: var(--sl-primary);
}
.gauge-mem {
  stroke: var(--sl-success);
}
.gauge-disk {
  stroke: #f59e0b;
}
.gauge-text {
  position: absolute;
  display: flex;
  flex-direction: column;
  align-items: center;
  line-height: 1.2;
}
.gauge-value {
  font-size: 0.75rem;
  font-weight: 600;
  font-family: var(--sl-font-mono);
}
.gauge-label {
  font-size: 0.5625rem;
  color: var(--sl-text-tertiary);
}

.gauge-details {
  display: flex;
  justify-content: space-between;
  padding-top: 4px;
  margin-top: 4px;
  border-top: 1px solid var(--sl-border-light);
}
.gauge-detail-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  flex: 1;
}
.detail-label {
  font-size: 0.625rem;
  color: var(--sl-text-tertiary);
}
.detail-value {
  font-size: 0.6875rem;
  font-family: var(--sl-font-mono);
  color: var(--sl-text-secondary);
}

.quote-display {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: var(--sl-space-xs) var(--sl-space-sm);
  margin-top: var(--sl-space-xs);
  border-top: 1px solid var(--sl-border-light);
  cursor: pointer;
  transition: all 0.3s ease;
  border-radius: var(--sl-radius-sm);
  position: relative;
  overflow: hidden;
}
.quote-display:hover {
  opacity: 0.9;
  background: var(--sl-bg-secondary);
  transform: translateY(-1px);
  box-shadow: var(--sl-shadow-sm);
}
.quote-text {
  font-size: 0.875rem;
  color: var(--sl-text-secondary);
  font-style: italic;
  text-align: center;
  transition: all 0.3s ease;
  opacity: 1;
}
.quote-text.fading {
  opacity: 0;
  transform: translateY(5px);
}
.quote-author {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  transition: all 0.3s ease;
  opacity: 1;
}
.quote-author.fading {
  opacity: 0;
  transform: translateY(5px);
}
.quote-loading {
  font-size: 0.875rem;
  color: var(--sl-text-tertiary);
  font-style: italic;
  animation: quoteLoading 1.5s ease-in-out infinite;
}
@keyframes quoteLoading {
  0%,
  100% {
    opacity: 0.6;
  }
  50% {
    opacity: 1;
  }
}

@media (max-width: 900px) {
  .top-row {
    grid-template-columns: 1fr;
  }
}
</style>
