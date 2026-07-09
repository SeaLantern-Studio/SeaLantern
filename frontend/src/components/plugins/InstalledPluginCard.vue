<script setup lang="ts">
import SLBadge from "@components/common/SLBadge.vue";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import { i18n } from "@language";
import type { PluginInfo } from "@type/plugin";
import { Layers } from "@lucide/vue";

defineProps<{
  plugin: PluginInfo;
  displayName: string;
  description: string;
  authorName: string | null;
  metaItems: string[];
  sceneTagLabel?: string | null;
  stateLabel: string;
  stateTone: "success" | "warning" | "neutral" | "error";
  iconUrl?: string;
  enabled: boolean;
  canToggle: boolean;
  toggleUnavailableMessage?: string;
  hasMissingRequiredDependencies: boolean;
  hasMissingOptionalDependencies: boolean;
  updateSummary: string | null;
  canOpenDetails: boolean;
}>();

defineEmits<{
  (e: "toggle", plugin: PluginInfo, nextEnabled: boolean): void;
  (e: "open-details", pluginId: string): void;
  (e: "open-repository", url: string): void;
}>();
</script>

<template>
  <SLCard class="installed-plugin-card" variant="outline">
    <div class="installed-plugin-card__content">
      <div class="installed-plugin-card__main">
        <div class="installed-plugin-card__icon-shell">
          <img
            v-if="iconUrl"
            :src="iconUrl"
            :alt="displayName"
            class="installed-plugin-card__icon-image"
          />
          <Layers v-else class="installed-plugin-card__icon-fallback" :size="26" />
        </div>

        <div class="installed-plugin-card__body">
          <div class="installed-plugin-card__header">
            <div class="installed-plugin-card__title-block">
              <div class="installed-plugin-card__title-row">
                <h3 class="installed-plugin-card__title">{{ displayName }}</h3>
                <span class="installed-plugin-card__version">v{{ plugin.manifest.version }}</span>
              </div>

              <p v-if="authorName" class="installed-plugin-card__subtitle">
                {{ i18n.t("plugins.next.card.author_prefix") }}: {{ authorName }}
              </p>
              <p v-else class="installed-plugin-card__subtitle">
                {{ i18n.t("plugins.next.card.id_prefix") }}: {{ plugin.manifest.id }}
              </p>
            </div>

            <div class="installed-plugin-card__status-block">
              <SLBadge :text="stateLabel" :variant="stateTone" size="small" rounded="full" />
              <SLBadge
                v-if="updateSummary"
                :text="updateSummary"
                variant="info"
                size="small"
                rounded="full"
              />
            </div>
          </div>

          <p v-if="description" class="installed-plugin-card__description">{{ description }}</p>
          <p
            v-else
            class="installed-plugin-card__description installed-plugin-card__description--muted"
          >
            {{ i18n.t("plugins.next.card.no_description") }}
          </p>

          <div class="installed-plugin-card__meta-row">
            <span v-for="item in metaItems" :key="item" class="installed-plugin-card__meta-pill">
              {{ item }}
            </span>
            <span
              v-if="sceneTagLabel"
              class="installed-plugin-card__meta-pill installed-plugin-card__meta-pill--info"
            >
              {{ sceneTagLabel }}
            </span>
            <span
              v-if="hasMissingRequiredDependencies"
              class="installed-plugin-card__meta-pill installed-plugin-card__meta-pill--error"
            >
              {{ i18n.t("plugins.next.card.missing_required") }}
            </span>
            <span
              v-else-if="hasMissingOptionalDependencies"
              class="installed-plugin-card__meta-pill installed-plugin-card__meta-pill--warning"
            >
              {{ i18n.t("plugins.next.card.missing_optional") }}
            </span>
          </div>
        </div>
      </div>

      <div class="installed-plugin-card__footer">
        <div class="installed-plugin-card__actions-left">
          <SLButton
            v-if="canOpenDetails"
            variant="ghost"
            size="sm"
            @click="$emit('open-details', plugin.manifest.id)"
          >
            {{ i18n.t("plugins.next.card.open_details") }}
          </SLButton>
          <SLButton
            v-if="plugin.manifest.repository"
            variant="ghost"
            size="sm"
            @click="$emit('open-repository', plugin.manifest.repository)"
          >
            {{ i18n.t("plugins.next.card.open_repository") }}
          </SLButton>
        </div>

        <div class="installed-plugin-card__actions-right">
          <span v-if="!canToggle" class="installed-plugin-card__toggle-note">{{
            toggleUnavailableMessage || i18n.t("plugins.next.card.toggle_unavailable")
          }}</span>
          <span
            v-else-if="hasMissingRequiredDependencies && !enabled"
            class="installed-plugin-card__toggle-note"
          >
            {{ i18n.t("plugins.next.card.toggle_requires_dependencies") }}
          </span>
          <SLSwitch
            v-if="canToggle"
            :modelValue="enabled"
            :disabled="hasMissingRequiredDependencies && !enabled"
            size="sm"
            @update:modelValue="$emit('toggle', plugin, Boolean($event))"
          />
        </div>
      </div>
    </div>
  </SLCard>
</template>

<style scoped>
.installed-plugin-card {
  min-width: 0;
}

.installed-plugin-card__content {
  display: grid;
  gap: 16px;
}

.installed-plugin-card__main {
  min-width: 0;
  display: flex;
  gap: 14px;
  align-items: flex-start;
}

.installed-plugin-card__icon-shell {
  width: 56px;
  height: 56px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 16px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 78%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 88%, transparent);
  overflow: hidden;
}

.installed-plugin-card__icon-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.installed-plugin-card__icon-fallback {
  color: var(--sl-text-tertiary);
}

.installed-plugin-card__body,
.installed-plugin-card__title-block {
  min-width: 0;
}

.installed-plugin-card__body {
  flex: 1;
  display: grid;
  gap: 10px;
}

.installed-plugin-card__header,
.installed-plugin-card__footer {
  display: flex;
  gap: 12px;
  justify-content: space-between;
  align-items: flex-start;
}

.installed-plugin-card__title-row,
.installed-plugin-card__status-block,
.installed-plugin-card__actions-left,
.installed-plugin-card__actions-right,
.installed-plugin-card__meta-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.installed-plugin-card__title {
  margin: 0;
  font-size: 1rem;
  color: var(--sl-text-primary);
}

.installed-plugin-card__version,
.installed-plugin-card__subtitle,
.installed-plugin-card__toggle-note {
  color: var(--sl-text-secondary);
  font-size: 0.82rem;
}

.installed-plugin-card__subtitle,
.installed-plugin-card__description {
  margin: 0;
  line-height: 1.6;
}

.installed-plugin-card__description {
  color: var(--sl-text-secondary);
}

.installed-plugin-card__description--muted,
.installed-plugin-card__toggle-note {
  color: var(--sl-text-tertiary);
}

.installed-plugin-card__meta-pill {
  padding: 4px 9px;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 74%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 92%, transparent);
  color: var(--sl-text-secondary);
  font-size: 0.74rem;
  line-height: 1;
}

.installed-plugin-card__meta-pill--warning {
  border-color: rgba(245, 158, 11, 0.28);
  color: var(--sl-warning);
}

.installed-plugin-card__meta-pill--info {
  border-color: color-mix(in srgb, var(--sl-primary) 26%, transparent);
  color: var(--sl-primary);
}

.installed-plugin-card__meta-pill--error {
  border-color: rgba(239, 68, 68, 0.28);
  color: var(--sl-error);
}

.installed-plugin-card__footer {
  align-items: center;
}

.installed-plugin-card__actions-right {
  align-items: center;
}

@media (max-width: 720px) {
  .installed-plugin-card__header,
  .installed-plugin-card__footer,
  .installed-plugin-card__main {
    flex-direction: column;
  }

  .installed-plugin-card__actions-right {
    width: 100%;
    justify-content: flex-start;
  }
}
</style>
