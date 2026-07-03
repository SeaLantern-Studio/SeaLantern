<script setup lang="ts">
import { i18n } from "@language";
import type { NextHomeServerCardModel } from "@src/pages/home/useNextHomePage";
import NextHomeServerCard from "./NextHomeServerCard.vue";

defineProps<{
  featuredServer: NextHomeServerCardModel | null;
  secondaryServers: NextHomeServerCardModel[];
  serverCountLabel: string;
  description: string;
  emptyTitle: string;
  emptyDescription: string;
  previewDataset?: boolean;
  cardId?: string;
  section?: string;
  editable?: boolean;
}>();

const emit = defineEmits<{
  toggle: [serverId: string];
}>();
</script>

<template>
  <section
    class="next-home-workspace-band"
    :data-card-id="cardId"
    :data-layout-section="section"
    :data-layout-editable="editable ? 'true' : 'false'"
  >
    <header class="next-home-workspace-band__header">
      <div>
        <span class="next-home-workspace-band__eyebrow">{{
          i18n.t("shell.home_workspace_eyebrow")
        }}</span>
        <h2>{{ serverCountLabel }}</h2>
        <p v-if="description">{{ description }}</p>
      </div>
    </header>

    <div v-if="featuredServer" class="next-home-workspace-band__grid">
      <NextHomeServerCard
        class="next-home-workspace-band__featured"
        :server="featuredServer"
        :featured="true"
        :preview-dataset="previewDataset"
        @toggle="emit('toggle', $event)"
      />

      <div class="next-home-workspace-band__secondary-list">
        <NextHomeServerCard
          v-for="server in secondaryServers"
          :key="server.id"
          :server="server"
          :preview-dataset="previewDataset"
          @toggle="emit('toggle', $event)"
        />
      </div>
    </div>

    <div v-else class="next-home-workspace-band__empty surface-panel">
      <h3>{{ emptyTitle }}</h3>
      <p v-if="emptyDescription">{{ emptyDescription }}</p>
    </div>
  </section>
</template>

<style scoped>
.next-home-workspace-band {
  min-width: 0;
  display: grid;
  gap: 16px;
}

.next-home-workspace-band__header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 16px;
}

.next-home-workspace-band__eyebrow {
  font-size: var(--sl-font-size-xs);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--sl-text-tertiary);
}

.next-home-workspace-band__header h2,
.next-home-workspace-band__empty h3,
.next-home-workspace-band__empty p {
  margin: 0;
}

.next-home-workspace-band__header h2 {
  margin-top: 6px;
  color: var(--sl-text-primary);
  font-size: clamp(1.3rem, 2.3vw, 1.75rem);
}

.next-home-workspace-band__header p {
  margin: 6px 0 0;
  max-width: 62ch;
  color: var(--sl-text-secondary);
  line-height: var(--sl-line-height-relaxed);
}

.next-home-workspace-band__grid {
  display: grid;
  grid-template-columns: minmax(0, 1.1fr) minmax(320px, 0.9fr);
  gap: 16px;
}

.next-home-workspace-band__secondary-list {
  min-width: 0;
  display: grid;
  gap: 16px;
}

.surface-panel {
  padding: 22px;
  border-radius: 20px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 84%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 74%, white);
}

.next-home-workspace-band__empty {
  display: grid;
  gap: 8px;
}

.next-home-workspace-band__empty h3 {
  color: var(--sl-text-primary);
}

.next-home-workspace-band__empty p {
  color: var(--sl-text-secondary);
  line-height: var(--sl-line-height-relaxed);
}

@media (max-width: 1023px) {
  .next-home-workspace-band__grid {
    grid-template-columns: 1fr;
  }
}
</style>
