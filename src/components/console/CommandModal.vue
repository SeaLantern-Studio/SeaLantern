<script setup lang="ts">
import { i18n } from "@language";
import type { ServerCommand } from "@type/server";
import { computed } from "vue";

interface Props {
  visible: boolean;
  title: string;
  editingCommand: ServerCommand | null;
  commandName: string;
  commandText: string;
  loading: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save"): void;
  (e: "delete", cmd: ServerCommand): void;
  (e: "updateName", value: string): void;
  (e: "updateText", value: string): void;
}>();

const commandNameModel = computed({
  get: () => props.commandName,
  set: (value: string) => emit("updateName", value),
});

const commandTextModel = computed({
  get: () => props.commandText,
  set: (value: string) => emit("updateText", value),
});
</script>

<template>
  <cmz-modal :visible="visible" :title="title" :close-on-overlay="false" @close="emit('close')">
    <div class="command-modal-content">
      <cmz-input
        :label="i18n.t('console.command_name')"
        v-model="commandNameModel"
        :placeholder="i18n.t('console.enter_command_name')"
        :disabled="loading"
      />
      <cmz-input
        :label="i18n.t('console.command_content')"
        v-model="commandTextModel"
        :placeholder="i18n.t('console.enter_command_content')"
        :disabled="loading"
      />
    </div>
    <template #footer>
      <cmz-button variant="outline" @click="emit('close')" :disabled="loading">
        {{ i18n.t("console.cancel") }}
      </cmz-button>
      <cmz-button
        v-if="editingCommand"
        variant="solid"
        color="#ef4444"
        @click="emit('delete', editingCommand)"
        :disabled="loading"
      >
        {{ i18n.t("console.delete") }}
      </cmz-button>
      <cmz-button
        @click="emit('save')"
        :disabled="loading || !commandName || !commandText"
      >
        {{ i18n.t("console.save") }}
      </cmz-button>
    </template>
  </cmz-modal>
</template>

<style scoped>
.command-modal-content {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
  padding: var(--sl-space-md);
}
</style>
