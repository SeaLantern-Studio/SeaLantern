<script setup lang="ts">
import { useTabIndicator } from "@composables/useTabIndicator";
import { i18n } from "@language";
import { watch } from "vue";

type PlayerTab = "online" | "whitelist" | "banned" | "ops";

const props = defineProps<{
  modelValue: PlayerTab;
  onlineCount: number;
  whitelistCount: number;
  bannedCount: number;
  opsCount: number;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: PlayerTab): void;
}>();

const activeTab = {
  get: () => props.modelValue,
  set: (value: PlayerTab) => emit("update:modelValue", value),
};

const { indicatorRef: tabIndicator, updatePosition: updateTabIndicator } =
  useTabIndicator(activeTab);

const localeRef = i18n.getLocaleRef();
watch(localeRef, () => {
  updateTabIndicator();
});
</script>

<template>
  <div class="player-tab-bar">
    <div class="tab-indicator" ref="tabIndicator"></div>
    <button
      class="tab-btn"
      :class="{ active: modelValue === 'online' }"
      @click="activeTab = 'online'"
    >
      {{ i18n.t("players.online_players") }}
      <span class="tab-count">{{ onlineCount }}</span>
    </button>
    <button
      class="tab-btn"
      :class="{ active: modelValue === 'whitelist' }"
      @click="activeTab = 'whitelist'"
    >
      {{ i18n.t("players.whitelist") }} <span class="tab-count">{{ whitelistCount }}</span>
    </button>
    <button
      class="tab-btn"
      :class="{ active: modelValue === 'banned' }"
      @click="activeTab = 'banned'"
    >
      {{ i18n.t("players.banned") }} <span class="tab-count">{{ bannedCount }}</span>
    </button>
    <button class="tab-btn" :class="{ active: modelValue === 'ops' }" @click="activeTab = 'ops'">
      {{ i18n.t("players.ops") }} <span class="tab-count">{{ opsCount }}</span>
    </button>
  </div>
</template>

<style scoped>
.player-tab-bar {
  display: flex;
  gap: var(--sl-space-xs);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-xs);
  width: fit-content;
  margin: var(--sl-space-md) 0;
  position: relative;
  overflow: hidden;
}

.tab-indicator {
  position: absolute;
  top: var(--sl-space-xs);
  bottom: var(--sl-space-xs);
  background: var(--sl-primary-bg);
  border-radius: var(--sl-radius-sm);
  transition: all 0.3s ease;
  box-shadow: var(--sl-shadow-sm);
  z-index: 1;
  border: 1px solid var(--sl-primary);
  opacity: 0.9;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: 6px 14px;
  border-radius: var(--sl-radius-sm);
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-text-secondary);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all var(--sl-transition-fast);
  position: relative;
  z-index: 2;
  white-space: nowrap;
}

.tab-btn:hover {
  color: var(--sl-text-primary);
}

.tab-btn.active {
  color: var(--sl-primary);
}

.tab-count {
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-full);
  font-size: 0.6875rem;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.tab-btn.active .tab-count {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
}
</style>
