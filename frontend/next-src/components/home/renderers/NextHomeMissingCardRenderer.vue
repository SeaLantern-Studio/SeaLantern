<script setup lang="ts">
import { computed } from "vue";
import { i18n } from "@language";
import type { NextHomeCardRendererProps, NextHomeResolvedCardRenderer } from "../../../pages/home/cardRendererContract";

const props = defineProps<NextHomeCardRendererProps & {
  resolution?: NextHomeResolvedCardRenderer;
}>();

const fallbackMessage = computed(() => {
  if (i18n.getLocale().startsWith("zh")) {
    return "当前卡片类型已存在于首页布局元数据中，但没有可用的 renderer 注册，暂时无法渲染。";
  }

  return "This homepage card type exists in the layout registry, but no renderer is registered for it right now.";
});

const rendererKey = computed(() => props.meta.pluginRendererKey ?? props.instance.kind);
</script>

<template>
  <section
    class="next-home-missing-card-renderer"
    :data-card-id="instance.instanceId"
    :data-layout-section="meta.section"
    data-renderer-status="missing"
  >
    <header class="next-home-missing-card-renderer__header">
      <span class="next-home-missing-card-renderer__eyebrow">Renderer missing</span>
      <h3>{{ i18n.t(meta.titleKey) }}</h3>
    </header>

    <p class="next-home-missing-card-renderer__body">
      {{ fallbackMessage }}
    </p>

    <dl class="next-home-missing-card-renderer__meta">
      <div>
        <dt>{{ i18n.t('shell.home_plugin_card_kind_label') }}</dt>
        <dd>{{ instance.kind }}</dd>
      </div>
      <div>
        <dt>{{ i18n.t('shell.home_plugin_card_renderer_label') }}</dt>
        <dd>{{ rendererKey }}</dd>
      </div>
    </dl>
  </section>
</template>

<style scoped>
.next-home-missing-card-renderer {
  min-width: 0;
  height: 100%;
  display: grid;
  align-content: start;
  gap: 14px;
  padding: 20px;
  border-radius: 20px;
  border: 1px dashed color-mix(in srgb, var(--sl-danger) 42%, var(--sl-border));
  background: color-mix(in srgb, var(--sl-danger) 8%, var(--sl-surface));
}

.next-home-missing-card-renderer__header,
.next-home-missing-card-renderer__meta {
  display: grid;
  gap: 8px;
}

.next-home-missing-card-renderer__eyebrow,
.next-home-missing-card-renderer__meta dt {
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-xs);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.next-home-missing-card-renderer__header h3,
.next-home-missing-card-renderer__body,
.next-home-missing-card-renderer__meta dd {
  margin: 0;
}

.next-home-missing-card-renderer__header h3,
.next-home-missing-card-renderer__meta dd {
  color: var(--sl-text-primary);
}

.next-home-missing-card-renderer__body {
  color: var(--sl-text-secondary);
  line-height: var(--sl-line-height-relaxed);
}

.next-home-missing-card-renderer__meta {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.next-home-missing-card-renderer__meta div {
  min-width: 0;
  display: grid;
  gap: 4px;
  padding: 12px;
  border-radius: 14px;
  background: color-mix(in srgb, var(--sl-surface) 92%, transparent);
}

.next-home-missing-card-renderer__meta dd {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
