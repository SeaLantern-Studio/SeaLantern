<script setup lang="ts">
const PADDING_MAP = {
  none: '0',
  sm: 'var(--sl-space-sm, 0.5rem)',
  md: 'var(--sl-space-md, 1rem)',
  lg: 'var(--sl-space-lg, 1.5rem)'
} as const


type PaddingType = keyof typeof PADDING_MAP

interface Props {
  title?: string
  subtitle?: string
  hoverable?: boolean
  padding?: PaddingType
}

const props = withDefaults(defineProps<Props>(), {
  hoverable: false,
  padding: 'md'
})


const paddingValue = PADDING_MAP[props.padding as PaddingType]
</script>

<template>
  <div
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
  </div>
</template>

<style scoped>
.sl-card {
  display: flex;
  flex-direction: column;
  background: var(--sl-surface, #fff);
  border-radius: var(--sl-radius-md, 6px);
  box-shadow: var(--sl-shadow-sm, 0 1px 3px rgba(0,0,0,0.1));
  transition: box-shadow 0.2s ease;
}

.sl-card--hoverable {
  cursor: pointer;
}

.sl-card--hoverable:hover {
  box-shadow: var(--sl-shadow-md, 0 4px 12px rgba(0,0,0,0.15));
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
  color: var(--sl-text-primary, #1a1a1a);
  margin: 0;
}

.sl-card-subtitle {
  font-size: 0.8125rem;
  color: var(--sl-text-tertiary, #666);
  margin: 0.125rem 0 0 0;
}

.sl-card-body {
  flex: 1;
}

.sl-card-footer {
  margin-top: var(--sl-space-md, 1rem);
  padding-top: var(--sl-space-md, 1rem);
  border-top: 1px solid var(--sl-border-light, #e5e5e5);
}
</style>