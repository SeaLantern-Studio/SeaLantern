<script setup lang="ts">
import { ref, watch, nextTick, shallowRef } from "vue";
import { i18n } from "@language";

interface Props {
  logs: string[];
  consoleFontSize: number;
  userScrolledUp: boolean;
}

interface ParsedLog {
  isParsed: boolean;
  time?: string;
  source?: string;
  level?: string;
  message?: string;
  raw: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "scroll", value: boolean): void;
  (e: "scrollToBottom"): void;
}>();

const logContainer = ref<HTMLElement | null>(null);

const LOG_REGEX = /^\[(\d{2}:\d{2}:\d{2})\] \[(.*?)\/(ERROR|INFO|WARN|DEBUG|FATAL)\]: (.*)$/;

function parseLogLine(line: string): ParsedLog {
  const match = line.match(LOG_REGEX);
  if (match) {
    const [, time, source, level, message] = match;
    return { isParsed: true, time, source, level, message, raw: line };
  }
  return { isParsed: false, raw: line };
}

const parsedLogs = shallowRef<ParsedLog[]>([]);
let lastParsedLength = 0;

watch(
  () => props.logs.length,
  (newLength) => {
    if (newLength === 0) {
      parsedLogs.value = [];
      lastParsedLength = 0;
      return;
    }
    if (newLength > lastParsedLength) {
      const newLogs = props.logs.slice(lastParsedLength);
      const newParsed = newLogs.map(parseLogLine);
      parsedLogs.value = [...parsedLogs.value, ...newParsed];
      lastParsedLength = newLength;
    } else if (newLength < lastParsedLength) {
      parsedLogs.value = props.logs.map(parseLogLine);
      lastParsedLength = newLength;
    }
  },
  { immediate: true },
);

watch(
  () => props.logs.length,
  () => {
    if (!props.userScrolledUp) doScroll();
  },
);

function doScroll() {
  nextTick(() => {
    if (logContainer.value) logContainer.value.scrollTop = logContainer.value.scrollHeight;
  });
}

function handleScroll() {
  if (!logContainer.value) return;
  const el = logContainer.value;
  emit("scroll", el.scrollHeight - el.scrollTop - el.clientHeight > 40);
}

defineExpose({ doScroll });
</script>

<template>
  <div
    class="console-output"
    ref="logContainer"
    @scroll="handleScroll"
    :style="{ fontSize: consoleFontSize + 'px' }"
  >
    <div
      v-for="(line, i) in logs"
      :key="i"
      class="log-line"
      :class="{
        'log-error':
          line.includes('[ERROR]') || line.includes('ERROR') || line.includes('[STDERR]'),
        'log-warn': line.includes('[WARN]') || line.includes('WARNING'),
        'log-command': line.startsWith('>'),
        'log-system': line.startsWith('[Sea Lantern]'),
      }"
    >
      <!-- 解析日志行，提取时间和等级 -->
      <template v-if="parsedLogs[i].isParsed">
        <span class="log-time">[{{ parsedLogs[i].time }}]</span>
        <span class="log-level" :class="'level-' + parsedLogs[i].level?.toLowerCase()"
          >[{{ parsedLogs[i].source }}/{{ parsedLogs[i].level }}]</span
        >
        <span class="log-content">{{ parsedLogs[i].message }}</span>
      </template>
      <!-- 对于不符合标准格式的日志行，直接显示 -->
      <template v-else>
        {{ line }}
      </template>
    </div>
    <div v-if="logs.length === 0" class="log-empty">
      {{ i18n.t("console.waiting_for_output") }}
    </div>
  </div>
  <div v-if="userScrolledUp" class="scroll-btn" @click="emit('scrollToBottom')">
    {{ i18n.t("console.back_to_bottom") }}
  </div>
</template>

<style scoped>
.console-output {
  flex: 1;
  background: var(--sl-bg-secondary);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-md);
  overflow-y: auto;
  font-family: var(--sl-font-mono);
  line-height: 1.7;
  color: var(--sl-text-primary);
  min-height: 0;
  user-select: text;
  -webkit-user-select: text;
  cursor: text;
}
.log-line {
  white-space: pre-wrap;
  word-break: break-all;
}
.log-error {
  color: var(--sl-error);
  font-weight: 500;
}
.log-warn {
  color: var(--sl-warning);
  font-weight: 500;
}
.log-command {
  color: var(--sl-info);
  font-weight: 600;
}
.log-system {
  color: var(--sl-success);
  font-style: italic;
}
.log-empty {
  color: var(--sl-text-tertiary);
  font-style: italic;
}

/* 日志时间和等级样式 */
.log-time {
  color: var(--sl-text-tertiary);
  margin-right: 8px;
}

.log-level {
  margin-right: 8px;
  font-weight: 500;
}

.log-level.level-error {
  color: var(--sl-error);
}

.log-level.level-info {
  color: var(--sl-success);
}

.log-level.level-warn {
  color: var(--sl-warning);
}

.log-level.level-debug {
  color: var(--sl-info);
}

.log-level.level-fatal {
  color: var(--sl-error);
  font-weight: 700;
}

.log-content {
  color: var(--sl-text-primary);
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
