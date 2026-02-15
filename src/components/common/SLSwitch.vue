<script setup lang="ts">
interface Props {
  modelValue?: boolean;
  label?: string;
  disabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: false,
  disabled: false,
});

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();

const handleClick = () => {
  if (!props.disabled) {
    emit("update:modelValue", !props.modelValue);
  }
};
</script>

<template>
  <label class="sl-switch-wrapper" :class="{ disabled: props.disabled }">
    <div
      class="sl-switch"
      :class="{ active: props.modelValue }"
      @click="handleClick"
      :aria-checked="props.modelValue"
      role="switch"
    >
      <div class="sl-switch-thumb" />
    </div>
    <span v-if="props.label" class="sl-switch-label">{{ props.label }}</span>
  </label>
</template>

<style scoped>
.sl-switch-wrapper {
  display: inline-flex;
  align-items: center;
  user-select: none;
}

.sl-switch-wrapper:not(.disabled) {
  cursor: pointer;
}

.sl-switch-wrapper.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.sl-switch-wrapper > * + * {
  margin-left: var(--sl-space-sm, 0.5rem);
}

.sl-switch {
  --switch-width: 40px;
  --switch-height: 22px;
  --thumb-size: calc(var(--switch-height) - 4px);
  --thumb-offset: 2px;
  --thumb-translate: calc(var(--switch-width) - var(--thumb-size) - var(--thumb-offset) * 2);

  position: relative;
  width: var(--switch-width);
  height: var(--switch-height);
  background: var(--sl-border, #e5e7eb);
  border-radius: var(--sl-radius-full, 9999px);
  transition: background 0.3s ease;
  flex-shrink: 0;
}

.sl-switch.active {
  background: var(--sl-primary, #3b82f6);
}

.sl-switch-thumb {
  position: absolute;
  top: var(--thumb-offset);
  left: var(--thumb-offset);
  width: var(--thumb-size);
  height: var(--thumb-size);
  background: var(--sl-surface, white);
  border-radius: 50%;
  box-shadow: var(--sl-shadow-sm, 0 1px 2px 0 rgb(0 0 0 / 0.05));
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  will-change: transform;
}

.sl-switch.active .sl-switch-thumb {
  transform: translateX(var(--thumb-translate));
}

.sl-switch-label {
  font-size: 0.875rem;
  color: var(--sl-text-secondary, #6b7280);
  line-height: 1;
}
</style>
