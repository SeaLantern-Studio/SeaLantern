<script setup lang="ts">
import { ref, watch, onUnmounted, computed, onMounted } from "vue";
import { AlertTriangle, Check, Info, X, XCircle } from "lucide-vue-next";
import { i18n } from "@language";

type NotificationType = "success" | "error" | "warning" | "info";
type Position =
  | "top-left"
  | "top-center"
  | "top-right"
  | "bottom-left"
  | "bottom-center"
  | "bottom-right";

interface Props {
  message: string;
  title?: string;
  type?: NotificationType;
  duration?: number;
  visible: boolean;
  position?: Position;
  showProgress?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  type: "info",
  duration: 3000,
  position: "bottom-right",
  showProgress: true,
});

const emit = defineEmits<{
  (e: "close"): void;
  (e: "update:visible", value: boolean): void;
  (e: "open"): void;
}>();

const show = ref(props.visible);
const showCopied = ref(false);
const progressPercent = ref(100);
let timer: number | null = null;
let copyTimer: number | null = null;
let progressTimer: number | null = null;
let startTime = 0;

const positionClass = computed(() => `sl-notify--${props.position}`);

function startTimer() {
  clearTimers();
  if (props.duration > 0) {
    startTime = Date.now();
    progressPercent.value = 100;

    timer = window.setTimeout(() => {
      handleClose();
    }, props.duration);

    if (props.showProgress) {
      const updateInterval = 50;
      progressTimer = window.setInterval(() => {
        const elapsed = Date.now() - startTime;
        progressPercent.value = Math.max(0, 100 - (elapsed / props.duration) * 100);
      }, updateInterval);
    }
  }
}

function clearTimers() {
  if (timer) {
    clearTimeout(timer);
    timer = null;
  }
  if (copyTimer) {
    clearTimeout(copyTimer);
    copyTimer = null;
  }
  if (progressTimer) {
    clearInterval(progressTimer);
    progressTimer = null;
  }
}

function handleClose() {
  show.value = false;
  emit("update:visible", false);
  emit("close");
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && show.value) {
    handleClose();
  }
}

async function copyToClipboard(text: string): Promise<boolean> {
  if (navigator.clipboard && window.isSecureContext) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      return fallbackCopy(text);
    }
  }
  return fallbackCopy(text);
}

function fallbackCopy(text: string): boolean {
  const textarea = document.createElement("textarea");
  textarea.value = text;
  textarea.style.position = "fixed";
  textarea.style.left = "-9999px";
  document.body.appendChild(textarea);
  textarea.select();
  try {
    document.execCommand("copy");
    return true;
  } catch {
    return false;
  } finally {
    document.body.removeChild(textarea);
  }
}

async function handleCopy() {
  if (showCopied.value) return;
  const success = await copyToClipboard(props.message);

  if (success) {
    showCopied.value = true;
    if (timer) clearTimeout(timer);

    copyTimer = window.setTimeout(() => {
      showCopied.value = false;
      startTimer();
    }, 1500);
  }
}

watch(
  () => props.visible,
  (val) => {
    show.value = val;
    if (val) {
      showCopied.value = false;
      progressPercent.value = 100;
      startTimer();
      emit("open");
    } else {
      clearTimers();
    }
  },
);

onMounted(() => {
  document.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
  clearTimers();
});
</script>

<template>
  <Teleport to="body">
    <Transition name="notify-slide">
      <div
        v-if="show"
        class="sl-notify"
        :class="[positionClass, `sl-notify--${type}`]"
        @click="handleCopy"
        role="alert"
        :aria-live="'polite'"
      >
        <div class="sl-notify__bg-circle"></div>

        <div class="sl-notify__content">
          <div class="sl-notify__icon">
            <Check v-if="type === 'success'" :size="24" />
            <XCircle v-else-if="type === 'error'" :size="24" />
            <AlertTriangle v-else-if="type === 'warning'" :size="24" />
            <Info v-else :size="24" />
          </div>
          <div class="sl-notify__text">
            <div v-if="title" class="sl-notify__title">{{ title }}</div>
            <div class="sl-notify__message">{{ message }}</div>
          </div>
          <button
            class="sl-notify__close"
            @click.stop="handleClose"
            :aria-label="i18n.t('common.close_notification')"
          >
            <X :size="18" />
          </button>
        </div>

        <Transition name="fade">
          <div v-if="showCopied" class="sl-notify__overlay">
            <Check :size="20" :stroke-width="3" />
            <span>已复制</span>
          </div>
        </Transition>

        <div
          v-if="showProgress && duration > 0"
          class="sl-notify__progress"
          :style="{ width: `${progressPercent}%` }"
        ></div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.sl-notify {
  position: fixed;
  width: 360px;
  max-width: 90vw;
  color: #fff;
  border-radius: 12px;
  box-shadow: 0 10px 30px -5px rgba(0, 0, 0, 0.4);
  cursor: pointer;
  overflow: hidden;
  z-index: 9999;
  font-family:
    system-ui,
    -apple-system,
    sans-serif;
  user-select: none;
  will-change: transform, opacity;
  transform: translateZ(0);
  backface-visibility: hidden;
}

/* Position variants */
.sl-notify--top-left {
  top: 24px;
  left: 24px;
}
.sl-notify--top-center {
  top: 24px;
  left: 50%;
  transform: translateX(-50%);
}
.sl-notify--top-right {
  top: 24px;
  right: 24px;
}
.sl-notify--bottom-left {
  bottom: 24px;
  left: 24px;
}
.sl-notify--bottom-center {
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
}
.sl-notify--bottom-right {
  bottom: 24px;
  right: 24px;
}

/* Color variants */
.sl-notify--success {
  background: linear-gradient(135deg, #059669 0%, #10b981 100%);
}
.sl-notify--error {
  background: linear-gradient(135deg, #dc2626 0%, #ef4444 100%);
}
.sl-notify--warning {
  background: linear-gradient(135deg, #d97706 0%, #f59e0b 100%);
}
.sl-notify--info {
  background: linear-gradient(135deg, #2563eb 0%, #3b82f6 100%);
}

.sl-notify__content {
  position: relative;
  display: flex;
  align-items: flex-start;
  padding: 16px;
  gap: 12px;
  z-index: 2;
}

.sl-notify__icon {
  flex-shrink: 0;
  margin-top: 2px;
}

.sl-notify__text {
  flex: 1;
  min-width: 0;
}

.sl-notify__title {
  font-size: 15px;
  font-weight: 700;
  margin-bottom: 4px;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.sl-notify__message {
  font-size: 14px;
  line-height: 1.5;
  font-weight: 500;
  word-break: break-word;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.sl-notify__close {
  background: rgba(255, 255, 255, 0.15);
  border: none;
  color: #fff;
  border-radius: 6px;
  padding: 4px;
  cursor: pointer;
  transition: all 0.2s;
  flex-shrink: 0;
  margin-left: 4px;
}
.sl-notify__close:hover {
  background: rgba(255, 255, 255, 0.3);
  transform: scale(1.1);
}

.sl-notify__bg-circle {
  position: absolute;
  top: -20px;
  right: -20px;
  width: 120px;
  height: 120px;
  background: radial-gradient(circle, rgba(255, 255, 255, 0.2) 0%, rgba(255, 255, 255, 0) 70%);
  border-radius: 50%;
  pointer-events: none;
  z-index: 1;
}

.sl-notify__overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.25);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-weight: bold;
  font-size: 16px;
  z-index: 10;
}

.sl-notify__progress {
  position: absolute;
  bottom: 0;
  left: 0;
  height: 3px;
  background: rgba(255, 255, 255, 0.6);
  transition: width 0.05s linear;
  z-index: 3;
}

/* Animations */
.notify-slide-enter-active {
  transition: all var(--sl-transition-slow) cubic-bezier(0.34, 1.56, 0.64, 1);
}
.notify-slide-leave-active {
  transition: all var(--sl-transition-normal) cubic-bezier(0.4, 0, 0.2, 1);
}

.sl-notify--top-left.notify-slide-enter-from,
.sl-notify--top-center.notify-slide-enter-from,
.sl-notify--top-right.notify-slide-enter-from {
  opacity: 0;
  transform: translateY(-20px) scale(0.9);
}

.sl-notify--bottom-left.notify-slide-enter-from,
.sl-notify--bottom-center.notify-slide-enter-from,
.sl-notify--bottom-right.notify-slide-enter-from {
  opacity: 0;
  transform: translateY(20px) scale(0.9);
}

.sl-notify--top-left.notify-slide-leave-to,
.sl-notify--top-center.notify-slide-leave-to,
.sl-notify--top-right.notify-slide-leave-to {
  opacity: 0;
  transform: translateY(-20px) scale(0.95);
}

.sl-notify--bottom-left.notify-slide-leave-to,
.sl-notify--bottom-center.notify-slide-leave-to,
.sl-notify--bottom-right.notify-slide-leave-to {
  opacity: 0;
  transform: translateY(20px) scale(0.95);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
