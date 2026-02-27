<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick } from "vue";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { i18n } from "@language";

interface Props {
  consoleFontSize: number;
  userScrolledUp: boolean;
  maxLogLines: number;
}

const props = withDefaults(defineProps<Props>(), {
  maxLogLines: 5000,
});

const emit = defineEmits<{
  (e: "scroll", value: boolean): void;
  (e: "scrollToBottom"): void;
}>();

const terminalHost = ref<HTMLDivElement | null>(null);

const LOG_REGEX = /^\[(\d{2}:\d{2}:\d{2})\] \[(.*?)\/(ERROR|INFO|WARN|DEBUG|FATAL)\]: (.*)$/;

let terminal: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let viewportEl: HTMLElement | null = null;
let resizeObserver: ResizeObserver | null = null;
let removeViewportScrollListener: (() => void) | null = null;
let hasAnyLine = false;

function cssVar(name: string, fallback: string): string {
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return value || fallback;
}

function getLevelColor(level: string): string {
  switch (level) {
    case "ERROR":
    case "FATAL":
      return "31";
    case "WARN":
      return "33";
    case "DEBUG":
      return "36";
    case "INFO":
    default:
      return "32";
  }
}

function formatLine(line: string): string {
  const parsed = line.match(LOG_REGEX);
  if (parsed) {
    const [, time, source, level, message] = parsed;
    const levelColor = getLevelColor(level);
    return `\x1b[90m[${time}]\x1b[0m \x1b[${levelColor}m[${source}/${level}]\x1b[0m ${message}`;
  }

  if (line.startsWith(">")) {
    return `\x1b[36;1m${line}\x1b[0m`;
  }
  if (line.startsWith("[Sea Lantern]")) {
    return `\x1b[32;3m${line}\x1b[0m`;
  }
  if (line.includes("[ERROR]") || line.includes("ERROR") || line.includes("[STDERR]")) {
    return `\x1b[31m${line}\x1b[0m`;
  }
  if (line.includes("[WARN]") || line.includes("WARNING")) {
    return `\x1b[33m${line}\x1b[0m`;
  }
  return line;
}

function fitTerminal() {
  fitAddon?.fit();
}

function renderEmptyState() {
  if (!terminal) return;
  terminal.writeln(`\x1b[2m${i18n.t("console.waiting_for_output")}\x1b[0m`);
}

function appendLines(lines: string[]) {
  if (!terminal) return;
  if (lines.length === 0) return;

  if (!hasAnyLine) {
    terminal.clear();
    terminal.reset();
    hasAnyLine = true;
  }

  for (const line of lines) {
    terminal.writeln(formatLine(line));
  }

  if (!props.userScrolledUp) {
    doScroll();
  }
}

function clear() {
  if (!terminal) return;
  terminal.clear();
  terminal.reset();
  hasAnyLine = false;
  renderEmptyState();
  emit("scroll", false);
}

function getAllPlainText(): string {
  if (!terminal || !hasAnyLine) return "";

  const buffer = terminal.buffer.active;
  const lines: string[] = [];
  for (let i = 0; i < buffer.length; i++) {
    const line = buffer.getLine(i);
    if (!line) continue;
    lines.push(line.translateToString(true));
  }
  return lines.join("\n");
}

function setupViewportScrollTracking() {
  if (!terminalHost.value) return;

  const viewport = terminalHost.value.querySelector(".xterm-viewport");
  if (!(viewport instanceof HTMLElement)) return;

  viewportEl = viewport;

  const onScroll = () => {
    if (!viewportEl) return;
    const delta = viewportEl.scrollHeight - viewportEl.scrollTop - viewportEl.clientHeight;
    emit("scroll", delta > 40);
  };

  viewportEl.addEventListener("scroll", onScroll);
  removeViewportScrollListener = () => {
    viewportEl?.removeEventListener("scroll", onScroll);
  };
}

function doScroll() {
  nextTick(() => {
    terminal?.scrollToBottom();
    if (viewportEl) {
      viewportEl.scrollTop = viewportEl.scrollHeight;
    }
    emit("scroll", false);
  });
}

onMounted(() => {
  if (!terminalHost.value || terminal) return;

  terminal = new Terminal({
    convertEol: true,
    allowTransparency: false,
    disableStdin: true,
    fontFamily: cssVar("--sl-font-mono", "monospace"),
    fontSize: props.consoleFontSize,
    lineHeight: 1,
    scrollback: Math.max(100, props.maxLogLines),
    theme: {
      background: cssVar("--sl-bg-secondary", "#111827"),
      foreground: cssVar("--sl-text-primary", "#e5e7eb"),
      cursor: cssVar("--sl-primary", "#3b82f6"),
      selectionBackground: cssVar("--sl-primary-bg", "#1e3a8a"),
    },
  });

  fitAddon = new FitAddon();
  terminal.loadAddon(fitAddon);
  terminal.open(terminalHost.value);
  terminal.attachCustomKeyEventHandler((event: KeyboardEvent) => {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "c") {
      const selectedText = terminal?.getSelection() || "";
      if (selectedText.length > 0) {
        void navigator.clipboard.writeText(selectedText);
        return false;
      }
    }
    return true;
  });
  fitTerminal();
  setupViewportScrollTracking();
  clear();

  resizeObserver = new ResizeObserver(() => {
    fitTerminal();
  });
  resizeObserver.observe(terminalHost.value);

  window.addEventListener("resize", fitTerminal);
});

onUnmounted(() => {
  window.removeEventListener("resize", fitTerminal);
  resizeObserver?.disconnect();
  resizeObserver = null;
  removeViewportScrollListener?.();
  removeViewportScrollListener = null;
  viewportEl = null;
  fitAddon = null;
  terminal?.dispose();
  terminal = null;
  hasAnyLine = false;
});

watch(
  () => props.consoleFontSize,
  (size) => {
    if (!terminal) return;
    terminal.options.fontSize = size;
    fitTerminal();
  },
);

watch(
  () => props.maxLogLines,
  (value) => {
    if (!terminal) return;
    terminal.options.scrollback = Math.max(100, value || 5000);
  },
);

watch(
  () => props.userScrolledUp,
  (value) => {
    if (!value) doScroll();
  },
);

defineExpose({ doScroll, appendLines, clear, getAllPlainText });
</script>

<template>
  <div class="console-output">
    <div ref="terminalHost" class="terminal-host"></div>
  </div>
  <div v-if="userScrolledUp" class="scroll-btn" @click="emit('scrollToBottom')">
    {{ i18n.t("console.back_to_bottom") }}
  </div>
</template>

<style scoped>
.console-output {
  flex: 1;
  display: flex;
  background: var(--sl-bg-secondary);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  min-height: 0;
  overflow: hidden;
  padding: var(--sl-space-md);
  user-select: text;
  -webkit-user-select: text;
  cursor: text;
}

.terminal-host {
  flex: 1;
  min-height: 0;
  height: 100%;
  width: 100%;
}

.terminal-host :deep(.xterm) {
  height: 100%;
  color: var(--sl-text-primary);
  font-family: var(--sl-font-mono);
}

.terminal-host :deep(.xterm-viewport) {
  overflow-y: auto !important;
  background: var(--sl-bg-secondary);
}

.scroll-btn {
  position: absolute;
  bottom: 70px;
  left: 50%;
  transform: translateX(-50%);
  padding: 6px 16px;
  background: var(--sl-primary);
  color: white;
  border-radius: var(--sl-radius-full);
  font-size: 0.75rem;
  cursor: pointer;
  box-shadow: var(--sl-shadow-md);
  z-index: 10;
}
</style>
