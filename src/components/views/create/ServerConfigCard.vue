<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import SLInput from "@components/common/SLInput.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import { i18n } from "@language";

const props = defineProps<{
  serverName: string;
  maxMemory: string;
  minMemory: string;
  port: string;
  onlineMode: boolean;
}>();

const emit = defineEmits<{
  (e: "update:serverName", value: string): void;
  (e: "update:maxMemory", value: string): void;
  (e: "update:minMemory", value: string): void;
  (e: "update:port", value: string): void;
  (e: "update:onlineMode", value: boolean): void;
}>();

function handleNumberInput(e: Event, type: "maxMemory" | "minMemory" | "port") {
  const target = e.target as HTMLInputElement;
  const value = target.value;
  if (value === "" || /^\d+$/.test(value)) {
    emit(`update:${type}`, value);
  }
}
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
          <SLSwitch :model-value="onlineMode" @update:model-value="$emit('update:onlineMode', $event)" />
        </div>
      </div>
    </div>
    <div class="file-hint">
      <span class="file-hint-icon">i</span>
      <span class="file-hint-text">{{ i18n.t("create.file_hint") }}</span>
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
.file-hint {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  margin-top: var(--sl-space-md);
  padding: var(--sl-space-sm) var(--sl-space-md);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
}
.file-hint-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  background: var(--sl-primary);
  color: white;
  font-size: 0.75rem;
  font-weight: 600;
  border-radius: 50%;
  flex-shrink: 0;
}
.file-hint-text {
  font-size: 0.8125rem;
  color: var(--sl-text-secondary);
  line-height: 1.4;
}
</style>
