<script setup lang="ts">
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
  <cmz-modal :visible="visible" :title="i18n.t('settings.import_title')" @close="close">
    <div class="import-form">
      <p class="text-caption">{{ i18n.t("settings.import_desc") }}</p>
      <cmz-textarea
        v-model="importJson"
        :placeholder="i18n.t('settings.import_placeholder')"
        :rows="10"
      />
    </div>
    <template #footer>
      <cmz-button variant="outline" @click="close">{{ i18n.t("settings.cancel") }}</cmz-button>
      <cmz-button @click="handleImport">{{ i18n.t("settings.import") }}</cmz-button>
    </template>
  </cmz-modal>
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

.import-form :deep(.cmz-textarea) {
  font-family: var(--sl-font-mono);
  font-size: 0.8125rem;
  line-height: 1.6;
}
</style>
