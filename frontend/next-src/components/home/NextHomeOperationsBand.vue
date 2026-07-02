<script setup lang="ts">
import SLButton from "@src/components/common/SLButton.vue";
import { i18n } from "@language";

defineProps<{
  refreshLabel: string;
  createLabel: string;
  importLabel: string;
  lastUpdatedLabel: string;
  isRefreshing: boolean;
  cardId?: string;
  section?: string;
  editable?: boolean;
}>();

const emit = defineEmits<{
  create: [];
  import: [];
  refresh: [];
}>();
</script>

<template>
  <section class="next-home-operations-band">
    <article
      class="next-home-operations-band__actions surface-card"
      :data-card-id="cardId"
      :data-layout-section="section"
      :data-layout-editable="editable ? 'true' : 'false'"
    >
      <div class="next-home-operations-band__copy">
        <h2>{{ i18n.t("shell.home_operations_title") }}</h2>
      </div>

      <div class="next-home-operations-band__action-row">
        <SLButton size="lg" @click="emit('create')">{{ createLabel }}</SLButton>
        <SLButton variant="secondary" size="lg" @click="emit('import')">{{ importLabel }}</SLButton>
        <SLButton variant="ghost" size="lg" :loading="isRefreshing" @click="emit('refresh')">{{
          refreshLabel
        }}</SLButton>
      </div>

      <div class="next-home-operations-band__footer">
        <span>{{ i18n.t("shell.home_operations_last_updated") }}</span>
        <strong>{{ lastUpdatedLabel }}</strong>
      </div>
    </article>
  </section>
</template>

<style scoped>
.next-home-operations-band {
  min-width: 0;
}

.surface-card {
  min-width: 0;
  display: grid;
  gap: 16px;
  border-radius: 20px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 86%, transparent);
  padding: 22px 24px;
  background: color-mix(in srgb, var(--sl-surface) 94%, transparent);
}

.next-home-operations-band__copy {
  display: grid;
  gap: 8px;
}

.next-home-operations-band__copy h2 {
  margin: 0;
  color: var(--sl-text-primary);
  font-size: clamp(1.35rem, 2.4vw, 1.8rem);
  line-height: 1.1;
}

.next-home-operations-band__action-row {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.next-home-operations-band__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding-top: 14px;
  border-top: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-sm);
}

.next-home-operations-band__footer strong {
  color: var(--sl-text-primary);
  font-weight: 600;
}

@media (max-width: 639px) {
  .next-home-operations-band__action-row {
    flex-direction: column;
  }

  .next-home-operations-band__action-row :deep(.sl-button) {
    width: 100%;
  }

  .next-home-operations-band__footer {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
