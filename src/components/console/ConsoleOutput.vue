<script setup lang="ts">
/**
 * ConsoleOutput.vue
 *
 * 控制台日志容器组件，替代原本对 cmz-console 的薄封装。
 * 职责：
 *   - 持有全量日志行（命令式 appendLines 注入，保持与原数据流兼容）
 *   - 根据 ConsoleView 传入的搜索/筛选状态派生 displayLines
 *   - 渲染 ConsoleLogList（虚拟滚动 + 高亮 + 导航）与 ConsoleInput（命令输入）
 *   - 修复 doScroll() 空实现与滚动事件缺失（Issue #479）
 */
import { ref, computed, watch } from "vue";
import { i18n } from "@language";
import ConsoleLogList from "./ConsoleLogList.vue";
import ConsoleInput from "./ConsoleInput.vue";
import type { ConsoleLineObj, LogFilterLevel, LogLineType } from "@type/console";

interface Props {
  consoleFontSize: number;
  consoleFontFamily: string;
  consoleLetterSpacing?: number;
  maxLogLines?: number;
  readonly?: boolean;
  history?: string[];
  completionMd?: string;
  // 搜索/筛选状态（由 ConsoleView 持有，便于 keep-alive 持久化）
  searchKeyword?: string;
  caseSensitive?: boolean;
  filterLevel?: LogFilterLevel;
  /** 搜索导航目标行（displayLines 中的索引，-1 表示不导航） */
  matchCursor?: number;
}

const props = withDefaults(defineProps<Props>(), {
  consoleLetterSpacing: 0,
  maxLogLines: 5000,
  readonly: false,
  history: () => [],
  completionMd: "",
  searchKeyword: "",
  caseSensitive: false,
  filterLevel: "all",
  matchCursor: -1,
});

const emit = defineEmits<{
  (e: "command", text: string): void;
  (e: "update:matchCount", count: number): void;
  (e: "scroll", userScrolledUp: boolean): void;
  (e: "scrollToBottom"): void;
}>();

// ============ 日志解析 ============
const LOG_REGEX = /^\[(\d{2}:\d{2}:\d{2})\] \[(.*?)\/(ERROR|INFO|WARN|DEBUG|FATAL)\]: (.*)$/;

const lines = ref<ConsoleLineObj[]>([]);

function levelToType(level: string): LogLineType {
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
  if (line.includes("[WARN]") || line.includes("WARNING")) return { text: line, type: "warning" };
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

// ============ 搜索/筛选派生 ============
const debouncedKeyword = ref("");
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  () => props.searchKeyword,
  (val) => {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      debouncedKeyword.value = val;
    }, 200);
  },
);

const displayLines = computed<ConsoleLineObj[]>(() => {
  let result = lines.value;

  // 级别筛选
  if (props.filterLevel !== "all") {
    const typeMap: Record<string, LogLineType | undefined> = {
      error: "error",
      warn: "warning",
      info: "info",
      debug: "info",
    };
    const target = typeMap[props.filterLevel];
    if (target) {
      result = result.filter((l) => l.type === target);
    }
  }

  // 关键词搜索（字面量匹配，转义正则元字符）
  const kw = debouncedKeyword.value.trim();
  if (kw) {
    const flags = props.caseSensitive ? "" : "i";
    try {
      const escaped = kw.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      const regex = new RegExp(escaped, flags);
      result = result.filter((l) => regex.test(l.text));
    } catch {
      // 非法模式时退化为不过滤
    }
  }

  return result;
});

// 向父组件上报当前匹配数量（用于 "3/15" 计数显示）
watch(
  () => displayLines.value.length,
  (count) => emit("update:matchCount", count),
  { immediate: true },
);

// ============ 渲染与滚动 ============
const logListRef = ref<InstanceType<typeof ConsoleLogList> | null>(null);

function doScroll(): void {
  logListRef.value?.scrollToBottom(false);
}

function handleScroll(userScrolledUp: boolean) {
  emit("scroll", userScrolledUp);
}

function handleScrollToBottom() {
  emit("scrollToBottom");
}

defineExpose({ doScroll, appendLines, clear, getAllPlainText });
</script>

<template>
  <div class="console-output" :style="{ letterSpacing: `${consoleLetterSpacing ?? 0}px` }">
    <ConsoleLogList
      ref="logListRef"
      :lines="displayLines"
      :keyword="debouncedKeyword"
      :case-sensitive="caseSensitive"
      :match-cursor="matchCursor"
      :font-size="consoleFontSize"
      :font-family="consoleFontFamily"
      :letter-spacing="consoleLetterSpacing"
      :placeholder="i18n.t('console.waiting_for_output')"
      @scroll="handleScroll"
      @scroll-to-bottom="handleScrollToBottom"
    />
    <ConsoleInput
      v-if="!readonly"
      :history="history"
      :completion-md="completionMd"
      @command="(text: string) => emit('command', text)"
    />
  </div>
</template>

<style scoped>
.console-output {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: var(--sl-console-bg, #0d1117);
}
</style>
