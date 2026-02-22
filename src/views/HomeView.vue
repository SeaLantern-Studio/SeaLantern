<script setup lang="ts">
import { Menu, Server, Pencil, FolderOpen, Check, X, Gauge } from "lucide-vue-next";
import { onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import SLCard from "@components/common/SLCard.vue";
import SLButton from "@components/common/SLButton.vue";
import SLBadge from "@components/common/SLBadge.vue";
import SLProgress from "@components/common/SLProgress.vue";
import { useServerStore } from "@stores/serverStore";
import { systemApi } from "@api/system";
import { i18n } from "@language";

// 导入拆分后的模块
import {
  currentQuote,
  displayText,
  isTyping,
  initQuote,
  updateQuote,
  startQuoteTimer,
  cleanupQuoteResources,
} from "@utils/quoteUtils";

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
} from "@utils/statsUtils";

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
} from "@utils/serverUtils";

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

<style src="@styles/views/HomeView.css" scoped></style>
