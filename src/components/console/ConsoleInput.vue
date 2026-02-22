<script setup lang="ts">
import { ref, computed, nextTick } from "vue";
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";

interface Props {
  consoleFontSize: number;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: "sendCommand", cmd: string): void;
}>();

const commandInput = ref("");
const inputRef = ref<HTMLInputElement | null>(null);
const suggestionsRef = ref<HTMLDivElement | null>(null);
const showSuggestions = ref(false);
const suggestionIndex = ref(0);
const lastTabOriginalWord = ref("");
const lastTabWordIndex = ref(-1);
const tabCycleIndex = ref(0);
let isCompleting = false;

// 命令树结构：按词层级组织
const commandTree: Record<string, string[]> = {
  help: [],
  list: [],
  stop: [],
  say: [],
  give: [],
  tp: [],
  teleport: [],
  kill: [],
  kick: [],
  ban: [],
  pardon: [],
  op: [],
  deop: [],
  tps: [],
  plugins: [],
  version: [],
  "save-all": [],
  time: ["set"],
  weather: ["clear", "rain", "thunder"],
  gamemode: ["survival", "creative", "adventure", "spectator"],
  difficulty: ["peaceful", "easy", "normal", "hard"],
  whitelist: ["add", "remove", "list"],
  gamerule: ["keepInventory", "doDaylightCycle", "mobGriefing"],
};

// gamerule 的值选项
const gameruleValues: Record<string, string[]> = {
  keepInventory: ["true", "false"],
  doDaylightCycle: ["true", "false"],
  mobGriefing: ["true", "false"],
};

// time set 的值
const timeValues = ["day", "night", "noon"];

// 获取当前光标所在词的位置
function getCurrentWordInfo(
  input: string,
  cursorPos: number,
): { word: string; startIndex: number; wordIndex: number } {
  const words = input.split(" ");
  let currentPos = 0;
  let wordIndex = 0;

  for (let i = 0; i < words.length; i++) {
    const wordEnd = currentPos + words[i].length;
    if (cursorPos <= wordEnd || (i === words.length - 1 && cursorPos <= wordEnd + 1)) {
      wordIndex = i;
      break;
    }
    currentPos = wordEnd + 1;
  }

  const words2 = input.split(" ");
  let startIndex = 0;
  for (let i = 0; i < wordIndex; i++) {
    startIndex += words2[i].length + 1;
  }

  return {
    word: words2[wordIndex] || "",
    startIndex,
    wordIndex,
  };
}

// 获取当前词的补全选项
function getCompletions(input: string, wordIndex: number, currentWord: string): string[] {
  const words = input.trim().split(/\s+/);
  const lowerWord = currentWord.toLowerCase();

  if (wordIndex === 0) {
    // 第一级：匹配命令名
    // 如果没有输入，返回所有命令
    if (!currentWord) {
      return Object.keys(commandTree).sort();
    }
    return Object.keys(commandTree)
      .filter((cmd) => cmd.toLowerCase().startsWith(lowerWord))
      .sort();
  }

  const cmd = words[0]?.toLowerCase();

  if (wordIndex === 1) {
    // 第二级：命令的子命令
    if (cmd === "time") {
      return ["set"].filter((s) => s.startsWith(lowerWord));
    }
    if (commandTree[cmd]) {
      return commandTree[cmd].filter((s) => s.toLowerCase().startsWith(lowerWord));
    }
  }

  if (wordIndex === 2) {
    // 第三级
    if (cmd === "time" && words[1]?.toLowerCase() === "set") {
      return timeValues.filter((s) => s.startsWith(lowerWord));
    }
    if (cmd === "gamerule") {
      const ruleName = words[1];
      if (gameruleValues[ruleName]) {
        return gameruleValues[ruleName].filter((s) => s.startsWith(lowerWord));
      }
    }
  }

  return [];
}

const filteredSuggestions = computed(() => {
  const input = commandInput.value;

  const cursorPos = inputRef.value?.selectionStart ?? input.length;
  const { word, wordIndex } = getCurrentWordInfo(input, cursorPos);

  // 连续Tab时用原始词匹配，否则用当前词
  const wordToMatch = lastTabWordIndex.value === wordIndex ? lastTabOriginalWord.value : word;

  return getCompletions(input, wordIndex, wordToMatch);
});

function sendCommand() {
  const command = commandInput.value.trim();
  if (!command) return;
  emit("sendCommand", command);
  commandInput.value = "";
  showSuggestions.value = false;
  lastTabOriginalWord.value = "";
  lastTabWordIndex.value = -1;
  tabCycleIndex.value = 0;
}

// 执行逐词补全
function doTabComplete() {
  const input = commandInput.value;
  const cursorPos = inputRef.value?.selectionStart ?? input.length;
  const { word, startIndex, wordIndex } = getCurrentWordInfo(input, cursorPos);

  // 检查是否是连续Tab（基于位置判断，用原始词匹配）
  const isContinuousTab = lastTabWordIndex.value === wordIndex;

  // 连续Tab时用原始词匹配，否则用当前词
  const wordToMatch = isContinuousTab ? lastTabOriginalWord.value : word;
  const completions = getCompletions(input, wordIndex, wordToMatch);

  if (completions.length === 0) return;

  if (isContinuousTab) {
    tabCycleIndex.value = (tabCycleIndex.value + 1) % completions.length;
  } else {
    tabCycleIndex.value = 0;
    lastTabOriginalWord.value = word;
    lastTabWordIndex.value = wordIndex;
  }

  // 无输入时或连续Tab时强制显示所有命令的建议列表
  if ((!word || isContinuousTab) && completions.length > 1) {
    showSuggestions.value = true;
  }

  // 滚动到选中的建议项
  scrollToActiveSuggestion();

  const completion = completions[tabCycleIndex.value];

  // 替换当前词
  const before = input.substring(0, startIndex);
  const after = input.substring(startIndex + word.length);
  const newInput = before + completion + after;

  // 标记正在补全，防止 onInputChange 重置状态
  isCompleting = true;
  commandInput.value = newInput;

  // 设置光标位置到补全词之后
  nextTick(() => {
    if (inputRef.value) {
      const newCursorPos = startIndex + completion.length;
      inputRef.value.setSelectionRange(newCursorPos, newCursorPos);
    }
    isCompleting = false;
  });

  // 更新显示
  showSuggestions.value = completions.length > 1;
  suggestionIndex.value = tabCycleIndex.value;
}

function handleKeydown(e: KeyboardEvent) {
  // 重置Tab状态（非Tab键时）
  if (e.key !== "Tab") {
    lastTabOriginalWord.value = "";
    lastTabWordIndex.value = -1;
    tabCycleIndex.value = 0;
  }

  if (e.key === "Enter") {
    if (showSuggestions.value && filteredSuggestions.value.length > 0) {
      // 使用选中的补全
      const completion = filteredSuggestions.value[suggestionIndex.value];
      applyCompletion(completion);
      showSuggestions.value = false;
    } else {
      sendCommand();
    }
    return;
  }

  if (e.key === "Tab") {
    e.preventDefault();
    doTabComplete();
    return;
  }

  if (e.key === "ArrowUp") {
    e.preventDefault();
    if (showSuggestions.value && suggestionIndex.value > 0) {
      suggestionIndex.value--;
      scrollToActiveSuggestion();
    }
    return;
  }

  if (e.key === "ArrowDown") {
    e.preventDefault();
    if (showSuggestions.value && suggestionIndex.value < filteredSuggestions.value.length - 1) {
      suggestionIndex.value++;
      scrollToActiveSuggestion();
    }
    return;
  }

  if (e.key === "Escape") {
    showSuggestions.value = false;
    return;
  }

  // 只有非Tab键才更新建议列表
  nextTick(() => {
    showSuggestions.value = filteredSuggestions.value.length > 1;
    suggestionIndex.value = 0;
    scrollToActiveSuggestion();
  });
}

// 滚动到选中的建议项，使其保持在中间
function scrollToActiveSuggestion() {
  nextTick(() => {
    if (!suggestionsRef.value) return;

    const activeItem = suggestionsRef.value.querySelector(".suggestion-item.active");
    if (!activeItem) return;

    const popup = suggestionsRef.value;
    const popupHeight = popup.clientHeight;
    const itemHeight = activeItem.clientHeight;
    const itemTop = activeItem.offsetTop;

    // 计算滚动位置，使选中项位于中间
    const scrollPosition = itemTop - popupHeight / 2 + itemHeight / 2;

    // 确保滚动位置在有效范围内
    const maxScroll = popup.scrollHeight - popupHeight;
    const finalScroll = Math.max(0, Math.min(scrollPosition, maxScroll));

    popup.scrollTop = finalScroll;
  });
}

// 应用补全到当前词
function applyCompletion(completion: string) {
  const input = commandInput.value;
  const cursorPos = inputRef.value?.selectionStart ?? input.length;
  const { word, startIndex } = getCurrentWordInfo(input, cursorPos);

  const before = input.substring(0, startIndex);
  const after = input.substring(startIndex + word.length);
  commandInput.value = before + completion + after;

  nextTick(() => {
    if (inputRef.value) {
      const newCursorPos = startIndex + completion.length;
      inputRef.value.setSelectionRange(newCursorPos, newCursorPos);
    }
  });
}

// 输入变化时重置Tab状态（补全过程中跳过）
function onInputChange() {
  if (isCompleting) return;
  lastTabOriginalWord.value = "";
  lastTabWordIndex.value = -1;
  tabCycleIndex.value = 0;
}
</script>

<template>
  <div class="console-input-wrapper">
    <div
      v-if="showSuggestions && filteredSuggestions.length > 0"
      class="suggestions-popup"
      ref="suggestionsRef"
    >
      <div
        v-for="(sug, i) in filteredSuggestions"
        :key="sug"
        class="suggestion-item"
        :class="{ active: i === suggestionIndex }"
        @mousedown.prevent="
          commandInput = sug;
          showSuggestions = false;
        "
      >
        {{ sug }}
      </div>
      <div class="suggestion-hint">Tab 补全 / Up Down 选择</div>
    </div>
    <div class="console-input-bar">
      <span class="input-prefix">&gt;</span>
      <input
        ref="inputRef"
        class="console-input"
        v-model="commandInput"
        :placeholder="i18n.t('common.enter_command')"
        @keydown="handleKeydown"
        @input="onInputChange"
        :style="{ fontSize: consoleFontSize + 'px' }"
      />
      <SLButton variant="primary" size="sm" @click="sendCommand()">{{
        i18n.t("console.send_command")
      }}</SLButton>
    </div>
  </div>
</template>

<style scoped>
.console-input-wrapper {
  position: relative;
  flex-shrink: 0;
}
.suggestions-popup {
  position: absolute;
  bottom: 100%;
  left: 0;
  right: 0;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  margin-bottom: 4px;
  max-height: 200px;
  overflow-y: auto;
  z-index: 20;
  box-shadow: var(--sl-shadow-md);
}
.suggestion-item {
  padding: 6px 14px;
  font-family: var(--sl-font-mono);
  font-size: 0.8125rem;
  color: var(--sl-text-primary);
  cursor: pointer;
  transition: background var(--sl-transition-fast);
}
.suggestion-item:hover,
.suggestion-item.active {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
}
.suggestion-hint {
  padding: 4px 14px;
  font-size: 0.6875rem;
  color: var(--sl-text-tertiary);
  border-top: 1px solid var(--sl-border-light);
}
.console-input-bar {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-sm) var(--sl-space-md);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
}
.input-prefix {
  color: var(--sl-primary);
  font-family: var(--sl-font-mono);
  font-weight: 700;
}
.console-input {
  flex: 1;
  background: transparent;
  color: var(--sl-text-primary);
  font-family: var(--sl-font-mono);
  padding: 6px 0;
  border: none;
  outline: none;
}
.console-input::placeholder {
  color: var(--sl-text-tertiary);
}
</style>
