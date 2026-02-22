<script setup lang="ts">
import { watch, onUnmounted, ref, nextTick } from "vue";
import { X } from "lucide-vue-next";
import { i18n } from "@language";

interface Props {
  visible: boolean;
  title?: string;
  width?: string;
  closeOnOverlay?: boolean;
  autoClose?: number;
}

const props = withDefaults(defineProps<Props>(), {
  width: "480px",
  closeOnOverlay: true,
  autoClose: 0,
});

const emit = defineEmits<{
  close: [];
}>();

const handleClose = () => emit("close");

const modalRef = ref<HTMLElement | null>(null);
let previousActiveElement: Element | null = null;

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    handleClose();
  }
}

let autoCloseTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  () => props.visible,
  (newVisible) => {
    if (autoCloseTimer) {
      clearTimeout(autoCloseTimer);
      autoCloseTimer = null;
    }

    if (newVisible) {
      previousActiveElement = document.activeElement;
      nextTick(() => {
        modalRef.value?.focus();
      });
      document.addEventListener("keydown", handleKeydown);

      if (props.autoClose > 0) {
        autoCloseTimer = setTimeout(() => {
          handleClose();
        }, props.autoClose);
      }
    } else {
      document.removeEventListener("keydown", handleKeydown);
      if (previousActiveElement instanceof HTMLElement) {
        previousActiveElement.focus();
      }
      previousActiveElement = null;
    }
  },
  { immediate: true },
);

onUnmounted(() => {
  if (autoCloseTimer) {
    clearTimeout(autoCloseTimer);
  }
  document.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="sl-modal-overlay" @click="closeOnOverlay && handleClose()">
      <div
        ref="modalRef"
        class="sl-modal glass-strong"
        :style="{ maxWidth: width }"
        tabindex="-1"
        role="dialog"
        aria-modal="true"
        @click.stop
      >
        <div class="sl-modal-header">
          <h3 v-if="title" class="sl-modal-title">{{ title }}</h3>
          <button
            class="sl-modal-close"
            @click="handleClose"
            :aria-label="i18n.t('common.close_modal')"
          >
            <X :size="18" />
          </button>
        </div>
        <div class="sl-modal-body">
          <slot />
        </div>
        <div v-if="$slots.footer" class="sl-modal-footer">
          <slot name="footer" />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style src="@styles/components/common/SLModal.css" scoped></style>
