<script setup lang="ts">
import { ref } from "vue";
import { useRegisterComponent } from "../../composables/useRegisterComponent";

interface Props {
  modelValue?: boolean;
  label?: string;
  disabled?: boolean;
  componentId?: string;
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

const elRef = ref<HTMLElement | null>(null);
const id = props.componentId ?? `sl-switch-${Math.random().toString(36).slice(2, 8)}`;
useRegisterComponent(id, {
  type: "SLSwitch",
  get: (prop) => (prop === "value" ? props.modelValue : undefined),
  set: (prop, value) => {
    if (prop === "value") emit("update:modelValue", !!value);
  },
  call: () => undefined,
  on: (event, cb) => {
    if (event === "change") {
    }
    return () => {};
  },
  el: () => elRef.value,
});
</script>

<template>
  <label ref="elRef" class="sl-switch-wrapper" :class="{ disabled: props.disabled }">
    <div
      class="sl-switch"
      :class="{ active: props.modelValue }"
      :tabindex="props.disabled ? -1 : 0"
      role="switch"
      :aria-checked="props.modelValue"
      :aria-disabled="props.disabled"
      @click="handleClick"
      @keydown.enter.prevent="handleClick"
      @keydown.space.prevent="handleClick"
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
  transition: background var(--sl-transition-fast, 150ms) ease;
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
  transition: transform var(--sl-transition-fast, 150ms) ease;
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
