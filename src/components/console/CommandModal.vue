<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import SLModal from "@components/common/SLModal.vue";
import { i18n } from "@language";
import type { ServerCommand } from "@type/server";

interface Props {
  visible: boolean;
  title: string;
  editingCommand: ServerCommand | null;
  commandName: string;
  commandText: string;
  loading: boolean;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save"): void;
  (e: "delete", cmd: ServerCommand): void;
  (e: "updateName", value: string): void;
  (e: "updateText", value: string): void;
}>();
</script>

<template>
  <SLModal :visible="visible" :title="title" :close-on-overlay="false" @close="emit('close')">
    <div class="command-modal-content">
      <div class="form-group">
        <label for="command-name">{{ i18n.t("console.command_name") }}</label>
        <SLInput
          id="command-name"
          :value="commandName"
          @input="emit('updateName', $event.target.value)"
          :placeholder="i18n.t('console.enter_command_name')"
          :disabled="loading"
        />
      </div>
      <div class="form-group">
        <label for="command-text">{{ i18n.t("console.command_content") }}</label>
        <SLInput
          id="command-text"
          :value="commandText"
          @input="emit('updateText', $event.target.value)"
          :placeholder="i18n.t('console.enter_command_content')"
          :disabled="loading"
        />
      </div>
    </div>
    <template #footer>
      <div class="modal-footer">
        <SLButton variant="secondary" @click="emit('close')" :disabled="loading">
          {{ i18n.t("console.cancel") }}
        </SLButton>
        <SLButton
          v-if="editingCommand"
          variant="danger"
          @click="emit('delete', editingCommand)"
          :disabled="loading"
        >
          {{ i18n.t("console.delete") }}
        </SLButton>
        <SLButton
          variant="primary"
          @click="emit('save')"
          :disabled="loading || !commandName || !commandText"
        >
          {{ i18n.t("console.save") }}
        </SLButton>
      </div>
    </template>
  </SLModal>
</template>

<style scoped>
.command-modal-content {
  padding: var(--sl-space-md);
}

.form-group {
  margin-bottom: var(--sl-space-md);
}

.form-group label {
  display: block;
  margin-bottom: var(--sl-space-xs);
  font-weight: 500;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--sl-space-sm);
  padding-top: var(--sl-space-md);
  border-top: 1px solid var(--sl-border-light);
}
</style>
