<script setup lang="ts">
/**
 * ConsoleInput.vue
 *
 * 命令输入框，替代 cmz-console 内置输入区。
 * 复用 cmzya-modern-ui 的 parseCompletionMd 实现 Minecraft 原版命令补全，
 * 支持：上下键历史导航、Tab/方向键补全循环、空格补全公共前缀、回车提交。
 */
import { ref, computed, nextTick } from "vue";
import { parseCompletionMd, type CompletionNode } from "cmzya-modern-ui";

interface Props {
  history?: string[];
  completionMd?: string;
  placeholder?: string;
  enableCompletion?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  history: () => [],
  completionMd: "",
  placeholder: "输入命令...",
  enableCompletion: true,
});

const emit = defineEmits<{
  (e: "command", text: string): void;
}>();

const inputValue = ref("");
const inputEl = ref<HTMLInputElement | null>(null);
const completionListEl = ref<HTMLElement | null>(null);

// ============ 历史导航 ============
let historyIndex = -1;
let historyBuffer = "";

function historyNav(dir: "up" | "down"): string | null {
  const h = props.history;
  if (h.length === 0) return null;
  if (dir === "up") {
    if (historyIndex === -1) {
      historyBuffer = inputValue.value;
      historyIndex = h.length - 1;
    } else if (historyIndex > 0) {
      historyIndex -= 1;
    } else {
      return h[0];
    }
    return h[historyIndex];
  } else {
    if (historyIndex === -1) return null;
    if (historyIndex >= h.length - 1) {
      historyIndex = -1;
      return historyBuffer;
    }
    historyIndex += 1;
    return h[historyIndex];
  }
}

function clearHistoryNav() {
  historyIndex = -1;
  historyBuffer = "";
}

// ============ 命令补全 ============
const completionTree = computed<CompletionNode[]>(() =>
  props.completionMd ? parseCompletionMd(props.completionMd) : [],
);

const completionOpen = ref(false);
const candidates = ref<CompletionNode[]>([]);
const activeIndex = ref(0);
let draft = ""; // 开启补全前的输入快照

function tokenize(cmd: string): { tokens: string[]; partial: string } {
  const lastSpace = cmd.lastIndexOf(" ");
  if (lastSpace === -1) return { tokens: [], partial: cmd };
  const tokens = cmd.slice(0, lastSpace).split(/\s+/).filter(Boolean);
  const partial = cmd.slice(lastSpace + 1);
  return { tokens, partial };
}

function resolveCandidates(cmd: string): {
  candidates: CompletionNode[];
  commonPrefix: string;
} {
  const { tokens, partial } = tokenize(cmd);
  let tree = completionTree.value;
  for (const t of tokens) {
    const node = tree.find((n) => n.label.toLowerCase() === t.toLowerCase());
    if (node && node.children) {
      tree = node.children;
    } else {
      return { candidates: [], commonPrefix: "" };
    }
  }
  const p = partial.toLowerCase();
  const matched = tree.filter((n) => (p ? n.label.toLowerCase().startsWith(p) : true));
  let commonPrefix = "";
  if (matched.length > 0) {
    const first = matched[0].label;
    let i = 0;
    for (; i < first.length; i++) {
      const ch = first[i];
      if (matched.every((c) => c.label[i] === ch)) commonPrefix += ch;
      else break;
    }
  }
  return { candidates: matched, commonPrefix };
}

function openCompletion() {
  if (!props.enableCompletion) {
    closeCompletion();
    return;
  }
  const { candidates: cands, commonPrefix } = resolveCandidates(inputValue.value);
  if (cands.length === 0) {
    closeCompletion();
    return;
  }
  candidates.value = cands;
  activeIndex.value = 0;
  draft = inputValue.value;
  completionOpen.value = true;
  // 若存在公共前缀则直接补全
  if (commonPrefix && commonPrefix !== tokenize(inputValue.value).partial) {
    applyFragment(commonPrefix);
  }
}

function closeCompletion() {
  completionOpen.value = false;
  candidates.value = [];
  activeIndex.value = 0;
  draft = "";
}

function applyFragment(fragment: string) {
  const { tokens } = tokenize(inputValue.value);
  inputValue.value = [...tokens, fragment].join(" ") + " ";
  nextTick(scrollActiveIntoView);
}

function applyActive() {
  const item = candidates.value[activeIndex.value];
  if (!item) return;
  const { tokens } = tokenize(draft || inputValue.value);
  inputValue.value = [...tokens, item.label].join(" ") + " ";
  nextTick(scrollActiveIntoView);
}

function cycle(dir: 1 | -1) {
  if (candidates.value.length === 0) return;
  const n = candidates.value.length;
  activeIndex.value = (activeIndex.value + dir + n) % n;
  applyActive();
}

function scrollActiveIntoView() {
  const list = completionListEl.value;
  if (!list) return;
  const items = list.querySelectorAll(".console-completion-item");
  const active = items[activeIndex.value] as HTMLElement | undefined;
  if (!active) return;
  const lr = list.getBoundingClientRect();
  const ar = active.getBoundingClientRect();
  if (ar.top < lr.top) list.scrollTop -= lr.top - ar.top;
  else if (ar.bottom > lr.bottom) list.scrollTop += ar.bottom - lr.bottom;
}

// ============ 提交 ============
function submit() {
  const cmd = inputValue.value.trim();
  clearHistoryNav();
  closeCompletion();
  if (!cmd) return;
  if (cmd.toLowerCase() === "whoami") {
    emit("command", cmd);
    inputValue.value = "";
    return;
  }
  emit("command", cmd);
  inputValue.value = "";
}

// ============ 事件处理 ============
function onKeydown(e: KeyboardEvent) {
  // 补全开启时优先处理方向键/Tab
  if (completionOpen.value) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      cycle(1);
      return;
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      cycle(-1);
      return;
    }
    if (e.key === "Tab" && !e.shiftKey) {
      e.preventDefault();
      cycle(1);
      return;
    }
    if (e.key === " ") {
      e.preventDefault();
      const { commonPrefix } = resolveCandidates(inputValue.value);
      if (commonPrefix) applyFragment(commonPrefix);
      closeCompletion();
      return;
    }
    if (e.key === "Escape") {
      e.preventDefault();
      inputValue.value = draft;
      closeCompletion();
      return;
    }
  }

  // 历史导航（补全未开启时）
  if (props.enableCompletion && !completionOpen.value) {
    if (e.key === "ArrowUp") {
      const h = historyNav("up");
      if (h !== null) {
        e.preventDefault();
        inputValue.value = h;
        nextTick(() => {
          const el = inputEl.value;
          if (el) el.selectionStart = el.selectionEnd = el.value.length;
        });
      }
      return;
    }
    if (e.key === "ArrowDown") {
      const h = historyNav("down");
      if (h !== null) {
        e.preventDefault();
        inputValue.value = h;
        nextTick(() => {
          const el = inputEl.value;
          if (el) el.selectionStart = el.selectionEnd = el.value.length;
        });
      }
      return;
    }
    if (e.key === "Tab" && !e.shiftKey) {
      e.preventDefault();
      openCompletion();
      return;
    }
  }

  if (e.key === "Enter") {
    e.preventDefault();
    submit();
  }
}

function onInput() {
  clearHistoryNav();
  if (props.enableCompletion) openCompletion();
}

function onFocus() {
  if (props.enableCompletion && inputValue.value.trim()) openCompletion();
}

function onClickOutside(e: FocusEvent) {
  const target = e.target as HTMLElement;
  if (!target.closest(".console-input-wrap") && !target.closest(".console-completion")) {
    closeCompletion();
  }
}

function onCompletionMousedown(item: CompletionNode, e: MouseEvent) {
  e.preventDefault();
  const idx = candidates.value.indexOf(item);
  if (idx >= 0) activeIndex.value = idx;
  applyActive();
}

function onCompletionMouseenter(idx: number) {
  activeIndex.value = idx;
}
</script>

<template>
  <div class="console-input-wrap">
    <div class="console-completion" v-if="completionOpen && candidates.length > 0">
      <div class="console-completion-list" ref="completionListEl">
        <div
          v-for="(item, idx) in candidates"
          :key="item.label + idx"
          class="console-completion-item"
          :class="{ 'console-completion-item--active': idx === activeIndex }"
          @mousedown="onCompletionMousedown(item, $event)"
          @mouseenter="onCompletionMouseenter(idx)"
        >
          <span class="console-completion-label">{{ item.label }}</span>
          <span v-if="item.desc" class="console-completion-desc">{{ item.desc }}</span>
          <span v-if="item.children && item.children.length > 0" class="console-completion-arrow">
            <svg
              width="10"
              height="10"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M9 18l6-6-6-6" />
            </svg>
          </span>
        </div>
      </div>
    </div>
    <div class="console-input-row">
      <span class="console-input-prompt">&gt;&gt;&gt;</span>
      <input
        ref="inputEl"
        v-model="inputValue"
        class="console-input"
        :placeholder="placeholder"
        spellcheck="false"
        autocomplete="off"
        @keydown="onKeydown"
        @input="onInput"
        @focus="onFocus"
        @blur="onClickOutside"
      />
    </div>
  </div>
</template>

<style scoped>
.console-input-wrap {
  position: relative;
  border-top: 1px solid var(--sl-border, #374151);
  background: var(--sl-surface, #161b22);
}

.console-input-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
}

.console-input-prompt {
  color: var(--sl-text-muted, #9ca3af);
  font-family: var(--sl-font-mono, monospace);
  font-size: 13px;
  user-select: none;
}

.console-input {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  color: var(--sl-text, #e5e7eb);
  font-family: var(--sl-font-mono, monospace);
  font-size: 13px;
  line-height: 1.5;
}

.console-input::placeholder {
  color: var(--sl-text-muted, #6b7280);
  opacity: 0.7;
}

.console-completion {
  position: absolute;
  bottom: 100%;
  left: 10px;
  right: 10px;
  max-height: 220px;
  overflow-y: auto;
  margin-bottom: 4px;
  background: var(--sl-surface, #1f2937);
  border: 1px solid var(--sl-border, #374151);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  z-index: 20;
}

.console-completion-list {
  padding: 4px;
}

.console-completion-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  font-family: var(--sl-font-mono, monospace);
}

.console-completion-item--active {
  background: var(--sl-primary-bg, rgba(59, 130, 246, 0.18));
  color: var(--sl-primary, #3b82f6);
}

.console-completion-label {
  font-weight: 500;
}

.console-completion-desc {
  color: var(--sl-text-muted, #6b7280);
  font-size: 12px;
  margin-left: auto;
}

.console-completion-arrow {
  color: var(--sl-text-muted, #6b7280);
  margin-left: auto;
  display: inline-flex;
}
</style>
