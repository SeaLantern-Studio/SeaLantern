<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import { i18n } from "@language";

defineProps<{
  developerMode: boolean;
}>();

const emit = defineEmits<{
  (e: "update:developerMode", value: boolean): void;
  (e: "change"): void;
}>();
</script>

<template>
  <SLCard
    :title="i18n.t('settings.developer_mode')"
    :subtitle="i18n.t('settings.developer_mode_desc')"
  >
    <div class="settings-group">
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ i18n.t("settings.developer_mode_toggle") }}</span>
          <span class="setting-desc">{{ i18n.t("settings.developer_mode_toggle_desc") }}</span>
        </div>
        <SLSwitch
          :model-value="developerMode"
          @update:model-value="
            (v) => {
              emit('update:developerMode', v);
              emit('change');
            }
          "
        />
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
</style>
