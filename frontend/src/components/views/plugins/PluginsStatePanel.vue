<script setup lang="ts">
import { i18n } from "@language";
import { Layers } from "@lucide/vue";

defineProps<{
  errorMessage: string | null;
  loading: boolean;
  hasPlugins: boolean;
}>();
</script>

<template>
  <template v-if="errorMessage || (loading && !hasPlugins) || (!loading && !hasPlugins)">
    <div v-if="errorMessage" class="error-banner">
      <span class="error-icon">!</span>
      <span class="error-text">{{ errorMessage }}</span>
    </div>

    <div v-if="loading && !hasPlugins" class="loading-state">
      <div class="loading-spinner"></div>
      <span class="loading-text">{{ i18n.t("plugins.loading_plugins") }}</span>
    </div>

    <div v-else-if="!loading && !hasPlugins" class="empty-state">
      <div class="empty-icon">
        <Layers :size="48" :stroke-width="1.5" />
      </div>
      <h3 class="empty-title">{{ i18n.t("plugins.no_plugins") }}</h3>
      <p class="empty-desc">{{ i18n.t("plugins.no_plugins_desc") }}</p>
    </div>
  </template>
</template>

<style scoped>
.error-banner {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: var(--sl-radius-md);
  background: var(--sl-error-bg);
  border: 1px solid var(--sl-error);
}

.error-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--sl-error);
  color: var(--sl-text-inverse);
  font-size: 12px;
  font-weight: 700;
}

.error-text {
  color: var(--sl-error);
  font-size: 14px;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 24px;
  border-radius: var(--sl-radius-md);
  text-align: center;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
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
  color: var(--sl-text-secondary);
  font-size: 14px;
}

.empty-icon {
  color: var(--sl-text-tertiary);
  margin-bottom: 16px;
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 8px;
}

.empty-desc {
  font-size: 14px;
  color: var(--sl-text-secondary);
  margin: 0;
  max-width: 320px;
}
</style>
