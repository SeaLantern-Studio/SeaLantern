<script setup lang="ts">
import { Menu, Server, Pencil, FolderOpen, Check, X, Gauge } from "lucide-vue-next";
import { onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import SLCard from "../components/common/SLCard.vue";
import SLButton from "../components/common/SLButton.vue";
import SLBadge from "../components/common/SLBadge.vue";
import SLProgress from "../components/common/SLProgress.vue";
import { useServerStore } from "../stores/serverStore";
import { systemApi } from "../api/system";
import { i18n } from "../language";

// 导入拆分后的模块
import {
  currentQuote,
  displayText,
  isTyping,
  initQuote,
  updateQuote,
  startQuoteTimer,
  cleanupQuoteResources,
} from "../utils/quoteUtils";

import {
  systemInfo,
  cpuUsage,
  memUsage,
  diskUsage,
  statsViewMode,
  statsLoading,
  cpuGaugeOption,
  memGaugeOption,
  diskGaugeOption,
  cpuLineOption,
  memLineOption,
  fetchSystemInfo,
  startThemeObserver,
  cleanupStatsResources,
} from "../utils/statsUtils";

import {
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
  recentAlerts,
  formatBytes,
  formatServerPath,
  getStatusVariant,
  getStatusText,
  handleStart,
  handleStop,
  startEditServerName,
  saveServerName,
  cancelEdit,
  showDeleteConfirmInput,
  confirmDelete,
  cancelDelete,
  handleAnimationEnd,
  handleClickOutside,
  closeDeleteConfirm,
} from "../utils/serverUtils";

const router = useRouter();
const store = useServerStore();

/**
 * 处理服务器路径点击事件
 * @param path 服务器路径
 */
async function handlePathClick(path: string) {
  try {
    await systemApi.openFolder(path);
  } catch (e) {
    console.error("打开文件夹失败:", e);
  }
}

let statsTimer: ReturnType<typeof setInterval> | null = null;
let refreshTimer: ReturnType<typeof setInterval> | null = null;

onMounted(() => {
  // 初始化名言
  initQuote();

  // 异步加载服务器列表，不阻塞页面渲染
  const loadServers = async () => {
    try {
      await store.refreshList();
      await Promise.all(store.servers.map((s) => store.refreshStatus(s.id)));
    } catch (e) {
      console.error("Failed to load servers:", e);
    }
  };

  // 启动异步加载
  loadServers();
  fetchSystemInfo();

  // 设置定时任务
  statsTimer = setInterval(fetchSystemInfo, 3000);

  // 名言每30秒更新一次
  startQuoteTimer();

  // Refresh server statuses
  refreshTimer = setInterval(async () => {
    await Promise.all(store.servers.map((s) => store.refreshStatus(s.id)));
  }, 3000);

  // 添加全局点击事件监听器，点击空白区域收回删除确认输入框
  document.addEventListener("click", handleClickOutside);

  // 监听主题和无障碍模式变化
  startThemeObserver();
});

onUnmounted(() => {
  if (statsTimer) clearInterval(statsTimer);
  if (refreshTimer) clearInterval(refreshTimer);
  // 清理引用相关资源
  cleanupQuoteResources();
  // 清理系统状态相关资源
  cleanupStatsResources();
  // 移除全局点击事件监听器
  document.removeEventListener("click", handleClickOutside);
});
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
          <span v-if="!displayText && !isTyping" class="quote-loading">{{
            i18n.t("common.loading")
          }}</span>
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
              <Gauge v-else :size="14" />
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
              <v-chart
                class="gauge-chart"
                :option="cpuGaugeOption"
                autoresize
                :update-options="{ notMerge: false }"
              />
            </div>
            <div class="gauge-item">
              <v-chart
                class="gauge-chart"
                :option="memGaugeOption"
                autoresize
                :update-options="{ notMerge: false }"
              />
            </div>
            <div class="gauge-item">
              <v-chart
                class="gauge-chart"
                :option="diskGaugeOption"
                autoresize
                :update-options="{ notMerge: false }"
              />
            </div>
          </div>
          <div v-if="systemInfo" class="gauge-details">
            <div class="gauge-detail-item">
              <span class="detail-label">{{ i18n.t("home.cpu") }}</span
              ><span class="detail-value"
                >{{ systemInfo.cpu.count }} {{ i18n.t("home.core") }}</span
              >
            </div>
            <div class="gauge-detail-item">
              <span class="detail-label">{{ i18n.t("home.memory") }}</span
              ><span class="detail-value"
                >{{ formatBytes(systemInfo.memory.used) }} /
                {{ formatBytes(systemInfo.memory.total) }}</span
              >
            </div>
            <div class="gauge-detail-item">
              <span class="detail-label">{{ i18n.t("home.disk") }}</span
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
                >{{ i18n.t("home.cpu")
                }}<span v-if="systemInfo" class="stat-detail">
                  · {{ systemInfo.cpu.count }} {{ i18n.t("home.core") }}</span
                ></span
              >
              <span class="stat-value">{{ cpuUsage }}%</span>
            </div>
            <div class="mini-chart">
              <v-chart class="line-chart" :option="cpuLineOption" autoresize />
            </div>
          </div>
          <div class="stat-item">
            <div class="stat-header">
              <span class="stat-label"
                >{{ i18n.t("home.memory")
                }}<span v-if="systemInfo" class="stat-detail">
                  · {{ formatBytes(systemInfo.memory.used) }} /
                  {{ formatBytes(systemInfo.memory.total) }}</span
                ></span
              >
              <span class="stat-value">{{ memUsage }}%</span>
            </div>
            <div class="mini-chart">
              <v-chart class="line-chart" :option="memLineOption" autoresize />
            </div>
          </div>
          <div class="stat-item">
            <div class="stat-header">
              <span class="stat-label"
                >{{ i18n.t("home.disk") }}
                <span v-if="systemInfo" class="stat-detail">
                  · {{ formatBytes(systemInfo.disk.used) }} /
                  {{ formatBytes(systemInfo.disk.total) }}
                </span>
              </span>
              <span class="stat-value">{{ diskUsage }}%</span>
            </div>
            <SLProgress :value="diskUsage" variant="warning" :showPercent="false" />
          </div>
        </div>
      </SLCard>
    </div>

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
      <Server :size="64" :stroke-width="1" class="empty-icon" />
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
                      <Check :size="16" />
                    </button>
                    <button
                      class="inline-edit-btn cancel"
                      @click="cancelEdit"
                      :disabled="editLoading"
                    >
                      <X :size="16" />
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
              <span>{{ i18n.t("home.port") }} {{ server.port }}</span>
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
          :title="server.path"
          @click="handlePathClick(server.path)"
        >
          <span class="server-path-text">{{ formatServerPath(server.jar_path) }}</span>
          <FolderOpen class="folder-icon" :size="16" />
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
        </div>

        <div
          v-if="deletingServerId === server.id"
          :class="['delete-confirm-area', { closing: isClosing }]"
          @animationend="handleAnimationEnd"
        >
          <p
            class="delete-confirm-message"
            v-html="
              i18n.t('home.delete_confirm_message', {
                server: '<strong>' + server.name + '</strong>',
              })
            "
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

    <SLConfirmDialog
      :visible="showDeleteConfirm"
      :title="i18n.t('home.delete_server')"
      :message="
        i18n.t('home.delete_confirm_message', {
          server: '<strong>' + deleteServerName + '</strong>',
        })
      "
      :confirmText="i18n.t('home.delete_confirm')"
      :cancelText="i18n.t('home.delete_cancel')"
      confirmVariant="danger"
      :requireInput="true"
      :inputPlaceholder="i18n.t('home.delete_input_placeholder')"
      :expectedInput="deleteServerName"
      @confirm="confirmDelete"
      @cancel="cancelDelete"
      @close="closeDeleteConfirm"
      dangerous
    />
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
  background: var(--sl-bg-secondary);
  border-radius: 4px;
  overflow: hidden;
}

.line-chart {
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
  border-color: var(--sl-primary-light);
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

.delete-confirm-area {
  margin-top: var(--sl-space-md);
  padding-top: var(--sl-space-md);
  border-top: 1px solid var(--sl-border);
  animation: slideDown 0.3s ease forwards;
}

.delete-confirm-area.closing {
  animation: slideUp 0.2s ease forwards;
}

@keyframes slideDown {
  from {
    opacity: 0;
    max-height: 0;
    padding-top: 0;
    margin-top: 0;
  }
  to {
    opacity: 1;
    max-height: 200px;
    padding-top: var(--sl-space-md);
    margin-top: var(--sl-space-md);
  }
}

@keyframes slideUp {
  from {
    opacity: 1;
    max-height: 200px;
    padding-top: var(--sl-space-md);
    margin-top: var(--sl-space-md);
  }
  to {
    opacity: 0;
    max-height: 0;
    padding-top: 0;
    margin-top: 0;
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
  border-radius: var(--sl-radius-md);
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-secondary);
  font-size: 0.75rem;
  outline: none;
  transition: all 0.2s ease;
}

.delete-input:focus {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 2px var(--sl-primary-bg);
  background: var(--sl-bg-secondary);
  color: var(--sl-text-primary);
}

.delete-input:hover {
  background: var(--sl-bg-secondary);
  border-color: var(--sl-primary-light);
  color: var(--sl-text-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.delete-input:active {
  transform: translateY(1px);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
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
  min-height: 70px;
}
.gauge-item {
  position: relative;
  width: 70px;
  height: 70px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.gauge-chart {
  width: 100%;
  height: 100%;
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
