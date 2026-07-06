<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";

interface Props {
  errorMessage?: string | null;
}

withDefaults(defineProps<Props>(), {
  errorMessage: null,
});

const emit = defineEmits<{
  create: [];
  importExisting: [];
}>();
</script>

<template>
  <section class="servers-empty-state">
    <div class="servers-empty-state__badge">{{ i18n.t("servers.next.empty.eyebrow") }}</div>
    <h2 class="servers-empty-state__title">{{ i18n.t("servers.next.empty.title") }}</h2>
    <p class="servers-empty-state__description">{{ i18n.t("servers.next.empty.description") }}</p>
    <p v-if="errorMessage" class="servers-empty-state__error">{{ errorMessage }}</p>

    <div class="servers-empty-state__actions">
      <SLButton variant="primary" @click="emit('create')">{{
        i18n.t("servers.next.empty.create")
      }}</SLButton>
      <SLButton variant="secondary" @click="emit('importExisting')">{{
        i18n.t("servers.next.empty.import")
      }}</SLButton>
    </div>
  </section>
</template>

<style scoped>
.servers-empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 48px 20px;
  border: 1px dashed color-mix(in srgb, var(--sl-border) 88%, transparent);
  border-radius: 24px;
  background: color-mix(in srgb, var(--sl-surface) 94%, transparent);
  text-align: center;
}

.servers-empty-state__badge {
  padding: 6px 12px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--sl-primary) 12%, transparent);
  color: var(--sl-primary);
  font-size: 0.875rem;
  font-weight: 600;
}

.servers-empty-state__title {
  margin: 0;
  color: var(--sl-text-primary);
  font-size: 1.5rem;
}

.servers-empty-state__description,
.servers-empty-state__error {
  max-width: 560px;
  margin: 0;
  color: var(--sl-text-secondary);
  line-height: 1.6;
}

.servers-empty-state__error {
  color: var(--sl-error);
}

.servers-empty-state__actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 12px;
}
</style>
