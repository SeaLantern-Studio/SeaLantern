<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import SLCheckbox from "@components/common/SLCheckbox.vue";
import SLMenu from "@components/common/SLMenu.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import PluginPermissionPanel from "@components/plugin/PluginPermissionPanel.vue";
import { i18n } from "@language";
import type { PluginInfo, PluginState, PluginUpdateInfo } from "@type/plugin";
import { GitBranch, Layers, MoreVertical, Settings, ShieldAlert } from "@lucide/vue";
import type { PluginMenuItem } from "./pluginListShared";

defineProps<{
  plugin: PluginInfo;
  batchMode: boolean;
  selected: boolean;
  updateInfo?: PluginUpdateInfo;
  iconUrl?: string;
  safeMode: boolean;
  getPluginName: (plugin: PluginInfo) => string;
  getPluginDescription: (plugin: PluginInfo) => string;
  isPluginEnabled: (state: PluginState) => boolean;
  getStatusColor: (state: PluginState) => string;
  getStatusLabel: (state: PluginState) => string;
  hasSettings: (plugin: PluginInfo) => boolean;
  hasMissingRequiredDependencies: (plugin: PluginInfo) => boolean;
  hasMissingOptionalDependencies: (plugin: PluginInfo) => boolean;
  getDependencyTooltip: (plugin: PluginInfo) => string;
  menuItems: PluginMenuItem[];
}>();

const emit = defineEmits<{
  (e: "toggle-selection", pluginId: string): void;
  (e: "show-missing-dependencies", plugin: PluginInfo): void;
  (e: "menu-select", item: { id: string | number }, pluginId: string): void;
  (e: "open-repository", url: string): void;
  (e: "open-settings", plugin: PluginInfo): void;
  (e: "toggle-plugin", pluginId: string, nextEnabled: boolean): void;
}>();
</script>

<template>
  <SLCard class="plugin-card" :class="{ 'plugin-card--selected': batchMode && selected }">
    <div class="plugin-content">
      <label v-if="batchMode" class="plugin-checkbox" @click.stop>
        <SLCheckbox
          :modelValue="selected"
          @update:modelValue="emit('toggle-selection', plugin.manifest.id)"
        />
      </label>

      <div class="plugin-card-actions">
        <div v-if="updateInfo" class="update-badge" :title="i18n.t('plugins.update_available')">
          <ShieldAlert :size="12" />
        </div>

        <div
          v-if="hasMissingRequiredDependencies(plugin)"
          class="dependency-indicator dependency-indicator--required"
          :title="getDependencyTooltip(plugin)"
          @click.stop="emit('show-missing-dependencies', plugin)"
        ></div>
        <div
          v-else-if="hasMissingOptionalDependencies(plugin)"
          class="dependency-indicator dependency-indicator--optional"
          :title="getDependencyTooltip(plugin)"
          @click.stop="emit('show-missing-dependencies', plugin)"
        ></div>

        <PluginPermissionPanel
          :plugin-id="plugin.manifest.id"
          :permissions="plugin.manifest.permissions || []"
        />

        <SLMenu
          :items="menuItems"
          position="bottom-end"
          @select="emit('menu-select', $event, plugin.manifest.id)"
        >
          <SLButton variant="ghost" icon-only size="sm">
            <MoreVertical :size="16" />
          </SLButton>
        </SLMenu>
      </div>

      <div class="plugin-main">
        <div class="plugin-icon">
          <img
            v-if="iconUrl"
            :src="iconUrl"
            :alt="i18n.t('plugins.icon_alt')"
            class="plugin-icon-img"
          />
          <Layers v-else :size="32" :stroke-width="1.5" class="plugin-icon-default" />
        </div>
        <div class="plugin-info">
          <div class="plugin-header">
            <div class="plugin-title-row">
              <h3 class="plugin-name">{{ getPluginName(plugin) }}</h3>
              <span class="plugin-version">v{{ plugin.manifest.version }}</span>
            </div>
            <div class="plugin-author-row">
              <span v-if="plugin.manifest.author" class="plugin-author">
                {{ i18n.t("common.by_author", { author: plugin.manifest.author.name }) }}
              </span>
              <SLButton
                v-if="plugin.manifest.repository"
                variant="ghost"
                icon-only
                size="sm"
                :title="i18n.t('plugins.open_repository')"
                @click.stop="emit('open-repository', plugin.manifest.repository)"
              >
                <GitBranch :size="14" />
              </SLButton>
            </div>
          </div>
          <p v-if="plugin.manifest.description" class="plugin-description">
            {{ getPluginDescription(plugin) }}
          </p>
          <p
            v-if="typeof plugin.state === 'object' && 'error' in plugin.state"
            class="plugin-error-message"
          >
            {{ plugin.state.error }}
          </p>
        </div>
      </div>

      <div class="plugin-footer">
        <span
          class="plugin-status"
          :style="{ color: getStatusColor(plugin.state) }"
          :title="
            typeof plugin.state === 'object' && 'error' in plugin.state
              ? plugin.state.error
              : undefined
          "
        >
          {{ getStatusLabel(plugin.state) }}
        </span>
        <div class="plugin-actions">
          <SLButton
            v-if="hasSettings(plugin)"
            variant="ghost"
            icon-only
            size="sm"
            :title="i18n.t('plugins.settings')"
            @click="emit('open-settings', plugin)"
          >
            <Settings :size="16" />
          </SLButton>
          <SLSwitch
            v-if="!safeMode"
            :modelValue="isPluginEnabled(plugin.state)"
            :disabled="hasMissingRequiredDependencies(plugin) && !isPluginEnabled(plugin.state)"
            :title="
              hasMissingRequiredDependencies(plugin) && !isPluginEnabled(plugin.state)
                ? i18n.t('plugins.missing_required_deps')
                : ''
            "
            size="sm"
            @update:modelValue="emit('toggle-plugin', plugin.manifest.id, Boolean($event))"
          />
          <span v-else class="safe-mode-label">{{ i18n.t("plugins.safe_mode_disabled") }}</span>
        </div>
      </div>
    </div>
  </SLCard>
</template>

<style scoped>
.safe-mode-label {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  background-color: var(--sl-surface);
  padding: 0.25rem 0.5rem;
  border-radius: var(--sl-radius-sm);
  border: 1px solid var(--sl-border-light);
  align-self: center;
}

.plugin-card {
  position: relative;
  overflow: hidden;
}

.plugin-card--selected {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 1px rgba(96, 165, 250, 0.35);
}

.plugin-content {
  padding: 8px;
  position: relative;
}

.plugin-checkbox {
  position: absolute;
  top: 12px;
  left: 12px;
  z-index: 2;
}

.plugin-card-actions {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  align-items: center;
  gap: 6px;
  z-index: 2;
}

.update-badge {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  color: var(--sl-warning);
}

.dependency-indicator {
  width: 10px;
  height: 10px;
  border-radius: 999px;
  cursor: pointer;
}

.dependency-indicator--required {
  background: var(--sl-error);
}

.dependency-indicator--optional {
  background: var(--sl-warning);
}

.plugin-main {
  display: flex;
  gap: 12px;
  margin-top: 18px;
}

.plugin-icon {
  width: 56px;
  height: 56px;
  border-radius: var(--sl-radius-md);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--sl-bg-secondary);
  flex-shrink: 0;
}

.plugin-icon-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.plugin-icon-default {
  color: var(--sl-text-tertiary);
}

.plugin-info {
  min-width: 0;
  flex: 1;
}

.plugin-header {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.plugin-title-row,
.plugin-author-row,
.plugin-footer,
.plugin-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.plugin-title-row,
.plugin-footer {
  justify-content: space-between;
}

.plugin-name {
  margin: 0;
  font-size: 16px;
  color: var(--sl-text-primary);
}

.plugin-version,
.plugin-author {
  font-size: 12px;
  color: var(--sl-text-tertiary);
}

.plugin-description,
.plugin-error-message {
  margin: 10px 0 0;
  font-size: 13px;
  line-height: 1.5;
}

.plugin-description {
  color: var(--sl-text-secondary);
}

.plugin-error-message {
  color: var(--sl-error);
}

.plugin-footer {
  margin-top: 14px;
}

.plugin-status {
  font-size: 12px;
}
</style>
