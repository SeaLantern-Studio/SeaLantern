<script setup lang="ts">
import { ref, watch } from "vue";
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import SLModal from "@components/common/SLModal.vue";
import { i18n } from "@language";

const props = defineProps<{
  visible: boolean;
  recommendedPath: string;
  busy?: boolean;
  error?: string | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "browse"): void;
  (e: "confirm", value: string): void;
}>();

const draftPath = ref("");

watch(
  () => [props.visible, props.recommendedPath] as const,
  ([visible, recommendedPath]) => {
    if (visible) {
      draftPath.value = recommendedPath || "";
    }
  },
  { immediate: true },
);
</script>

<template>
  <SLModal
    :visible="visible"
    :title="i18n.t('settings.data_dir_setup_title')"
    :close-on-overlay="false"
    :show-close-button="false"
    width="560px"
    @close=""
  >
    <div class="setup-body">
      <p class="setup-copy">{{ i18n.t("settings.data_dir_setup_desc") }}</p>

      <div v-if="error" class="setup-error">{{ error }}</div>

      <SLInput v-model="draftPath" :placeholder="recommendedPath">
        <template #suffix>
          <button type="button" class="sl-input-action" @click="emit('browse')">
            {{ i18n.t("settings.browse") }}
          </button>
        </template>
      </SLInput>

      <p class="setup-hint">{{ i18n.t("settings.data_dir_setup_hint") }}</p>
    </div>

    <template #footer>
      <div class="setup-actions">
        <SLButton variant="secondary" :disabled="busy" @click="emit('close')">
          {{ i18n.t("common.cancel") }}
        </SLButton>
        <SLButton variant="primary" :loading="busy" @click="emit('confirm', draftPath)">
          {{ i18n.t("settings.data_dir_setup_confirm") }}
        </SLButton>
      </div>
    </template>
  </SLModal>
</template>

<style scoped>
.setup-body {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}

.setup-copy,
.setup-hint {
  margin: 0;
  color: var(--sl-text-secondary);
  line-height: 1.6;
}

.setup-error {
  padding: 10px 12px;
  border-radius: var(--sl-radius-md);
  border: 1px solid var(--sl-error);
  background: var(--sl-error-bg);
  color: var(--sl-error);
  font-size: var(--sl-font-size-sm);
}

.setup-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--sl-space-sm);
}
</style>
