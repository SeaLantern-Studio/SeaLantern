<script setup lang="ts">
defineProps<{
  message: string | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();
</script>

<template>
  <div v-if="message" class="error-banner">
    <span>{{ message }}</span>
    <button class="error-close" @click="emit('close')">x</button>
  </div>
</template>

<style scoped>
.error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: var(--sl-radius-md);
  color: var(--sl-error);
  font-size: 0.875rem;
}

.error-close {
  color: var(--sl-error);
  font-weight: 600;
}
</style>
