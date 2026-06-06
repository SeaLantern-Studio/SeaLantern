<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import SLInput from "@components/common/SLInput.vue";
import { i18n } from "@language";

defineProps<{
  updateUrl: string;
  downloadingUpdate: boolean;
  triggeringCrash: boolean;
  canTriggerCrash: boolean;
  hasSystemInfo: boolean;
  isBrowserMode: boolean;
}>();

const emit = defineEmits<{
  (e: "update:updateUrl", value: string): void;
  (e: "copy-system"): void;
  (e: "download-update"): void;
  (e: "trigger-crash"): void;
}>();
</script>

<template>
  <SLCard :title="i18n.t('developer.actions_title')" :subtitle="i18n.t('developer.actions_desc')">
    <div class="developer-actions-grid">
      <section class="developer-action-block">
        <h4 class="developer-action-title">{{ i18n.t("developer.system_tools_title") }}</h4>
        <p class="developer-action-text">{{ i18n.t("developer.system_tools_desc") }}</p>
        <SLButton
          variant="secondary"
          size="sm"
          :disabled="!hasSystemInfo"
          @click="emit('copy-system')"
        >
          {{ i18n.t("developer.copy_system") }}
        </SLButton>
      </section>

      <section class="developer-action-block">
        <h4 class="developer-action-title">{{ i18n.t("developer.update_test_title") }}</h4>
        <p class="developer-action-text">{{ i18n.t("developer.update_test_desc") }}</p>
        <SLInput
          :model-value="updateUrl"
          :placeholder="i18n.t('developer.update_test_placeholder')"
          @update:model-value="emit('update:updateUrl', $event)"
        />
        <SLButton
          variant="secondary"
          size="sm"
          :loading="downloadingUpdate"
          :disabled="isBrowserMode || !updateUrl.trim()"
          @click="emit('download-update')"
        >
          {{ i18n.t("developer.update_test_button") }}
        </SLButton>
      </section>

      <section class="developer-action-block">
        <h4 class="developer-action-title">{{ i18n.t("developer.crash_test_title") }}</h4>
        <p class="developer-action-text">
          {{
            canTriggerCrash
              ? i18n.t("developer.crash_test_desc")
              : i18n.t("developer.crash_test_unavailable")
          }}
        </p>
        <SLButton
          variant="danger"
          size="sm"
          :loading="triggeringCrash"
          :disabled="!canTriggerCrash"
          @click="emit('trigger-crash')"
        >
          {{ i18n.t("developer.crash_test_button") }}
        </SLButton>
      </section>
    </div>
  </SLCard>
</template>

<style scoped>
.developer-actions-grid {
  display: grid;
  gap: var(--sl-space-md);
}

.developer-action-block {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  border-radius: var(--sl-radius-md);
  border: 1px solid var(--sl-border);
  background: var(--sl-bg-secondary);
}

.developer-action-title {
  margin: 0;
  font-size: var(--sl-font-size-base);
  color: var(--sl-text-primary);
}

.developer-action-text {
  margin: 0;
  color: var(--sl-text-secondary);
  line-height: 1.6;
}
</style>
