<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import SLCard from "@components/common/SLCard.vue";
import SLInput from "@components/common/SLInput.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import { i18n } from "@language";

type StartupMode = "jar" | "bat" | "sh";

const props = defineProps<{
  serverName: string;
  startupMode: StartupMode;
  maxMemory: string;
  minMemory: string;
  port: string;
  onlineMode: boolean;
}>();

const emit = defineEmits<{
  (e: "update:serverName", value: string): void;
  (e: "update:startupMode", value: StartupMode): void;
  (e: "update:maxMemory", value: string): void;
  (e: "update:minMemory", value: string): void;
  (e: "update:port", value: string): void;
  (e: "update:onlineMode", value: boolean): void;
}>();

const startupModes: StartupMode[] = ["jar", "bat", "sh"];

const indicatorRef = ref<HTMLElement | null>(null);

function handleStartupModeChange(mode: StartupMode) {
  if (props.startupMode === mode) {
    return;
  }
  emit("update:startupMode", mode);
}

function handleNumberInput(e: Event, type: "maxMemory" | "minMemory" | "port") {
  const target = e.target as HTMLInputElement;
  const value = target.value;
  if (value === "" || /^\d+$/.test(value)) {
    emit(`update:${type}`, value);
  }
}

function updateIndicator() {
  requestAnimationFrame(() => {
    if (indicatorRef.value) {
      const tabs = indicatorRef.value.parentElement;
      if (!tabs) return;
      const activeTab = tabs.querySelector(".startup-mode-tab.active") as HTMLElement;
      if (activeTab) {
        indicatorRef.value.style.left = `${activeTab.offsetLeft}px`;
        indicatorRef.value.style.width = `${activeTab.offsetWidth}px`;
      }
    }
  });
}

const localeRef = i18n.getLocaleRef();
watch(localeRef, updateIndicator);

watch(() => props.startupMode, updateIndicator);

onMounted(updateIndicator);
</script>

<template>
  <SLCard :title="i18n.t('create.title')">
    <div class="form-grid">
      <div class="server-name-row">
        <SLInput
          :label="i18n.t('create.server_name')"
          :placeholder="i18n.t('create.server_name')"
          :model-value="serverName"
          @update:model-value="$emit('update:serverName', $event)"
        />
      </div>
      <div class="startup-mode-row">
        <span class="startup-mode-label">{{ i18n.t("create.startup_mode") }}</span>
        <div class="startup-mode-control">
          <div class="startup-mode-tabs">
            <div class="startup-mode-indicator" ref="indicatorRef"></div>
            <button
              v-for="mode in startupModes"
              :key="mode"
              type="button"
              class="startup-mode-tab"
              :class="{ active: startupMode === mode }"
              @click="handleStartupModeChange(mode)"
            >
              {{ mode === "jar" ? "JAR" : mode }}
            </button>
          </div>
        </div>
      </div>

      <SLInput
        :label="i18n.t('create.max_memory')"
        type="text"
        :model-value="maxMemory"
        @input="handleNumberInput($event, 'maxMemory')"
      />
      <SLInput
        :label="i18n.t('create.min_memory')"
        type="text"
        :model-value="minMemory"
        @input="handleNumberInput($event, 'minMemory')"
      />
      <SLInput
        :label="i18n.t('settings.default_port')"
        type="text"
        :model-value="port"
        :placeholder="i18n.t('create.default_port_placeholder')"
        @input="handleNumberInput($event, 'port')"
      />
      <div class="online-mode-cell">
        <span class="online-mode-label">{{ i18n.t("create.online_mode") }}</span>
        <div class="online-mode-box">
          <span class="online-mode-text">{{
            onlineMode ? i18n.t("create.online_mode_on") : i18n.t("create.online_mode_off")
          }}</span>
          <SLSwitch
            :model-value="onlineMode"
            @update:model-value="$emit('update:onlineMode', $event)"
          />
        </div>
      </div>
    </div>
  </SLCard>
</template>

<style scoped>
.form-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--sl-space-md);
}
.server-name-row {
  grid-column: 1 / -1;
}
.startup-mode-row {
  grid-column: 1 / -1;
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
}
.startup-mode-label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-text-secondary);
}
.startup-mode-control {
  display: flex;
  align-items: center;
}
.startup-mode-tabs {
  display: flex;
  gap: 2px;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  padding: 3px;
  width: 100%;
  position: relative;
  overflow: hidden;
}
.startup-mode-indicator {
  position: absolute;
  top: 3px;
  bottom: 3px;
  background: var(--sl-primary-bg);
  border-radius: var(--sl-radius-sm);
  transition: all var(--sl-transition-normal);
  box-shadow: var(--sl-shadow-sm);
  z-index: 1;
  border: 1px solid var(--sl-primary);
  opacity: 0.9;
}
.startup-mode-tab {
  flex: 1;
  padding: 6px 14px;
  border-radius: var(--sl-radius-sm);
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-text-secondary);
  transition: all var(--sl-transition-fast);
  position: relative;
  z-index: 2;
  background: transparent;
  border: none;
  cursor: pointer;
  text-align: center;
}
.startup-mode-tab:hover {
  color: var(--sl-text-primary);
}
.startup-mode-tab.active {
  color: var(--sl-primary);
}

@media (prefers-color-scheme: dark) {
  .startup-mode-tab {
    color: var(--sl-text-tertiary);
  }
  .startup-mode-tab:hover {
    color: var(--sl-text-primary);
  }
  .startup-mode-tab.active {
    color: var(--sl-primary);
  }
}
.online-mode-cell {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
}
.online-mode-label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-text-secondary);
}
.online-mode-box {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-md);
  padding: 6px 12px;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  height: 36px;
  box-sizing: border-box;
}
.online-mode-text {
  font-size: 0.875rem;
  color: var(--sl-text-tertiary);
}
</style>
