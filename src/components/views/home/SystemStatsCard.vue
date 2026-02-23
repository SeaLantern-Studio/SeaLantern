<script setup lang="ts">
import { Menu, Gauge } from "lucide-vue-next";
import SLCard from "@components/common/SLCard.vue";
import SLProgress from "@components/common/SLProgress.vue";
import { i18n } from "@language";
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
} from "@utils/statsUtils";
import { formatBytes } from "@utils/serverUtils";

function toggleViewMode() {
  statsViewMode.value = statsViewMode.value === "gauge" ? "detail" : "gauge";
}
</script>

<template>
  <SLCard class="stats-card">
    <template #header>
      <div class="stats-card-header">
        <span class="card-title">{{ i18n.t("home.system_resources") }}</span>
        <button
          class="view-toggle"
          @click="toggleViewMode"
          :title="
            statsViewMode === 'gauge' ? i18n.t('home.detail_view') : i18n.t('home.gauge_view')
          "
        >
          <Menu v-if="statsViewMode === 'gauge'" :size="14" />
          <Gauge v-else :size="14" />
        </button>
      </div>
    </template>
    
    <div v-if="statsLoading" class="stats-loading">
      <div class="spinner"></div>
      <span>{{ i18n.t("common.loading") }}</span>
    </div>
    
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
</template>

<style scoped>
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

.stats-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  min-height: 240px;
  color: var(--sl-text-tertiary);
}

.gauge-view {
  min-height: 240px;
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

.stats-grid {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-xs) 0;
  min-height: 240px;
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
</style>
