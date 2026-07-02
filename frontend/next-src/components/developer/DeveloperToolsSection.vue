<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import WorkbenchPanel from "@next-src/components/workbench/WorkbenchPanel.vue";
import WorkbenchStatusBanner from "@next-src/components/workbench/WorkbenchStatusBanner.vue";
import { i18n } from "@language";
import type { DeveloperBannerTone } from "@composables/useDeveloperTools";

interface ActiveBanner {
  tone: DeveloperBannerTone;
  message: string;
}

defineProps<{
  updateUrl: string;
  downloadingUpdate: boolean;
  isBrowserMode: boolean;
  activeBanner: ActiveBanner | null;
}>();

const emit = defineEmits<{
  updateUpdateUrl: [value: string];
  downloadUpdate: [];
  showToast: [kind: "success" | "error" | "warning" | "info"];
  showBanner: [tone: DeveloperBannerTone];
  clearBanner: [];
}>();
</script>

<template>
  <WorkbenchPanel :title="i18n.t('developer.next.tools.title')" :description="i18n.t('developer.next.tools.description')">
    <WorkbenchStatusBanner v-if="isBrowserMode" tone="neutral">{{ i18n.t("plugins.ui_shell.desktop_only") }}</WorkbenchStatusBanner>

    <div class="developer-tools-section__group">
      <div class="developer-tools-section__copy">
        <strong>{{ i18n.t("developer.next.tools.toast.title") }}</strong>
        <p>{{ i18n.t("developer.next.tools.toast.description") }}</p>
      </div>
      <div class="developer-tools-section__actions developer-tools-section__actions--wrap">
        <SLButton variant="success" size="sm" @click="emit('showToast', 'success')">{{ i18n.t("developer.next.tools.toast.success_button") }}</SLButton>
        <SLButton variant="danger" size="sm" @click="emit('showToast', 'error')">{{ i18n.t("developer.next.tools.toast.error_button") }}</SLButton>
        <SLButton variant="secondary" size="sm" @click="emit('showToast', 'warning')">{{ i18n.t("developer.next.tools.toast.warning_button") }}</SLButton>
        <SLButton variant="secondary" size="sm" @click="emit('showToast', 'info')">{{ i18n.t("developer.next.tools.toast.info_button") }}</SLButton>
      </div>
    </div>

    <div class="developer-tools-section__group">
      <div class="developer-tools-section__copy">
        <strong>{{ i18n.t("developer.next.tools.banner.title") }}</strong>
        <p>{{ i18n.t("developer.next.tools.banner.description") }}</p>
      </div>
      <WorkbenchStatusBanner v-if="activeBanner" :tone="activeBanner.tone">{{ activeBanner.message }}</WorkbenchStatusBanner>
      <div class="developer-tools-section__actions developer-tools-section__actions--wrap">
        <SLButton variant="secondary" size="sm" @click="emit('showBanner', 'info')">{{ i18n.t("developer.next.tools.banner.info_button") }}</SLButton>
        <SLButton variant="secondary" size="sm" @click="emit('showBanner', 'warning')">{{ i18n.t("developer.next.tools.banner.warning_button") }}</SLButton>
        <SLButton variant="danger" size="sm" @click="emit('showBanner', 'error')">{{ i18n.t("developer.next.tools.banner.error_button") }}</SLButton>
        <SLButton v-if="activeBanner" variant="ghost" size="sm" @click="emit('clearBanner')">{{ i18n.t("developer.next.tools.banner.clear_button") }}</SLButton>
      </div>
    </div>

    <div class="developer-tools-section__group">
      <div class="developer-tools-section__copy">
        <strong>{{ i18n.t("developer.update_test_title") }}</strong>
        <p>{{ i18n.t("developer.update_test_desc") }}</p>
      </div>
      <SLInput :model-value="updateUrl" :placeholder="i18n.t('developer.update_test_placeholder')" @update:model-value="emit('updateUpdateUrl', $event)" />
      <div class="developer-tools-section__actions">
        <SLButton variant="secondary" size="sm" :loading="downloadingUpdate" :disabled="isBrowserMode || !updateUrl.trim()" @click="emit('downloadUpdate')">{{ i18n.t("developer.update_test_button") }}</SLButton>
      </div>
    </div>
  </WorkbenchPanel>
</template>

<style scoped>
.developer-tools-section__group { display: grid; gap: 10px; padding: 14px 16px; border-radius: 18px; border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent); background: color-mix(in srgb, var(--sl-bg-secondary) 68%, transparent); }
.developer-tools-section__copy strong, .developer-tools-section__copy p { margin: 0; }
.developer-tools-section__copy { display: grid; gap: 4px; }
.developer-tools-section__copy p { color: var(--sl-text-secondary); font-size: 0.84rem; line-height: 1.45; }
.developer-tools-section__actions { display: flex; justify-content: flex-start; }
.developer-tools-section__actions--wrap { flex-wrap: wrap; gap: 10px; }
</style>
