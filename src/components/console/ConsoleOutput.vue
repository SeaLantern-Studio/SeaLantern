<script setup lang="ts">
import { ref, computed } from "vue";
import { i18n } from "@language";

interface Props {
  consoleFontSize: number;
  consoleFontFamily: string;
  consoleLetterSpacing?: number;
  maxLogLines?: number;
  readonly?: boolean;
}

interface ConsoleLineObj {
  text: string;
  type?: "input" | "output" | "error" | "warning" | "info" | "success" | "system";
  timestamp?: string;
}

const props = withDefaults(defineProps<Props>(), {
  consoleLetterSpacing: 0,
  maxLogLines: 5000,
  readonly: false,
});

const emit = defineEmits<{
  (e: "command", text: string): void;
}>();

const LOG_REGEX = /^\[(\d{2}:\d{2}:\d{2})\] \[(.*?)\/(ERROR|INFO|WARN|DEBUG|FATAL)\]: (.*)$/;

const lines = ref<ConsoleLineObj[]>([]);

function levelToType(level: string): ConsoleLineObj["type"] {
  switch (level) {
    case "ERROR":
    case "FATAL":
      return "error";
    case "WARN":
      return "warning";
    case "DEBUG":
      return "info";
    case "INFO":
    default:
      return "info";
  }
}

function parseLine(line: string): ConsoleLineObj {
  const parsed = line.match(LOG_REGEX);
  if (parsed) {
    const [, time, , level] = parsed;
    return { text: line, type: levelToType(level), timestamp: time };
  }
  if (line.startsWith(">")) return { text: line, type: "input" };
  if (line.startsWith("[Sea Lantern]")) return { text: line, type: "system" };
  if (line.includes("[ERROR]") || line.includes("ERROR") || line.includes("[STDERR]"))
    return { text: line, type: "error" };
  if (line.includes("[WARN]") || line.includes("WARNING"))
    return { text: line, type: "warning" };
  return { text: line, type: "output" };
}

function appendLines(rawLines: string[]): void {
  if (rawLines.length === 0) return;
  const newLines = rawLines.map(parseLine);
  lines.value.push(...newLines);
  if (lines.value.length > props.maxLogLines) {
    lines.value.splice(0, lines.value.length - props.maxLogLines);
  }
}

function clear(): void {
  lines.value = [];
}

function getAllPlainText(): string {
  return lines.value.map((l) => l.text).join("\n");
}

function doScroll(): void {}

const consoleStyle = computed(() => ({
  "--cmz-font-mono": props.consoleFontFamily || "var(--sl-font-mono)",
  "--cmz-font-size-base": `${props.consoleFontSize}px`,
  letterSpacing: `${props.consoleLetterSpacing ?? 0}px`,
}));

defineExpose({ doScroll, appendLines, clear, getAllPlainText });
</script>

<template>
  <cmz-console
    :style="consoleStyle"
    :lines="lines"
    :show-timestamps="true"
    :auto-scroll="true"
    :max-lines="maxLogLines"
    :readonly="readonly"
    :placeholder="i18n.t('console.waiting_for_output')"
    height="100%"
    @command="(text: string) => emit('command', text)"
  />
</template>
