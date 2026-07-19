<script setup lang="ts">
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
  <cmz-card
    :title="i18n.t('settings.developer_mode')"
    :subtitle="i18n.t('settings.developer_mode_desc')"
  >
    <div class="sl-settings-group">
      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.developer_mode_toggle") }}</span>
          <span class="settings-entry-desc">{{
            i18n.t("settings.developer_mode_toggle_desc")
          }}</span>
        </div>
        <cmz-switch
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
  </cmz-card>
</template>
