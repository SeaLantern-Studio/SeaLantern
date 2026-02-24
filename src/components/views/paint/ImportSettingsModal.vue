<script setup lang="ts">
import SLModal from "@components/common/SLModal.vue";
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";
import { ref, watch } from "vue";

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: "update:visible", value: boolean): void;
  (e: "import", json: string): void;
}>();

const importJson = ref("");

watch(
  () => props.visible,
  (v) => {
    if (!v) {
      importJson.value = "";
    }
  },
);

function handleImport() {
  emit("import", importJson.value);
}

function close() {
  emit("update:visible", false);
}
</script>

<template>
  <SLModal :visible="visible" :title="i18n.t('settings.import_settings')" @close="close">
    <div class="import-form">
      <p class="text-caption">{{ i18n.t("settings.import_desc") }}</p>
      <textarea
        v-model="importJson"
        class="import-textarea"
        placeholder='{"close_servers_on_exit": true, ...}'
        rows="10"
      ></textarea>
    </div>
    <template #footer>
      <SLButton variant="secondary" @click="close">{{ i18n.t("settings.cancel") }}</SLButton>
      <SLButton variant="primary" @click="handleImport">{{
        i18n.t("settings.confirm_import")
      }}</SLButton>
    </template>
  </SLModal>
</template>

<style scoped>
.import-form {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}

.text-caption {
  font-size: 0.8125rem;
  color: var(--sl-text-tertiary);
}

.import-textarea {
  width: 100%;
  margin-top: var(--sl-space-sm);
  padding: var(--sl-space-sm) var(--sl-space-md);
  font-family: var(--sl-font-mono);
  font-size: 0.8125rem;
  color: var(--sl-text-primary);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  resize: vertical;
  line-height: 1.6;
}

.import-textarea:focus {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 3px var(--sl-primary-bg);
  outline: none;
}
</style>
