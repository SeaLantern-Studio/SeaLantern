<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from "vue";

interface Props {
  content: string;
  delay?: number;
}

const props = defineProps<Props>();

const visible = ref(false);
const triggerRef = ref<HTMLElement | null>(null);
const tooltipRef = ref<HTMLElement | null>(null);
const tooltipStyle = ref<Record<string, string>>({});
const placement = ref<"top" | "bottom">("top");
let showTimer: ReturnType<typeof setTimeout> | null = null;

const isInteractive = computed(() => Boolean(props.content));

function updatePosition() {
  if (!triggerRef.value) {
    return;
  }

  const rect = triggerRef.value.getBoundingClientRect();
  const tooltipRect = tooltipRef.value?.getBoundingClientRect();
  const tooltipWidth = tooltipRect?.width ?? 0;
  const tooltipHeight = tooltipRect?.height ?? 0;
  const viewportWidth = window.innerWidth;
  const viewportHeight = window.innerHeight;
  const margin = 12;
  const idealLeft = rect.left + rect.width / 2;
  const minLeft = margin + tooltipWidth / 2;
  const maxLeft = viewportWidth - margin - tooltipWidth / 2;
  const left = Math.min(Math.max(idealLeft, minLeft), Math.max(minLeft, maxLeft));
  const canPlaceAbove = rect.top - tooltipHeight - 12 >= margin;
  const canPlaceBelow = rect.bottom + tooltipHeight + 12 <= viewportHeight - margin;

  let top = rect.top - 8;

  if (canPlaceAbove || !canPlaceBelow) {
    placement.value = "top";
    top = Math.max(rect.top - 8, tooltipHeight + margin);
  } else {
    placement.value = "bottom";
    top = Math.min(rect.bottom + 8, viewportHeight - margin - tooltipHeight);
  }

  tooltipStyle.value = {
    left: `${left}px`,
    top: `${top}px`,
  };
}

async function show() {
  if (!isInteractive.value) {
    return;
  }

  if (showTimer) {
    clearTimeout(showTimer);
  }

  const openTooltip = async () => {
    visible.value = true;
    await nextTick();
    updatePosition();
  };

  if (props.delay && props.delay > 0) {
    showTimer = setTimeout(() => {
      void openTooltip();
      showTimer = null;
    }, props.delay);
    return;
  }

  await openTooltip();
}

function hide() {
  if (showTimer) {
    clearTimeout(showTimer);
    showTimer = null;
  }
  visible.value = false;
}

function handleWindowChange() {
  if (visible.value) {
    updatePosition();
  }
}

onMounted(() => {
  if (typeof window !== "undefined") {
    window.addEventListener("scroll", handleWindowChange, true);
    window.addEventListener("resize", handleWindowChange);
  }
});

onUnmounted(() => {
  if (showTimer) {
    clearTimeout(showTimer);
  }
  if (typeof window !== "undefined") {
    window.removeEventListener("scroll", handleWindowChange, true);
    window.removeEventListener("resize", handleWindowChange);
  }
});
</script>

<template>
  <span
    ref="triggerRef"
    class="sl-tooltip"
    @mouseenter="show"
    @mouseleave="hide"
    @focusin="show"
    @focusout="hide"
  >
    <slot />
    <Teleport to="body">
      <transition name="sl-tooltip-fade">
        <span
          v-if="visible && content"
          ref="tooltipRef"
          class="sl-tooltip__content"
          :class="`sl-tooltip__content--${placement}`"
          :style="tooltipStyle"
          role="tooltip"
        >
          {{ content }}
        </span>
      </transition>
    </Teleport>
  </span>
</template>

<style scoped>
.sl-tooltip {
  position: relative;
  display: inline-flex;
  align-items: center;
}

.sl-tooltip__content {
  position: fixed;
  transform: translate(-50%, -100%);
  max-width: min(360px, calc(100vw - 32px));
  padding: 8px 10px;
  border-radius: var(--sl-radius-sm);
  background: #ffffff;
  color: var(--sl-text-primary);
  border: 1px solid var(--sl-primary-light);
  box-shadow: var(--sl-shadow-md);
  font-size: var(--sl-font-size-xs);
  line-height: 1.45;
  white-space: normal;
  word-break: break-word;
  z-index: 9999;
  pointer-events: none;
}

.sl-tooltip__content::after,
.sl-tooltip__content::before {
  content: "";
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  border-style: solid;
}

.sl-tooltip__content::after {
  border-width: 5px;
}

.sl-tooltip__content::before {
  border-width: 6px;
}

.sl-tooltip__content--top::after {
  top: 100%;
  border-color: #ffffff transparent transparent transparent;
}

.sl-tooltip__content--top::before {
  top: calc(100% + 1px);
  border-color: var(--sl-primary-light) transparent transparent transparent;
}

.sl-tooltip__content--bottom {
  transform: translate(-50%, 0);
}

.sl-tooltip__content--bottom::after {
  bottom: 100%;
  border-color: transparent transparent #ffffff transparent;
}

.sl-tooltip__content--bottom::before {
  bottom: calc(100% + 1px);
  border-color: transparent transparent var(--sl-primary-light) transparent;
}

.sl-tooltip-fade-enter-active,
.sl-tooltip-fade-leave-active {
  transition:
    opacity var(--sl-transition-fast),
    transform var(--sl-transition-fast);
}

.sl-tooltip-fade-enter-from,
.sl-tooltip-fade-leave-to {
  opacity: 0;
  transform: translate(-50%, calc(-100% + 4px));
}

.sl-tooltip__content--bottom.sl-tooltip-fade-enter-from,
.sl-tooltip__content--bottom.sl-tooltip-fade-leave-to {
  transform: translate(-50%, -4px);
}
</style>
