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
import { getStatusVariant, getStatusText } from "../utils/serverStatus";

const router = useRouter();
const store = useServerStore();
const consoleStore = useConsoleStore();

const actionLoading = ref<Record<string, boolean>>({});
const actionError = ref<string | null>(null);

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

onMounted(async () => {
  await store.refreshList();
  for (const s of store.servers) {
    await store.refreshStatus(s.id);
  }

  // 获取真实系统信息
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

  await fetchSystemInfo();
  statsTimer = setInterval(fetchSystemInfo, 2000);

  // Refresh server statuses
  refreshTimer = setInterval(async () => {
    for (const s of store.servers) {
      await store.refreshStatus(s.id);
    }
  }, 3000);
});

onUnmounted(() => {
  if (statsTimer) clearInterval(statsTimer);
  if (refreshTimer) clearInterval(refreshTimer);
});

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
      <SLCard title="快速开始" subtitle="创建或导入你的 Minecraft 服务器" class="quick-start-card">
        <div class="quick-actions">
          <SLButton variant="primary" size="lg" @click="router.push('/create')">
            创建新服务器
          </SLButton>
        </div>
      </SLCard>

      <SLCard class="stats-card">
        <template #header>
          <div class="stats-card-header">
            <span class="card-title">系统资源</span>
            <button
              class="view-toggle"
              @click="statsViewMode = statsViewMode === 'gauge' ? 'detail' : 'gauge'"
              :title="statsViewMode === 'gauge' ? '切换到详细视图' : '切换到仪表盘'"
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
        服务器列表
        <span class="server-count">{{ store.servers.length }}</span>
      </h3>
    </div>

    <div v-if="store.loading" class="loading-state">
      <SLSpinner />
      <span>加载中...</span>
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
      <p class="text-body">还没有服务器</p>
      <p class="text-caption">点击「创建新服务器」开始吧</p>
    </div>

    <div v-else class="server-grid">
      <div v-for="server in store.servers" :key="server.id" class="server-card glass-card">
        <div class="server-card-header">
          <div class="server-info">
            <h4 class="server-name">{{ server.name }}</h4>
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
            >启动</SLButton
          >
          <SLButton
            v-else
            variant="danger"
            size="sm"
            :loading="actionLoading[server.id]"
            @click="handleStop(server.id)"
            >停止</SLButton
          >
          <SLButton
            variant="ghost"
            size="sm"
            @click="
              store.setCurrentServer(server.id);
              router.push('/console/' + server.id);
            "
          >
            控制台
          </SLButton>
          <SLButton
            variant="ghost"
            size="sm"
            @click="
              store.setCurrentServer(server.id);
              router.push('/config/' + server.id);
            "
          >
            配置
          </SLButton>
          <SLButton variant="ghost" size="sm" @click="handleDelete(server.id)"> 删除 </SLButton>
        </div>
      </div>
    </div>

    <!-- Recent Alerts -->
    <div v-if="recentAlerts.length > 0" class="alerts-section">
      <h3 class="section-title">最近警告 / 错误</h3>
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

.server-name {
  font-size: 1rem;
  font-weight: 600;
}
.server-meta {
  margin-top: 2px;
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
  border-radius: 3px;
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
