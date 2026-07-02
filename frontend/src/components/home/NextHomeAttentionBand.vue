<script setup lang="ts">
import { i18n } from "@language";

defineProps<{
  alerts: Array<{ server: string; line: string }>;
  title: string;
  emptyTitle: string;
  emptyDescription: string;
  cardId?: string;
  section?: string;
  editable?: boolean;
}>();
</script>

<template>
  <section
    class="next-home-attention-band surface-panel"
    :data-card-id="cardId"
    :data-layout-section="section"
    :data-layout-editable="editable ? 'true' : 'false'"
  >
    <header class="next-home-attention-band__header">
      <span class="next-home-attention-band__eyebrow">{{
        i18n.t("shell.home_attention_eyebrow")
      }}</span>
      <h2>{{ title }}</h2>
    </header>

    <div v-if="alerts.length > 0" class="next-home-attention-band__list">
      <article
        v-for="(alert, index) in alerts"
        :key="`${alert.server}-${index}`"
        class="next-home-attention-band__item"
      >
        <span class="next-home-attention-band__server">{{ alert.server }}</span>
        <p>{{ alert.line }}</p>
      </article>
    </div>

    <div v-else class="next-home-attention-band__empty">
      <h3>{{ emptyTitle }}</h3>
      <p>{{ emptyDescription }}</p>
    </div>
  </section>
</template>

<style scoped>
.surface-panel {
  min-width: 0;
  padding: 20px;
  display: grid;
  gap: 16px;
  border-radius: 20px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 84%, transparent);
  background: color-mix(in srgb, var(--sl-surface) 94%, transparent);
}

.next-home-attention-band__header,
.next-home-attention-band__empty {
  display: grid;
  gap: 8px;
}

.next-home-attention-band__eyebrow {
  font-size: var(--sl-font-size-xs);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--sl-text-tertiary);
}

.next-home-attention-band__header h2,
.next-home-attention-band__empty h3,
.next-home-attention-band__empty p,
.next-home-attention-band__item p {
  margin: 0;
}

.next-home-attention-band__header h2,
.next-home-attention-band__empty h3 {
  color: var(--sl-text-primary);
}

.next-home-attention-band__list {
  display: grid;
  gap: 12px;
}

.next-home-attention-band__item {
  display: grid;
  gap: 8px;
  padding: 14px 16px;
  border-radius: 16px;
  background: #161c27;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.next-home-attention-band__server {
  display: inline-flex;
  align-items: center;
  justify-self: start;
  min-height: 24px;
  padding: 0 10px;
  border-radius: 999px;
  background: rgba(59, 130, 246, 0.16);
  color: #93c5fd;
  font-size: var(--sl-font-size-xs);
  font-weight: 600;
}

.next-home-attention-band__item p {
  color: #e2e8f0;
  font-family: var(--sl-font-mono);
  font-size: 0.78rem;
  line-height: 1.5;
  word-break: break-word;
}

.next-home-attention-band__empty p {
  color: var(--sl-text-secondary);
  line-height: var(--sl-line-height-relaxed);
}
</style>
