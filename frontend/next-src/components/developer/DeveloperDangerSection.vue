<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import WorkbenchPanel from "@next-src/components/workbench/WorkbenchPanel.vue";
import WorkbenchStatusBanner from "@next-src/components/workbench/WorkbenchStatusBanner.vue";
import { i18n } from "@language";

defineProps<{
  canTriggerCrash: boolean;
  triggeringCrash: boolean;
}>();

const emit = defineEmits<{
  triggerCrash: [];
}>();
</script>

<template>
  <WorkbenchPanel tone="danger" :title="i18n.t('developer.next.other.title')" :description="i18n.t('developer.next.other.description')">
    <WorkbenchStatusBanner tone="error">{{ canTriggerCrash ? i18n.t("developer.next.other.warning") : i18n.t("developer.crash_test_unavailable") }}</WorkbenchStatusBanner>
    <div class="developer-danger-section__group">
      <div class="developer-danger-section__copy">
        <strong>{{ i18n.t("developer.crash_test_title") }}</strong>
        <p>{{ i18n.t("developer.crash_test_desc") }}</p>
      </div>
      <div class="developer-danger-section__actions">
        <SLButton variant="danger" size="sm" :loading="triggeringCrash" :disabled="!canTriggerCrash" @click="emit('triggerCrash')">{{ i18n.t("developer.crash_test_button") }}</SLButton>
      </div>
    </div>
  </WorkbenchPanel>
</template>

<style scoped>
.developer-danger-section__group { display: grid; gap: 10px; padding: 14px 16px; border-radius: 18px; border: 1px solid color-mix(in srgb, var(--sl-error) 24%, var(--sl-border)); background: color-mix(in srgb, var(--sl-error) 6%, var(--sl-bg-secondary)); }
.developer-danger-section__copy { display: grid; gap: 4px; }
.developer-danger-section__copy strong, .developer-danger-section__copy p { margin: 0; }
.developer-danger-section__copy p { color: var(--sl-text-secondary); font-size: 0.84rem; line-height: 1.45; }
.developer-danger-section__actions { display: flex; justify-content: flex-start; }
</style>
