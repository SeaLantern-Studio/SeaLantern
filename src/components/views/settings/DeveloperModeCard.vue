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
      <cmz-form-field
        :label="i18n.t('settings.developer_mode_toggle')"
        :hint="i18n.t('settings.developer_mode_toggle_desc')"
        label-position="left"
      >
        <cmz-switch
          :model-value="developerMode"
          @update:model-value="
            (v) => {
              emit('update:developerMode', v);
              emit('change');
            }
          "
        />
      </cmz-form-field>
    </div>
  </cmz-card>
</template>
