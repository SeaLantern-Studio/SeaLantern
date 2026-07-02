<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";

interface Props {
  hasPluginsDir: boolean;
  hasModsDir: boolean;
  pluginCount: number;
  modCount: number;
  otherCount: number;
  refreshing: boolean;
}

defineProps<Props>();

const emit = defineEmits<{
  refresh: [];
}>();
</script>

<template>
  <section class="extensions-summary-bar">
    <div class="extensions-summary-bar__stats">
      <article class="extensions-summary-bar__stat">
        <strong>{{
          hasPluginsDir
            ? i18n.t("servers.next.instance.extensions.detected")
            : i18n.t("servers.next.instance.extensions.missing")
        }}</strong>
        <span>{{ i18n.t("servers.next.instance.extensions.plugins_dir") }}</span>
      </article>

      <article class="extensions-summary-bar__stat">
        <strong>{{
          hasModsDir
            ? i18n.t("servers.next.instance.extensions.detected")
            : i18n.t("servers.next.instance.extensions.missing")
        }}</strong>
        <span>{{ i18n.t("servers.next.instance.extensions.mods_dir") }}</span>
      </article>

      <article class="extensions-summary-bar__stat extensions-summary-bar__stat--accent">
        <strong>{{ pluginCount }}</strong>
        <span>{{ i18n.t("servers.next.instance.extensions.plugins_count") }}</span>
      </article>

      <article class="extensions-summary-bar__stat extensions-summary-bar__stat--accent-alt">
        <strong>{{ modCount }}</strong>
        <span>{{ i18n.t("servers.next.instance.extensions.mods_count") }}</span>
      </article>

      <article class="extensions-summary-bar__stat">
        <strong>{{ otherCount }}</strong>
        <span>{{ i18n.t("servers.next.instance.extensions.other_count") }}</span>
      </article>
    </div>

    <div class="extensions-summary-bar__actions">
      <span class="extensions-summary-bar__hint">
        {{ i18n.t("servers.next.instance.extensions.read_only_hint") }}
      </span>
      <SLButton variant="secondary" size="sm" :loading="refreshing" @click="emit('refresh')">
        {{ i18n.t("servers.next.instance.extensions.refresh") }}
      </SLButton>
    </div>
  </section>
</template>

<style scoped>
.extensions-summary-bar,
.extensions-summary-bar__stats,
.extensions-summary-bar__actions {
  display: flex;
}

.extensions-summary-bar {
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
  padding: 18px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 84%, transparent);
  border-radius: 22px;
  background: color-mix(in srgb, var(--sl-surface) 92%, transparent);
}

.extensions-summary-bar__stats {
  flex-wrap: wrap;
  gap: 12px;
}

.extensions-summary-bar__stat {
  min-width: 128px;
  display: grid;
  gap: 4px;
  padding: 12px 14px;
  border-radius: 16px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 80%, transparent);
}

.extensions-summary-bar__stat strong {
  color: var(--sl-text-primary);
  font-size: 1.2rem;
}

.extensions-summary-bar__stat span,
.extensions-summary-bar__hint {
  color: var(--sl-text-secondary);
}

.extensions-summary-bar__stat--accent strong {
  color: var(--sl-primary);
}

.extensions-summary-bar__stat--accent-alt strong {
  color: #7c3aed;
}

.extensions-summary-bar__actions {
  align-items: center;
  gap: 12px;
}

@media (max-width: 767px) {
  .extensions-summary-bar__actions {
    width: 100%;
    justify-content: space-between;
  }
}
</style>
