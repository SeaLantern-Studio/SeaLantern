<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import { i18n } from "@language";

defineProps<{
  loading: boolean;
  description?: string;
}>();

defineEmits<{
  (e: "refresh"): void;
  (e: "open-market"): void;
}>();
</script>

<template>
  <SLCard class="plugins-empty-state" variant="outline">
    <div class="plugins-empty-state__content">
      <span class="plugins-empty-state__eyebrow">{{ i18n.t("plugins.next.empty.eyebrow") }}</span>
      <h2 class="plugins-empty-state__title">{{ i18n.t("plugins.next.empty.title") }}</h2>
      <p class="plugins-empty-state__description">
        {{ description || i18n.t("plugins.next.empty.description") }}
      </p>

      <div class="plugins-empty-state__actions">
        <SLButton variant="primary" size="sm" @click="$emit('open-market')">
          {{ i18n.t("plugins.next.empty.open_market") }}
        </SLButton>
        <SLButton variant="secondary" size="sm" :loading="loading" @click="$emit('refresh')">
          {{ i18n.t("plugins.next.empty.refresh") }}
        </SLButton>
      </div>
    </div>
  </SLCard>
</template>

<style scoped>
.plugins-empty-state__content {
  padding: 8px;
  display: grid;
  gap: 12px;
}

.plugins-empty-state__eyebrow {
  font-size: 0.78rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--sl-text-tertiary);
}

.plugins-empty-state__title {
  margin: 0;
  color: var(--sl-text-primary);
}

.plugins-empty-state__description {
  margin: 0;
  color: var(--sl-text-secondary);
  line-height: 1.7;
}

.plugins-empty-state__actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}
</style>
