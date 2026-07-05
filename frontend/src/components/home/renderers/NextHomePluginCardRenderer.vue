<script setup lang="ts">
import { computed } from "vue";
import { i18n } from "@language";
import { resolveNextHomeCardTitle } from "@src/pages/home/cardMeta";
import type {
  NextHomeCardRendererProps,
  NextHomeResolvedCardRenderer,
} from "@src/pages/home/cardRendererContract";

const props = defineProps<
  NextHomeCardRendererProps & {
    resolution?: NextHomeResolvedCardRenderer;
  }
>();

const rendererKey = computed(() => props.meta.pluginRendererKey ?? props.instance.kind);
const resolvedTitle = computed(() => resolveNextHomeCardTitle(props.meta));

const fallbackMessage = computed(() => {
  if (props.resolution?.status === "plugin-pending") {
    return i18n.t("shell.home_plugin_card_placeholder");
  }

  return i18n.t("shell.home_plugin_card_unavailable");
});
</script>

<template>
  <section
    class="next-home-plugin-card-renderer"
    :data-card-id="instance.instanceId"
    :data-layout-section="meta.section"
    :data-layout-editable="meta.movable ? 'true' : 'false'"
  >
    <header class="next-home-plugin-card-renderer__header">
      <span class="next-home-plugin-card-renderer__eyebrow">{{
        i18n.t("shell.home_plugin_card_eyebrow")
      }}</span>
      <h3>{{ resolvedTitle }}</h3>
    </header>

    <p class="next-home-plugin-card-renderer__body">
      {{ fallbackMessage }}
    </p>

    <dl class="next-home-plugin-card-renderer__meta">
      <div>
        <dt>{{ i18n.t("shell.home_plugin_card_kind_label") }}</dt>
        <dd>{{ instance.kind }}</dd>
      </div>
      <div>
        <dt>{{ i18n.t("shell.home_plugin_card_renderer_label") }}</dt>
        <dd>{{ rendererKey }}</dd>
      </div>
    </dl>
  </section>
</template>

<style scoped>
.next-home-plugin-card-renderer {
  min-width: 0;
  height: 100%;
  display: grid;
  align-content: start;
  gap: 14px;
  padding: 20px;
  border-radius: 20px;
  border: 1px dashed color-mix(in srgb, var(--sl-border) 86%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 68%, white);
}

.next-home-plugin-card-renderer__header,
.next-home-plugin-card-renderer__meta {
  display: grid;
  gap: 8px;
}

.next-home-plugin-card-renderer__eyebrow,
.next-home-plugin-card-renderer__meta dt {
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-xs);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.next-home-plugin-card-renderer__header h3,
.next-home-plugin-card-renderer__body,
.next-home-plugin-card-renderer__meta dd {
  margin: 0;
}

.next-home-plugin-card-renderer__header h3,
.next-home-plugin-card-renderer__meta dd {
  color: var(--sl-text-primary);
}

.next-home-plugin-card-renderer__body {
  color: var(--sl-text-secondary);
  line-height: var(--sl-line-height-relaxed);
}

.next-home-plugin-card-renderer__meta {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.next-home-plugin-card-renderer__meta div {
  min-width: 0;
  display: grid;
  gap: 4px;
  padding: 12px;
  border-radius: 14px;
  background: color-mix(in srgb, var(--sl-surface) 90%, transparent);
}

.next-home-plugin-card-renderer__meta dd {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
