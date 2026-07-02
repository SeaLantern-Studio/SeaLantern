<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import WorkbenchPanel from "@next-src/components/workbench/WorkbenchPanel.vue";
import { useGlobalMessage } from "@composables/useMessage";
import { i18n } from "@language";
import { useSettingsStore } from "@stores/settingsStore";
import { normalizeAppError } from "@utils/appError";

const settingsStore = useSettingsStore();
const globalMessage = useGlobalMessage();

const pending = reactive({
  developerMode: false,
});

const developerMode = computed(() => settingsStore.settings.developer_mode);

watch(
  () => settingsStore.settings.developer_mode,
  () => {
    if (!pending.developerMode) {
      return;
    }

    pending.developerMode = false;
  },
);

async function updateDeveloperMode(value: boolean): Promise<void> {
  if (pending.developerMode) {
    return;
  }

  const previous = settingsStore.settings.developer_mode;
  settingsStore.updateSettings({ developer_mode: value });
  pending.developerMode = true;

  try {
    await settingsStore.updatePartial({ developer_mode: value });
  } catch (error) {
    settingsStore.updateSettings({ developer_mode: previous });
    const normalized = normalizeAppError(error);
    globalMessage.error(normalized.message || i18n.t("common.message_unknown_error"));
  } finally {
    pending.developerMode = false;
  }
}
</script>

<template>
  <WorkbenchPanel :title="i18n.t('settings.next.developer_management.title')" :description="i18n.t('settings.next.developer_management.description')">
    <div class="developer-management-section__row">
      <div class="developer-management-section__copy">
        <strong>{{ i18n.t("settings.developer_mode_toggle") }}</strong>
        <p>{{ i18n.t("settings.developer_mode_toggle_desc") }}</p>
      </div>

      <SLSwitch :model-value="developerMode" :disabled="pending.developerMode" @update:model-value="updateDeveloperMode" />
    </div>
  </WorkbenchPanel>
</template>

<style scoped>
.developer-management-section__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  padding: 14px 16px;
  border-radius: 18px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 68%, transparent);
}

.developer-management-section__copy {
  min-width: 0;
  display: grid;
  gap: 4px;
}

.developer-management-section__copy strong,
.developer-management-section__copy p {
  margin: 0;
}

.developer-management-section__copy p {
  color: var(--sl-text-secondary);
  font-size: 0.84rem;
  line-height: 1.45;
}

@media (max-width: 720px) {
  .developer-management-section__row {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
