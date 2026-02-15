<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { useRouter } from "vue-router";
import SLCard from "../components/common/SLCard.vue";
import SLButton from "../components/common/SLButton.vue";
import SLBadge from "../components/common/SLBadge.vue";
import SLProgress from "../components/common/SLProgress.vue";
import SLSpinner from "../components/common/SLSpinner.vue";
import { useServerStore } from "../stores/serverStore";
import { useConsoleStore } from "../stores/consoleStore";
import { serverApi } from "../api/server";
import { systemApi, type SystemInfo } from "../api/system";
import { i18n } from "../locales";
import { getStatusVariant, getStatusText } from "../utils/serverStatus";

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
const statsViewMode = ref<"detail" | "gauge">("gauge"); // 视图模式
let statsTimer: ReturnType<typeof setInterval> | null = null;
let refreshTimer: ReturnType<typeof setInterval> | null = null;

// 格式化字节
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
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
      cpuUsage.value = Math.round(info.cpu.usage);
      memUsage.value = Math.round(info.memory.usage);
      diskUsage.value = Math.round(info.disk.usage);
      cpuHistory.value.push(cpuUsage.value);
      memHistory.value.push(memUsage.value);
      if (cpuHistory.value.length > 30) cpuHistory.value.shift();
      if (memHistory.value.length > 30) memHistory.value.shift();
    } catch (e) {
      console.error("Failed to fetch system info:", e);
    }
  };

  // 启动异步加载
  loadServers();
  fetchSystemInfo();

  // 设置定时任务
  statsTimer = setInterval(fetchSystemInfo, 2000);

  // Refresh server statuses
  refreshTimer = setInterval(async () => {
    for (const s of store.servers) {
      await store.refreshStatus(s.id);
    }
  }, 3000);

  // 添加全局点击事件监听器，点击空白区域收回删除确认输入框
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  if (statsTimer) clearInterval(statsTimer);
  if (refreshTimer) clearInterval(refreshTimer);
  // 移除全局点击事件监听器
  document.removeEventListener('click', handleClickOutside);
});

// 处理点击空白区域的逻辑
function handleClickOutside(event: MouseEvent) {
  if (!deletingServerId.value) return;
  
  const target = event.target as HTMLElement;
  // 检查是否点击了删除确认输入框或删除按钮
  const isDeleteConfirmInput = target.closest('.delete-confirm-input');
  const isDeleteButton = target.closest('.server-card-actions')?.querySelector('button');
  
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

async function handleDelete(id: string) {
  try {
    await serverApi.deleteServer(id);
    await store.refreshList();
  } catch (e) {
    actionError.value = String(e);
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
    deleteError.value = "服务器名称输入错误，请重新输入";
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
  if (event.animationName === 'deleteInputCollapse') {
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
              <svg
                v-if="statsViewMode === 'gauge'"
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <path d="M4 6h16M4 12h16M4 18h16" />
              </svg>
              <svg
                v-else
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <circle cx="12" cy="12" r="10" />
                <path d="M12 6v6l4 2" />
              </svg>
            </button>
          </div>
        </template>
        <!-- 仪表盘视图 -->
        <div v-if="statsViewMode === 'gauge'" class="gauge-view">
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
            <SLProgress :value="cpuUsage" variant="primary" :showPercent="false" />
            <div class="mini-chart">
              <svg viewBox="0 0 120 20" class="chart-svg">
                <polyline
                  :points="cpuHistory.map((v, i) => i * 4 + ',' + (20 - v * 0.2)).join(' ')"
                  fill="none"
                  stroke="var(--sl-primary)"
                  stroke-width="1.5"
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
            <SLProgress :value="memUsage" variant="success" :showPercent="false" />
            <div class="mini-chart">
              <svg viewBox="0 0 120 20" class="chart-svg">
                <polyline
                  :points="memHistory.map((v, i) => i * 4 + ',' + (20 - v * 0.2)).join(' ')"
                  fill="none"
                  stroke="var(--sl-success)"
                  stroke-width="1.5"
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
      <svg
        width="64"
        height="64"
        viewBox="0 0 24 24"
        fill="none"
        stroke="var(--sl-text-tertiary)"
        stroke-width="1"
        stroke-linecap="round"
      >
        <path d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
      </svg>
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
                      ✓
                    </button>
                    <button
                      class="inline-edit-btn cancel"
                      @click="cancelEdit"
                      :disabled="editLoading"
                    >
                      ✕
                    </button>
                  </div>
                </div>
              </template>
              <template v-else>
                <h4 class="server-name">{{ server.name }}</h4>
                <button
                  class="edit-server-name"
                  @click="startEditServerName(server)"
                  title="编辑服务器名称"
                >
                  ✏️
                </button>
              </template>
            </div>
            <span class="server-meta text-caption">
              {{ server.core_type }} | 端口 {{ server.port }} | {{ server.max_memory }}MB
            </span>
          </div>
          <SLBadge
            :text="getStatusText(store.statuses[server.id]?.status)"
            :variant="getStatusVariant(store.statuses[server.id]?.status)"
          />
        </div>

        <div class="server-card-path text-mono text-caption" :title="server.jar_path">
          {{ server.jar_path }}
        </div>

        <div class="server-card-actions">
          <SLButton
            v-if="store.statuses[server.id]?.status !== 'Running'"
            variant="primary"
            size="sm"
            :loading="actionLoading[server.id]"
            @click="handleStart(server.id)"
            >{{ i18n.t("home.start") }}</SLButton
          >
          <SLButton
            v-else
            variant="danger"
            size="sm"
            :loading="actionLoading[server.id]"
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
          <SLButton variant="ghost" size="sm" @click="systemApi.openFolder(server.path)">
            打开文件夹
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
            <p class="delete-confirm-message">
              请输入服务器名称 <strong>{{ server.name }}</strong> 以确认删除
            </p>
            <div class="delete-input-group">
              <input
                type="text"
                v-model="inputServerName"
                class="delete-input"
                placeholder="输入服务器名称"
                @keyup.enter="confirmDelete"
                @keyup.esc="cancelDelete"
                ref="deleteInput"
              >
              <div v-if="deleteError" class="delete-error">{{ deleteError }}</div>
            </div>
            <div class="delete-actions">
              <SLButton variant="ghost" size="sm" @click="cancelDelete">取消</SLButton>
              <SLButton variant="danger" size="sm" @click="confirmDelete">确认删除</SLButton>
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
  gap: var(--sl-space-md);
  margin-top: var(--sl-space-sm);
}

.stats-grid {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
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
  height: 20px;
}

.chart-svg {
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
  grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
  gap: var(--sl-space-md);
}

.server-card {
  padding: var(--sl-space-md);
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}

.server-card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}

.server-name-container {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.server-name {
  font-size: 1rem;
  font-weight: 600;
}

.edit-server-name {
  opacity: 0;
  background: transparent;
  border: none;
  cursor: pointer;
  font-size: 0.875rem;
  transition: opacity 0.2s ease;
  padding: 2px;
  border-radius: var(--sl-radius-sm);
}

.server-card:hover .edit-server-name {
  opacity: 1;
}

.edit-server-name:hover {
  background: var(--sl-bg-secondary);
}

.server-meta {
  margin-top: 2px;
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
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
}

.server-card-actions {
  display: flex;
  gap: var(--sl-space-xs);
  padding-top: var(--sl-space-sm);
  border-top: 1px solid var(--sl-border-light);
  flex-wrap: wrap;
}

/* 删除确认输入框样式 */
.delete-confirm-input {
  width: 100%;
  margin-top: var(--sl-space-sm);
  padding: var(--sl-space-sm);
  background: rgba(26, 29, 40, 0.8);
  backdrop-filter: blur(16px) saturate(180%);
  -webkit-backdrop-filter: blur(16px) saturate(180%);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: var(--sl-radius-md);
  overflow: hidden;
  animation: deleteInputExpand 0.3s ease forwards;
}

[data-theme="light"] .delete-confirm-input {
  background: rgba(255, 255, 255, 0.72);
  border: 1px solid rgba(255, 255, 255, 0.5);
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
}
.card-title {
  font-size: 1rem;
  font-weight: 600;
}
.view-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: transparent;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  color: var(--sl-text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}
.view-toggle:hover {
  background: var(--sl-bg-hover);
  color: var(--sl-text-primary);
}

.gauge-grid {
  display: flex;
  justify-content: space-around;
  align-items: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-xs) 0;
}
.gauge-item {
  position: relative;
  width: 70px;
  height: 70px;
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
  font-size: 0.875rem;
  font-weight: 600;
  font-family: var(--sl-font-mono);
}
.gauge-label {
  font-size: 0.625rem;
  color: var(--sl-text-tertiary);
}

.gauge-details {
  display: flex;
  justify-content: space-between;
  padding-top: var(--sl-space-sm);
  margin-top: var(--sl-space-sm);
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
  font-size: 0.6875rem;
  color: var(--sl-text-tertiary);
}
.detail-value {
  font-size: 0.75rem;
  font-family: var(--sl-font-mono);
  color: var(--sl-text-secondary);
}

@media (max-width: 900px) {
  .top-row {
    grid-template-columns: 1fr;
  }
}


</style>
