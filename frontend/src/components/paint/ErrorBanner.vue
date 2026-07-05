<script setup lang="ts">
import { i18n } from "@language";

defineProps<{
  message: string | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();
</script>

<template>
  <div v-if="message" class="msg-banner error-banner">
    <span>{{ message }}</span>
    <button
      :aria-label="i18n.t('common.close')"
      :title="i18n.t('common.close')"
      @click="emit('close')"
    >
      ×
    </button>
  </div>
</template>

<style scoped>
.msg-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-radius: var(--sl-radius-md);
  font-size: 0.875rem;
}

.error-banner {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: var(--sl-error);
}

.msg-banner button {
  font-weight: 600;
  color: inherit;
}
</style>
