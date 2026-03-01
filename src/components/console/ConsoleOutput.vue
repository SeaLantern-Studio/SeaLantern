<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick } from "vue";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { ClipboardAddon } from "@xterm/addon-clipboard";
import { SerializeAddon } from "@xterm/addon-serialize";
import "@xterm/xterm/css/xterm.css";
import { i18n } from "@language";

interface Props {
  consoleFontSize: number;
  consoleFontFamily: string;
  consoleLetterSpacing?: number;
  userScrolledUp: boolean;
  maxLogLines?: number;
}

const props = withDefaults(defineProps<Props>(), {
  consoleLetterSpacing: 0,
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
let clipboardAddon: ClipboardAddon | null = null;
let serializeAddon: SerializeAddon | null = null;
let resizeObserver: ResizeObserver | null = null;
let scrollDisposable: { dispose: () => void } | null = null;
let hasAnyLine = false;
let terminalTextarea: HTMLTextAreaElement | null = null;

async function handleCopyShortcut(event: KeyboardEvent) {
  if (!(event.ctrlKey || event.metaKey) || event.key.toLowerCase() !== "c") return;
  if (!terminal?.hasSelection()) return;

  const selectedText = terminal.getSelection();
  if (!selectedText) return;

  event.preventDefault();
  const copied = typeof document.execCommand === "function" && document.execCommand("copy");
  if (copied) {
    terminal.clearSelection();
    return;
  }

  try {
    await navigator.clipboard.writeText(selectedText);
    terminal.clearSelection();
  } catch (_err) {
    // Keep selection when clipboard write fails.
  }
}

function handleCopyEvent(event: ClipboardEvent) {
  if (!terminal?.hasSelection()) return;
  const selectedText = terminal.getSelection();
  if (!selectedText || !event.clipboardData) return;
  event.preventDefault();
  event.clipboardData.setData("text/plain", selectedText);
}

function keepDisplayOnlyFocus() {
  terminal?.blur();
}

function cssVar(name: string, fallback: string): string {
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return value || fallback;
}

function getConsoleFontFamily(fontFamily: string): string {
  return fontFamily && fontFamily.trim().length > 0
    ? fontFamily
    : cssVar("--sl-font-mono", "monospace");
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
  terminal.write(`\x1b[2m${i18n.t("console.waiting_for_output")}\x1b[0m`);
}

function appendLines(lines: string[]) {
  if (!terminal) return;
  if (lines.length === 0) return;

  let isFirstLineInBuffer = !hasAnyLine;
  if (!hasAnyLine) {
    terminal.clear();
    terminal.reset();
    hasAnyLine = true;
  }

  for (const line of lines) {
    const formattedLine = formatLine(line);
    if (isFirstLineInBuffer) {
      terminal.write(formattedLine);
      isFirstLineInBuffer = false;
    } else {
      terminal.write(`\r\n${formattedLine}`);
    }
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
  if (!serializeAddon || !hasAnyLine) return "";
  const serialized = serializeAddon.serialize({
    excludeAltBuffer: true,
    excludeModes: true,
  });
  return stripAnsi(serialized).replaceAll("\r", "");
}

function stripAnsi(text: string): string {
  let result = "";
  let i = 0;
  while (i < text.length) {
    if (text.charCodeAt(i) === 27 && text.charCodeAt(i + 1) === 91) {
      i += 2;
      while (i < text.length) {
        const code = text.charCodeAt(i);
        if (code >= 64 && code <= 126) {
          i += 1;
          break;
        }
        i += 1;
      }
      continue;
    }
    result += text[i];
    i += 1;
  }
  return result;
}

function setupScrollTracking() {
  if (!terminal) return;
  scrollDisposable = terminal.onScroll(() => {
    const buffer = terminal?.buffer.active;
    if (!buffer) return;
    emit("scroll", buffer.baseY - buffer.viewportY > 0);
  });
}

function doScroll() {
  nextTick(() => {
    terminal?.scrollToBottom();
    emit("scroll", false);
  });
}

onMounted(() => {
  if (!terminalHost.value || terminal) return;

  terminal = new Terminal({
    convertEol: true,
    allowTransparency: false,
    disableStdin: true,
    cursorBlink: false,
    cursorInactiveStyle: "none",
    fontFamily: getConsoleFontFamily(props.consoleFontFamily),
    fontSize: props.consoleFontSize,
    letterSpacing: props.consoleLetterSpacing,
    lineHeight: 1,
    scrollback: Math.max(100, props.maxLogLines),
    theme: {
      background: cssVar("--sl-bg-secondary", "#111827"),
      foreground: cssVar("--sl-text-primary", "#e5e7eb"),
      cursor: "transparent",
      cursorAccent: "transparent",
      selectionBackground: cssVar("--sl-primary-bg", "#1e3a8a"),
    },
  });

  fitAddon = new FitAddon();
  clipboardAddon = new ClipboardAddon();
  serializeAddon = new SerializeAddon();
  terminal.loadAddon(fitAddon);
  terminal.loadAddon(clipboardAddon);
  terminal.loadAddon(serializeAddon);
  terminal.open(terminalHost.value);
  terminalTextarea = terminal.textarea;
  if (terminalTextarea) {
    terminalTextarea.tabIndex = -1;
    terminalTextarea.readOnly = true;
    terminalTextarea.addEventListener("focus", keepDisplayOnlyFocus);
  }
  terminalHost.value.addEventListener("mousedown", keepDisplayOnlyFocus);
  fitTerminal();
  setupScrollTracking();
  clear();

  resizeObserver = new ResizeObserver(() => {
    fitTerminal();
  });
  resizeObserver.observe(terminalHost.value);

  window.addEventListener("resize", fitTerminal);
  window.addEventListener("keydown", handleCopyShortcut);
  document.addEventListener("copy", handleCopyEvent, true);
  keepDisplayOnlyFocus();
});

onUnmounted(() => {
  window.removeEventListener("resize", fitTerminal);
  window.removeEventListener("keydown", handleCopyShortcut);
  document.removeEventListener("copy", handleCopyEvent, true);
  terminalHost.value?.removeEventListener("mousedown", keepDisplayOnlyFocus);
  terminalTextarea?.removeEventListener("focus", keepDisplayOnlyFocus);
  terminalTextarea = null;
  resizeObserver?.disconnect();
  resizeObserver = null;
  scrollDisposable?.dispose();
  scrollDisposable = null;
  fitAddon = null;
  clipboardAddon = null;
  serializeAddon = null;
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
  () => props.consoleFontFamily,
  (family) => {
    if (!terminal) return;
    terminal.options.fontFamily = getConsoleFontFamily(family);
    terminal.clearTextureAtlas();
    terminal.refresh(0, terminal.rows - 1);
    fitTerminal();
  },
);

watch(
  () => props.consoleLetterSpacing,
  (value) => {
    if (!terminal) return;
    terminal.options.letterSpacing = value;
    terminal.clearTextureAtlas();
    terminal.refresh(0, terminal.rows - 1);
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
