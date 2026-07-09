<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import { i18n } from "@language";
import type { ServerExtensionEntrySummary } from "@src/api/serverExtensions";
import ExtensionEntryCard from "./ExtensionEntryCard.vue";

interface Props {
  title: string;
  description: string;
  entries: ServerExtensionEntrySummary[];
  emptyMessage: string;
  formatSize: (bytes: number) => string;
  formatModifiedAt: (value: string | null) => string;
  resolveTypeLabel: (entry: ServerExtensionEntrySummary) => string;
}

defineProps<Props>();
</script>

<template>
  <section class="extension-entry-list">
    <div class="extension-entry-list__header">
      <div>
        <h3>{{ title }}</h3>
        <p>{{ description }}</p>
      </div>
      <span class="extension-entry-list__count">
        {{ i18n.t("servers.next.instance.extensions.entries_count", { count: entries.length }) }}
      </span>
    </div>

    <div v-if="entries.length > 0" class="extension-entry-list__grid">
      <ExtensionEntryCard
        v-for="entry in entries"
        :key="entry.relative_path"
        :entry="entry"
        :size-label="formatSize(entry.size_bytes)"
        :modified-label="formatModifiedAt(entry.modified_at)"
        :type-label="resolveTypeLabel(entry)"
      />
    </div>

    <SLCard v-else variant="outline" padding="md">
      <div class="extension-entry-list__empty">
        <strong>{{ title }}</strong>
        <span>{{ emptyMessage }}</span>
      </div>
    </SLCard>
  </section>
</template>

<style scoped>
.extension-entry-list {
  display: grid;
  gap: 14px;
}

.extension-entry-list__header {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
}

.extension-entry-list__header h3,
.extension-entry-list__header p {
  margin: 0;
}

.extension-entry-list__header h3,
.extension-entry-list__empty strong {
  color: var(--sl-text-primary);
}

.extension-entry-list__header p,
.extension-entry-list__count,
.extension-entry-list__empty span {
  color: var(--sl-text-secondary);
}

.extension-entry-list__grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 16px;
}

.extension-entry-list__empty {
  display: grid;
  gap: 6px;
  padding: 8px 4px;
}

@media (max-width: 767px) {
  .extension-entry-list__header {
    flex-direction: column;
  }
}
</style>
