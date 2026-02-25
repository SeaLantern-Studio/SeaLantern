<script setup lang="ts">
import { ref } from "vue";
import DOMPurify from "dompurify";
import { useRegisterComponent } from "@composables/useRegisterComponent";

const PADDING_MAP = {
  none: "0",
  sm: "var(--sl-space-sm, 0.5rem)",
  md: "var(--sl-space-md, 1rem)",
  lg: "var(--sl-space-lg, 1.5rem)",
} as const;

type PaddingType = keyof typeof PADDING_MAP;

interface Props {
  title?: string;
  subtitle?: string;
  hoverable?: boolean;
  padding?: PaddingType;
  componentId?: string;
}

const props = withDefaults(defineProps<Props>(), {
  hoverable: false,
  padding: "md",
});

const paddingValue = PADDING_MAP[props.padding as PaddingType];

const elRef = ref<HTMLElement | null>(null);
const pluginFooterHtml = ref<string | null>(null);

const id = props.componentId ?? `sl-card-${Math.random().toString(36).slice(2, 8)}`;
useRegisterComponent(id, {
  type: "SLCard",
  get: (prop) => {
    if (prop === "pluginFooter") return pluginFooterHtml.value;
    if (prop === "title") return props.title;
    return undefined;
  },
  set: (prop, value) => {
    if (prop === "pluginFooter") {
      pluginFooterHtml.value = value
        ? DOMPurify.sanitize(value, {
            FORBID_TAGS: ["script", "iframe", "style", "link"],
            FORBID_ATTR: ["style"],
          })
        : null;
    }
  },
  call: () => undefined,
  on: () => () => {},
  el: () => elRef.value,
});
</script>

<template>
  <div
    ref="elRef"
    class="sl-card"
    :class="{ 'sl-card--hoverable': hoverable }"
    :style="{ padding: paddingValue }"
  >
    <div v-if="title || $slots.header || $slots.actions" class="sl-card-header">
      <div v-if="title" class="sl-card-header-text">
        <h3 class="sl-card-title">{{ title }}</h3>
        <p v-if="subtitle" class="sl-card-subtitle">{{ subtitle }}</p>
      </div>
      <slot name="header" />
      <slot v-if="$slots.actions" name="actions" />
    </div>

    <div class="sl-card-body">
      <slot />
    </div>

    <div v-if="$slots.footer" class="sl-card-footer">
      <slot name="footer" />
    </div>

    <div
      v-if="pluginFooterHtml"
      class="sl-card-footer sl-card-plugin-footer"
      v-html="pluginFooterHtml"
    />
  </div>
</template>

<style scoped>
.sl-card {
  display: flex;
  flex-direction: column;
  background: var(--sl-surface, #fff);
  border-radius: var(--sl-radius-lg, 12px);
  border: 1px solid var(--sl-border, #e2e8f0);
  box-shadow: var(--sl-shadow-sm, 0 1px 3px rgba(0, 0, 0, 0.04));
  position: relative;
  overflow: hidden;
  transition:
    box-shadow 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    transform 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    border-color 0.2s ease;
}

.sl-card::before {
  content: "";
  position: absolute;
  inset: 0;
  background: linear-gradient(
    135deg,
    rgba(14, 165, 233, 0.03) 0%,
    transparent 50%
  );
  opacity: 0;
  transition: opacity 0.3s ease;
  pointer-events: none;
}

.sl-card--hoverable {
  cursor: pointer;
}

.sl-card--hoverable:hover {
  box-shadow: 
    var(--sl-shadow-md, 0 4px 12px rgba(0, 0, 0, 0.08)),
    0 0 0 1px var(--sl-primary-bg, rgba(14, 165, 233, 0.1));
  transform: translateY(-2px);
  border-color: var(--sl-primary-light, #7dd3fc);
}

.sl-card--hoverable:hover::before {
  opacity: 1;
}

.sl-card--hoverable:active {
  transform: translateY(0);
  box-shadow: var(--sl-shadow-sm, 0 1px 3px rgba(0, 0, 0, 0.04));
}

.sl-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-bottom: var(--sl-space-md, 1rem);
}

.sl-card-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--sl-text-primary, #0f172a);
  margin: 0;
  transition: color 0.2s ease;
}

.sl-card--hoverable:hover .sl-card-title {
  color: var(--sl-primary, #0ea5e9);
}

.sl-card-subtitle {
  font-size: 0.8125rem;
  color: var(--sl-text-tertiary, #94a3b8);
  margin: 0.125rem 0 0 0;
}

.sl-card-body {
  flex: 1;
}

.sl-card-footer {
  margin-top: var(--sl-space-md, 1rem);
  padding-top: var(--sl-space-md, 1rem);
  border-top: 1px solid var(--sl-border-light, #f1f5f9);
}

/* Dark mode */
[data-theme="dark"] .sl-card {
  background: var(--sl-surface, #1e2130);
  border-color: var(--sl-border, rgba(255, 255, 255, 0.08));
}

[data-theme="dark"] .sl-card--hoverable:hover {
  border-color: var(--sl-primary, #60a5fa);
  box-shadow: 
    0 4px 12px rgba(0, 0, 0, 0.3),
    0 0 0 1px rgba(96, 165, 250, 0.2);
}
</style>
