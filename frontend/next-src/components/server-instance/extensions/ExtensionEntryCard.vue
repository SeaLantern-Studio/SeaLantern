<script setup lang="ts">
import { computed } from "vue";
import SLCard from "@components/common/SLCard.vue";
import { i18n } from "@language";
import type { ServerExtensionEntrySummary } from "@next-src/api/serverExtensions";

interface Props {
  entry: ServerExtensionEntrySummary;
  sizeLabel: string;
  modifiedLabel: string;
  typeLabel: string;
}

const props = defineProps<Props>();

const kindToneClass = computed(() => ({
  "extension-entry-card__kind": true,
  "extension-entry-card__kind--plugin": props.entry.kind === "plugin",
  "extension-entry-card__kind--mod": props.entry.kind === "mod",
  "extension-entry-card__kind--other": props.entry.kind === "other",
}));
</script>

<template>
  <SLCard variant="outline" class="extension-entry-card" padding="md">
    <div class="extension-entry-card__header">
      <div class="extension-entry-card__title-group">
        <strong class="extension-entry-card__name">{{ entry.name }}</strong>
        <span :class="kindToneClass">{{ typeLabel }}</span>
      </div>
      <span class="extension-entry-card__size">{{ sizeLabel }}</span>
    </div>

    <dl class="extension-entry-card__meta">
      <div class="extension-entry-card__meta-row">
        <dt>{{ i18n.t("servers.next.instance.extensions.relative_path") }}</dt>
        <dd>{{ entry.relative_path }}</dd>
      </div>
      <div class="extension-entry-card__meta-row">
        <dt>{{ i18n.t("servers.next.instance.extensions.modified_at") }}</dt>
        <dd>{{ modifiedLabel }}</dd>
      </div>
      <div class="extension-entry-card__meta-row">
        <dt>{{ i18n.t("servers.next.instance.extensions.file_type") }}</dt>
        <dd>{{ typeLabel }}</dd>
      </div>
    </dl>
  </SLCard>
</template>

<style scoped>
.extension-entry-card,
.extension-entry-card__header,
.extension-entry-card__title-group,
.extension-entry-card__meta-row {
  display: flex;
}

.extension-entry-card {
  height: 100%;
}

.extension-entry-card__header,
.extension-entry-card__meta-row {
  justify-content: space-between;
  gap: 12px;
}

.extension-entry-card__header {
  align-items: flex-start;
  margin-bottom: 14px;
}

.extension-entry-card__title-group {
  min-width: 0;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.extension-entry-card__name,
.extension-entry-card__size,
.extension-entry-card__meta-row dd {
  color: var(--sl-text-primary);
}

.extension-entry-card__name,
.extension-entry-card__meta-row dd {
  word-break: break-word;
}

.extension-entry-card__kind {
  display: inline-flex;
  align-items: center;
  min-height: 24px;
  padding: 0 10px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 84%, transparent);
  color: var(--sl-text-secondary);
  font-size: 0.8rem;
}

.extension-entry-card__kind--plugin {
  color: var(--sl-primary);
}

.extension-entry-card__kind--mod {
  color: #7c3aed;
}

.extension-entry-card__kind--other {
  color: var(--sl-text-secondary);
}

.extension-entry-card__meta {
  display: grid;
  gap: 10px;
  margin: 0;
}

.extension-entry-card__meta-row {
  align-items: baseline;
}

.extension-entry-card__meta-row dt {
  color: var(--sl-text-secondary);
  font-size: 0.86rem;
}

.extension-entry-card__meta-row dd {
  margin: 0;
  text-align: right;
  font-size: 0.92rem;
}

@media (max-width: 767px) {
  .extension-entry-card__header,
  .extension-entry-card__meta-row {
    flex-direction: column;
  }

  .extension-entry-card__meta-row dd {
    text-align: left;
  }
}
</style>
