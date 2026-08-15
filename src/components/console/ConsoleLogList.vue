<script setup lang="ts">
/**
 * ConsoleLogList.vue
 *
 * 自定义虚拟滚动日志列表，替代 cmz-console 的渲染层。
 * - 使用 @tanstack/vue-virtual 实现虚拟滚动（仅渲染可视区域行，支撑大数据量）
 * - 复用 cmz-console 的段解析逻辑实现日志级别着色
 * - 支持关键词高亮（搜索时分段渲染匹配词）
 * - 支持编程式滚动到指定行（搜索结果导航）
 * - 用户向上滚动时暂停自动滚动，提供"回到底部"能力（修复 #479）
 */
import {
  ref,
  computed,
  watch,
  nextTick,
  onMounted,
  onUnmounted,
  type ComponentPublicInstance,
} from "vue";
import { useVirtualizer } from "@tanstack/vue-virtual";
import type { ConsoleLineObj, LogSegmentBracket } from "@type/console";

interface Props {
  /** 已过滤后的日志行（筛选在 ConsoleOutput 中完成） */
  lines: ConsoleLineObj[];
  /** 搜索关键词（空 = 不高亮） */
  keyword: string;
  /** 是否大小写敏感 */
  caseSensitive: boolean;
  /** 当前导航目标行索引（lines 中的索引，-1 表示不导航） */
  matchCursor: number;
  /** 字体大小（px） */
  fontSize: number;
  /** 字体族 */
  fontFamily: string;
  /** 字间距（px） */
  letterSpacing: number;
  /** 空状态占位文本 */
  placeholder: string;
}

const props = withDefaults(defineProps<Props>(), {
  keyword: "",
  caseSensitive: false,
  matchCursor: -1,
  fontSize: 13,
  fontFamily: "",
  letterSpacing: 0,
  placeholder: "",
});

const emit = defineEmits<{
  (e: "scroll", userScrolledUp: boolean): void;
  (e: "scrollToBottom"): void;
}>();

// ============ 段解析（复用 cmz-console 的 B() 逻辑） ============
const LEVEL_PATTERN =
  /\b(INFO|SUCCESS|ERROR|WARN|WARNING|DEBUG|FATAL|CRITICAL|TRACE|VERBOSE|NOTICE|EMERG|ALERT)\b/i;
const TIME_PATTERN = /\d{2}:\d{2}:\d{2}/;
// ANSI 转义符 (ESC, \x1b)：用 fromCharCode 构造，避免源码中出现控制字符转义触发 lint
const ESC = String.fromCharCode(27);
const ANSI_PATTERN = new RegExp(`(?:${ESC}\\[|\\[)[0-9;]*m`, "g");
const LEVEL_MAP: Record<string, string> = {
  INFO: "info",
  SUCCESS: "success",
  ERROR: "error",
  FATAL: "error",
  CRITICAL: "error",
  EMERG: "error",
  ALERT: "error",
  WARN: "warn",
  WARNING: "warn",
  DEBUG: "debug",
  TRACE: "debug",
  VERBOSE: "debug",
  NOTICE: "notice",
};

interface RawSegment {
  text: string;
  bracket: LogSegmentBracket;
  levelType?: string;
}

function parseSegments(line: ConsoleLineObj): RawSegment[] {
  const segments: RawSegment[] = [];
  const text = line.text.replace(ANSI_PATTERN, "").trimStart();
  let i = 0;
  while (i < text.length) {
    const ws = text.slice(i).match(/^\s+/);
    if (ws) {
      i += ws[0].length;
      continue;
    }
    if (text[i] === "[") {
      const closeIdx = text.indexOf("]", i);
      if (closeIdx !== -1) {
        const bracket = text.slice(i, closeIdx + 1);
        const levelMatch = bracket.match(LEVEL_PATTERN);
        const isTime = TIME_PATTERN.test(bracket);
        if (levelMatch) {
          const lvl = levelMatch[1].toUpperCase();
          segments.push({ text: bracket, bracket: "level", levelType: LEVEL_MAP[lvl] || "info" });
        } else if (isTime) {
          segments.push({ text: bracket, bracket: "time" });
        } else {
          segments.push({ text: bracket, bracket: "meta" });
        }
        i = closeIdx + 1;
        continue;
      }
    }
    segments.push({ text: text.slice(i), bracket: null });
    break;
  }
  return segments;
}

interface RenderToken {
  text: string;
  bracket: LogSegmentBracket;
  levelType?: string;
  highlight: boolean;
}

const keywordCache = new Map<
  string,
  { kw: string; caseSensitive: boolean; regex: RegExp | null }
>();

function getKeywordRegex(kw: string, caseSensitive: boolean): RegExp | null {
  const trimmed = kw.trim();
  if (!trimmed) return null;
  const cacheKey = `${trimmed}__${caseSensitive}`;
  const cached = keywordCache.get(cacheKey);
  if (cached) return cached.regex;
  const escaped = trimmed.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const regex = new RegExp(`(${escaped})`, caseSensitive ? "g" : "gi");
  keywordCache.set(cacheKey, { kw: trimmed, caseSensitive, regex });
  return regex;
}

function buildTokens(line: ConsoleLineObj): RenderToken[] {
  const segments = parseSegments(line);
  const regex = getKeywordRegex(props.keyword, props.caseSensitive);
  if (!regex) {
    return segments.map((s) => ({ ...s, highlight: false }));
  }
  const tokens: RenderToken[] = [];
  for (const seg of segments) {
    const parts = seg.text.split(regex);
    parts.forEach((part, idx) => {
      if (part === "") return;
      // split 带捕获组时，奇数索引为匹配到的关键词
      tokens.push({
        text: part,
        bracket: seg.bracket,
        levelType: seg.levelType,
        highlight: idx % 2 === 1,
      });
    });
  }
  return tokens;
}

// ============ 虚拟滚动 ============
const parentRef = ref<HTMLElement | null>(null);
const isNearBottom = ref(true);
const userScrolledUp = ref(false);

const virtualizerOptions = computed(() => ({
  count: props.lines.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => Math.round(props.fontSize * 1.6),
  overscan: 20,
}));

const virtualizer = useVirtualizer(virtualizerOptions);

function scrollToBottom(smooth = false) {
  const el = parentRef.value;
  if (!el) return;
  el.scrollTo({ top: el.scrollHeight, behavior: smooth ? "smooth" : "auto" });
}

function measureEl(el: Element | ComponentPublicInstance | null) {
  if (el instanceof Element) virtualizer.value.measureElement(el);
}

function onScroll() {
  const el = parentRef.value;
  if (!el) return;
  const threshold = 30;
  const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < threshold;
  isNearBottom.value = nearBottom;
  userScrolledUp.value = !nearBottom;
  emit("scroll", userScrolledUp.value);
}

// 新行追加后，若用户处于底部则自动滚动到底部
watch(
  () => props.lines.length,
  () => {
    if (isNearBottom.value) {
      nextTick(() => scrollToBottom(false));
    }
  },
);

// 搜索导航：滚动到指定行
watch(
  () => props.matchCursor,
  (idx) => {
    if (idx == null || idx < 0) return;
    nextTick(() => {
      if (idx >= props.lines.length) return;
      virtualizer.value.scrollToIndex(idx, { align: "center" });
    });
  },
);

function handleBackToBottom() {
  scrollToBottom(true);
  userScrolledUp.value = false;
  emit("scroll", false);
  emit("scrollToBottom");
}

defineExpose({ scrollToBottom, isNearBottom });

onMounted(() => {
  nextTick(() => scrollToBottom(false));
});

onUnmounted(() => {
  // nothing
});
</script>

<template>
  <div class="console-log-list">
    <div
      ref="parentRef"
      class="console-log-body"
      :style="{
        fontFamily: fontFamily || 'var(--sl-font-mono)',
        fontSize: fontSize + 'px',
        letterSpacing: letterSpacing + 'px',
      }"
      @scroll="onScroll"
    >
      <div v-if="lines.length === 0 && placeholder" class="console-log-placeholder">
        {{ placeholder }}
      </div>
      <div class="console-log-viewport" :style="{ height: virtualizer.getTotalSize() + 'px' }">
        <div
          v-for="item in virtualizer.getVirtualItems()"
          :key="item.index"
          class="console-log-line"
          :class="[`console-log-line--${lines[item.index]?.type || 'output'}`]"
          :data-index="item.index"
          :ref="measureEl"
          :style="{
            transform: `translateY(${item.start}px)`,
          }"
        >
          <template v-for="(token, ti) in buildTokens(lines[item.index])" :key="ti">
            <span
              v-if="token.bracket === 'level'"
              class="log-token log-token--level"
              :class="`log-token--level-${token.levelType}`"
              >{{ token.text.slice(1, -1) }}</span
            >
            <span v-else-if="token.bracket === 'time'" class="log-token log-token--time">{{
              token.text.slice(1, -1)
            }}</span>
            <span v-else-if="token.bracket === 'meta'" class="log-token log-token--meta">{{
              token.text.slice(1, -1)
            }}</span>
            <span v-else class="log-token" :class="{ 'log-highlight': token.highlight }">{{
              token.text
            }}</span>
          </template>
        </div>
      </div>
    </div>
    <div
      v-if="!isNearBottom && lines.length > 0"
      class="console-back-to-bottom"
      title="滚动到底部"
      @click="handleBackToBottom"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M6 9l6 6 6-6" />
      </svg>
      <span>回到底部</span>
    </div>
  </div>
</template>

<style scoped>
.console-log-list {
  position: relative;
  height: 100%;
  width: 100%;
  display: flex;
  flex-direction: column;
}

.console-log-body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 4px 10px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
  background: var(--sl-console-bg, #0d1117);
  color: var(--sl-console-fg, #c9d1d9);
  font-family: var(--sl-font-mono, "JetBrains Mono", "Fira Code", monospace);
}

.console-log-viewport {
  width: 100%;
  position: relative;
}

.console-log-line {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  padding: 0;
  min-height: 1.6em;
  box-sizing: border-box;
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
}

.console-log-placeholder {
  padding: 12px 4px;
  color: var(--sl-text-muted, #6b7280);
  font-style: italic;
}

.log-token {
  white-space: pre-wrap;
  word-break: break-word;
}

.log-token--level {
  font-weight: 600;
}
.log-token--level-error {
  color: var(--sl-error, #ef4444);
}
.log-token--level-warn {
  color: var(--sl-warning, #f59e0b);
}
.log-token--level-info {
  color: var(--sl-info, #3b82f6);
}
.log-token--level-success {
  color: var(--sl-success, #22c55e);
}
.log-token--level-notice {
  color: var(--sl-info, #3b82f6);
}
.log-token--level-debug {
  color: #c084fc;
}
.log-token--time {
  color: var(--sl-text-muted, #6b7280);
  opacity: 0.8;
}
.log-token--meta {
  color: var(--sl-text-muted, #6b7280);
  opacity: 0.8;
}

/* 行级着色（与 cmz-console 行为一致） */
.console-log-line--error .log-token:not(.log-token--level) {
  color: var(--sl-error, #ef4444);
}
.console-log-line--warning .log-token:not(.log-token--level) {
  color: var(--sl-warning, #f59e0b);
}
.console-log-line--system {
  color: var(--sl-info, #3b82f6);
}
.console-log-line--input {
  color: var(--sl-text-muted, #9ca3af);
}

/* 关键词高亮 */
.log-highlight {
  background: var(--sl-highlight-bg, #facc15);
  color: var(--sl-highlight-fg, #1f2937);
  border-radius: 2px;
  padding: 0 1px;
  font-weight: 600;
}

.console-back-to-bottom {
  position: absolute;
  bottom: 12px;
  right: 16px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  font-size: 12px;
  border-radius: 999px;
  background: var(--sl-surface, #1f2937);
  color: var(--sl-text, #e5e7eb);
  border: 1px solid var(--sl-border, #374151);
  cursor: pointer;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  user-select: none;
}
.console-back-to-bottom:hover {
  background: var(--sl-surface-hover, #374151);
}
</style>
