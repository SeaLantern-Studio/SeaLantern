<script setup lang="ts">
import { i18n } from "@language";
import { AlertCircle, Search } from "lucide-vue-next";

defineProps<{
  loading: boolean;
  error: string | null;
  marketErrorHint: string;
  hasPlugins: boolean;
}>();

const emit = defineEmits<{
  (e: "retry"): void;
}>();
</script>

<template>
  <div v-if="loading" class="market-loading">
    <div class="loading-spinner"></div>
    <span class="loading-text">{{ i18n.t("market.loading") }}</span>
  </div>

  <div v-else-if="error" class="market-error">
    <AlertCircle :size="48" :stroke-width="1.5" />
    <p class="error-title">{{ i18n.t("market.error_title") }}</p>
    <p class="error-detail">{{ error }}</p>
    <p v-if="marketErrorHint" class="error-hint">{{ marketErrorHint }}</p>
    <button class="retry-btn" @click="emit('retry')">
      {{ i18n.t("market.retry") }}
    </button>
  </div>

  <div v-else-if="!hasPlugins" class="market-empty">
    <Search :size="48" :stroke-width="1.5" />
    <p>{{ i18n.t("market.no_plugins") }}</p>
  </div>
</template>

<style scoped>
.market-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-2xl);
  text-align: center;
  color: var(--sl-text-tertiary);
}

.market-error,
.market-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-2xl);
  text-align: center;
  color: var(--sl-text-tertiary);
}

.error-hint {
  margin-top: 8px;
  color: var(--sl-text-secondary);
  max-width: 640px;
  line-height: 1.5;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--sl-border);
  border-top-color: var(--sl-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.loading-text {
  margin-top: 16px;
}

.error-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--sl-text-primary);
  margin: 16px 0 8px;
}

.error-detail {
  font-size: 14px;
  color: var(--sl-text-tertiary);
  margin: 0 0 16px;
}

.retry-btn {
  padding: 8px 24px;
  border-radius: var(--sl-radius-md);
  border: none;
  background: var(--sl-primary);
  color: white;
  cursor: pointer;
}
</style>
