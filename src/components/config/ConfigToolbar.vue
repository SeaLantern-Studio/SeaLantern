<script setup lang="ts">
import { i18n } from "../../locales";
import SLButton from "../common/SLButton.vue";
import SLInput from "../common/SLInput.vue";
import SLSpinner from "../common/SLSpinner.vue";

interface Props {
  serverPath: string;
  searchQuery: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "updateSearch", value: string): void;
}>();
</script>

<template>
  <div class="config-toolbar">
    <div class="toolbar-left">
      <SLInput
        :modelValue="searchQuery"
        :placeholder="i18n.t('config.search')"
        @input="emit('updateSearch', $event.target.value)"
        style="width: 200px"
      />
    </div>
    <div class="toolbar-right"></div>
  </div>
</template>

<style scoped>
.config-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--sl-space-sm) var(--sl-space-md);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  margin-bottom: var(--sl-space-md);
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}
</style>
