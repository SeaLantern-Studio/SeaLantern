<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import type { MarketPlugin } from "@components/views/plugins/pluginMarketShared";
import { i18n } from "@language";
import { Puzzle } from "lucide-vue-next";

defineProps<{
  plugins: MarketPlugin[];
  installingPluginId: string | null;
  getIconUrl: (plugin: MarketPlugin) => string | null;
  resolveI18n: (value: Record<string, string> | string | undefined) => string;
  getCategoryLabel: (key: string) => string;
  isInstalled: (pluginId: string) => boolean;
  isInstalledAndEnabled: (pluginId: string) => boolean;
  getInstallButtonText: (pluginId: string) => string;
}>();

const emit = defineEmits<{
  (e: "show-detail", plugin: MarketPlugin): void;
  (e: "install", plugin: MarketPlugin): void;
}>();
</script>

<template>
  <div class="market-grid">
    <SLCard
      v-for="plugin in plugins"
      :key="plugin.id"
      class="market-card"
      @click="emit('show-detail', plugin)"
    >
      <div class="card-icon">
        <img v-if="getIconUrl(plugin)" :src="getIconUrl(plugin)!" :alt="resolveI18n(plugin.name)" />
        <Puzzle v-else :size="32" :stroke-width="1.5" />
      </div>
      <div class="card-info">
        <div class="card-header">
          <span class="card-name">{{ resolveI18n(plugin.name) }}</span>
          <span class="card-version">{{ plugin.version ? "v" + plugin.version : "" }}</span>
        </div>
        <span class="card-author">by {{ plugin.author?.name || "Unknown" }}</span>
        <p class="card-desc">{{ resolveI18n(plugin.description) }}</p>

        <div v-if="plugin.dependencies?.length" class="card-deps">
          <span class="deps-label">{{ i18n.t("market.requires") }}</span>
          <span class="deps-list">{{ plugin.dependencies.join(", ") }}</span>
        </div>

        <div class="card-footer">
          <div class="card-tags">
            <span v-for="tag in plugin.categories?.slice(0, 2)" :key="tag" class="card-tag">
              {{ getCategoryLabel(tag) }}
            </span>
          </div>
          <button
            :class="[
              'install-btn',
              {
                installed: isInstalled(plugin.id),
                'is-enabled': isInstalledAndEnabled(plugin.id),
              },
            ]"
            :disabled="isInstalled(plugin.id) || installingPluginId === plugin.id"
            :title="isInstalledAndEnabled(plugin.id) ? i18n.t('market.plugin_running_warning') : ''"
            @click.stop="emit('install', plugin)"
          >
            {{ getInstallButtonText(plugin.id) }}
          </button>
        </div>
      </div>
    </SLCard>
  </div>
</template>

<style scoped>
.market-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--sl-space-md);
}

@media (max-width: 1200px) {
  .market-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .market-grid {
    grid-template-columns: 1fr;
  }
}

.market-card {
  cursor: pointer;
  transition: all var(--sl-transition-fast);
  display: flex;
  gap: var(--sl-space-lg);
  box-sizing: border-box;
  height: 100%;
}

.market-card:hover {
  border-color: var(--sl-border);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.card-icon {
  flex-shrink: 0;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--sl-text-tertiary);
}

.card-icon img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  border-radius: var(--sl-radius-md);
}

.card-info {
  flex: 1;
  min-width: 0;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.card-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--sl-text-primary);
  word-wrap: break-word;
}

.card-version {
  padding: 2px 6px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-xs);
  font-size: 11px;
  color: var(--sl-text-tertiary);
}

.card-author {
  font-size: 12px;
  color: var(--sl-text-secondary);
  margin-bottom: 10px;
}

.card-desc {
  margin: 0 0 12px;
  font-size: 13px;
  color: var(--sl-text-secondary);
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-wrap: break-word;
}

.card-deps {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 6px 0;
  font-size: 12px;
}

.deps-label {
  color: var(--sl-warning);
  font-weight: 500;
}

.deps-list {
  color: var(--sl-text-secondary);
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 14px;
  padding-top: 8px;
  border-top: 1px solid var(--sl-border);
}

.card-tags {
  display: flex;
  gap: 6px;
}

.card-tag {
  padding: 2px 8px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-xs);
  font-size: 11px;
  color: var(--sl-text-tertiary);
}

.install-btn {
  padding: 6px 16px;
  border-radius: var(--sl-radius-sm);
  border: none;
  background: var(--sl-primary);
  color: white;
  font-size: 13px;
  cursor: pointer;
  transition: opacity 0.2s;
}

.install-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.install-btn:disabled {
  cursor: not-allowed;
}

.install-btn.installed {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-secondary);
}

.install-btn.is-enabled {
  background: var(--sl-bg-tertiary);
  color: var(--sl-warning);
  font-size: 12px;
}
</style>
