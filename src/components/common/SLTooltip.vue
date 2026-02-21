<script setup lang="ts">
import { ref, onUnmounted } from "vue";

interface Props {
  content?: string;
  position?: "top" | "bottom" | "left" | "right";
  delay?: number;
}

const props = withDefaults(defineProps<Props>(), {
  position: "top",
  delay: 300,
});

const visible = ref(false);
let timer: ReturnType<typeof setTimeout> | null = null;

const show = () => {
  timer = setTimeout(() => {
    visible.value = true;
  }, props.delay);
};

const hide = () => {
  if (timer) {
    clearTimeout(timer);
    timer = null;
  }
  visible.value = false;
};

onUnmounted(() => {
  if (timer) {
    clearTimeout(timer);
    timer = null;
  }
});
</script>

<template>
  <div class="sl-tooltip-trigger" @mouseenter="show" @mouseleave="hide" @focus="show" @blur="hide">
    <slot />
    <Transition name="sl-tooltip">
      <div
        v-if="visible && content"
        class="sl-tooltip"
        :class="`sl-tooltip--${position}`"
        role="tooltip"
      >
        {{ content }}
        <div class="sl-tooltip__arrow" />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.sl-tooltip-trigger {
  position: relative;
  display: inline-flex;
}

.sl-tooltip {
  position: absolute;
  z-index: 9999;
  padding: 6px 10px;
  font-size: 0.75rem;
  font-weight: 500;
  line-height: 1.4;
  color: var(--sl-text-inverse, #ffffff);
  background: var(--sl-text-primary, #0f172a);
  border-radius: var(--sl-radius-sm, 6px);
  white-space: nowrap;
  pointer-events: none;
  box-shadow: var(--sl-shadow-md, 0 4px 12px rgba(0, 0, 0, 0.06));
}

.sl-tooltip__arrow {
  position: absolute;
  width: 6px;
  height: 6px;
  background: inherit;
  transform: rotate(45deg);
}

/* -- Position: top -- */
.sl-tooltip--top {
  bottom: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
}

.sl-tooltip--top .sl-tooltip__arrow {
  bottom: -3px;
  left: 50%;
  margin-left: -3px;
}

/* -- Position: bottom -- */
.sl-tooltip--bottom {
  top: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
}

.sl-tooltip--bottom .sl-tooltip__arrow {
  top: -3px;
  left: 50%;
  margin-left: -3px;
}

/* -- Position: left -- */
.sl-tooltip--left {
  right: calc(100% + 8px);
  top: 50%;
  transform: translateY(-50%);
}

.sl-tooltip--left .sl-tooltip__arrow {
  right: -3px;
  top: 50%;
  margin-top: -3px;
}

/* -- Position: right -- */
.sl-tooltip--right {
  left: calc(100% + 8px);
  top: 50%;
  transform: translateY(-50%);
}

.sl-tooltip--right .sl-tooltip__arrow {
  left: -3px;
  top: 50%;
  margin-top: -3px;
}

/* -- Transition -- */
.sl-tooltip-enter-active,
.sl-tooltip-leave-active {
  transition:
    opacity var(--sl-transition-fast, 0.15s ease),
    transform var(--sl-transition-fast, 0.15s ease);
}

.sl-tooltip-enter-from,
.sl-tooltip-leave-to {
  opacity: 0;
}

.sl-tooltip--top.sl-tooltip-enter-from,
.sl-tooltip--top.sl-tooltip-leave-to {
  transform: translateX(-50%) translateY(4px);
}

.sl-tooltip--bottom.sl-tooltip-enter-from,
.sl-tooltip--bottom.sl-tooltip-leave-to {
  transform: translateX(-50%) translateY(-4px);
}

.sl-tooltip--left.sl-tooltip-enter-from,
.sl-tooltip--left.sl-tooltip-leave-to {
  transform: translateY(-50%) translateX(4px);
}

.sl-tooltip--right.sl-tooltip-enter-from,
.sl-tooltip--right.sl-tooltip-leave-to {
  transform: translateY(-50%) translateX(-4px);
}
</style>
