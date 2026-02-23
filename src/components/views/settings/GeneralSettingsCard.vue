<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLSelect from "@components/common/SLSelect.vue";
import { i18n } from "@language";

const props = defineProps<{
  closeServersOnExit: boolean;
  autoAcceptEula: boolean;
  closeAction: "ask" | "minimize" | "close";
}>();

type CloseAction = "ask" | "minimize" | "close";

const emit = defineEmits<{
  (e: "update:closeServersOnExit", value: boolean): void;
  (e: "update:autoAcceptEula", value: boolean): void;
  (e: "update:closeAction", value: CloseAction): void;
  (e: "change"): void;
}>();

function handleCloseActionChange(v: string | number) {
  emit("update:closeAction", v as CloseAction);
  emit("change");
}

const closeActionOptions = [
  { label: i18n.t("settings.close_action_ask"), value: "ask" },
  { label: i18n.t("settings.close_action_minimize"), value: "minimize" },
  { label: i18n.t("settings.close_action_close"), value: "close" },
];
</script>

<template>
  <SLCard :title="i18n.t('settings.general')" :subtitle="i18n.t('settings.general_desc')">
    <div class="settings-group">
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ i18n.t("settings.auto_stop") }}</span>
          <span class="setting-desc">{{ i18n.t("settings.auto_stop_desc") }}</span>
        </div>
        <SLSwitch
          :model-value="closeServersOnExit"
          @update:model-value="
            (v) => {
              emit('update:closeServersOnExit', v);
              emit('change');
            }
          "
        />
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ i18n.t("settings.auto_eula") }}</span>
          <span class="setting-desc">{{ i18n.t("settings.auto_eula_desc") }}</span>
        </div>
        <SLSwitch
          :model-value="autoAcceptEula"
          @update:model-value="
            (v) => {
              emit('update:autoAcceptEula', v);
              emit('change');
            }
          "
        />
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ i18n.t("settings.close_action") }}</span>
          <span class="setting-desc">{{ i18n.t("settings.close_action_desc") }}</span>
        </div>
        <div class="input-md">
          <SLSelect
            :model-value="closeAction"
            :options="closeActionOptions"
            @update:model-value="handleCloseActionChange"
          />
        </div>
      </div>
    </div>
  </SLCard>
</template>

<style scoped>
.settings-group {
  display: flex;
  flex-direction: column;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sl-space-md) 0;
  border-bottom: 1px solid var(--sl-border-light);
  gap: var(--sl-space-lg);
}

.setting-row:last-child {
  border-bottom: none;
}

.setting-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.setting-label {
  font-size: 0.9375rem;
  font-weight: 500;
  color: var(--sl-text-primary);
}

.setting-desc {
  font-size: 0.8125rem;
  color: var(--sl-text-tertiary);
  line-height: 1.4;
}

.input-md {
  width: 200px;
  flex-shrink: 0;
}
</style>
