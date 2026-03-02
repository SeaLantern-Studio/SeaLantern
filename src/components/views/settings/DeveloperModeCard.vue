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
    <div class="sl-settings-group">
      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.developer_mode_toggle") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.developer_mode_toggle_desc") }}</span>
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
