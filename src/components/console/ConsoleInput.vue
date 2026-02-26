<script setup lang="ts">
import { ref, computed, nextTick } from "vue";
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";

interface Props {
  consoleFontSize: number;
  minecraftVersion: string; // 新增：当前服务器版本
}

interface CommandTree {
  [rootCommand: string]: string[]; // 根命令 -> 二级子命令列表
}

interface VersionDelta {
  version: string;          // 变更发生的版本（如 "1.13"）
  adds?: Record<string, string[]>;     // 新增的根命令及其子命令
  removes?: string[];                  // 移除的根命令
  renames?: { old: string; new: string; subcommands?: string[] }[]; // 重命名
  subcommandAdds?: { command: string; subcommands: string[] }[];     // 给现有命令新增子命令
  subcommandRemoves?: { command: string; subcommands: string[] }[]; // 移除子命令
}

const props = defineProps<Props>();

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

// 基命令树（Classic 0.0.15a）
const baseTree: CommandTree = {
  ban: [],
  banip: [],
  broadcast: [],
  deop: [],
  kick: [],
  op: [],
  tp: [],
  unban: [],
};

const versionDeltas: VersionDelta[] = [
  // ==================== Classic / Alpha / Beta ====================
  {
    version: "0.0.16a_01",
    renames: [{ old: "broadcast", new: "say" }]
  },
  {
    version: "0.0.17a",
    adds: { setspawn: [] }
  },
  {
    version: "0.0.20a",
    adds: { solid: [] }
  },
  // Indev 0.31: 移除所有命令
  {
    version: "0.31",
    removes: Object.keys(baseTree)
  },
  {
    version: "v1.0.16",
    adds: {
      "ban": [], "ban-ip": [], "banlist": [], "deop": [],
      "kick": [], "op": [], "pardon": [], "pardon-ip": [], "say": [], "stop": []
    }
  },
  {
    version: "v1.0.16_01",
    adds: { "save-all": [], "save-on": [], "save-off": [], "tp": [] }
  },
  {
    version: "v1.0.16_02",
    adds: { list: [], msg: [] }
  },
  {
    version: "v1.2.0",
    removes: ["wood", "iron"]
  },
  {
    version: "v1.2.5",
    removes: ["home"]
  },
  {
    version: "v1.2.6",
    adds: { kill: [] }
  },
  {
    version: "b1.3", // Beta 1.3
    adds: { whitelist: [] }
  },

  // ==================== Java Edition 1.x ====================
  {
    version: "1.3.1",
    adds: {
      gamemode: [],
      give: [],
      time: ["set"],
      toggledownfall: [],
      xp: [],
      help: [],
      seed: [],
      defaultgamemode: [],
      publish: [],
      debug: []
    }
  },
  {
    version: "1.4.2",
    adds: {
      difficulty: [],
      gamerule: [],
      spawnpoint: [],
      weather: [],
      clear: [],
      enchant: []
    }
  },
  {
    version: "1.5",
    adds: {
      testfor: [],
      scoreboard: [],
      effect: []
    }
  },
  {
    version: "1.6.1",
    adds: {
      spreadplayers: [],
      playsound: []
    }
  },
  {
    version: "1.7.2",
    adds: {
      summon: [],
      achievement: [],
      setblock: [],
      tellraw: [],
      testforblock: [],
      setidletimeout: [],
      setworldspawn: []
    }
  },
  {
    version: "1.8",
    adds: {
      blockdata: [],
      clone: [],
      fill: [],
      particle: [],
      trigger: [],
      execute: [],
      testforblocks: [],
      worldborder: [],
      title: [],
      replaceitem: [],
      stats: [],
      entitydata: []
    }
  },
  {
    version: "1.9.3-pre2",
    adds: { stopsound: [] }
  },
  {
    version: "1.10",
    adds: { teleport: [] }
  },
  {
    version: "1.11",
    adds: { locate: [] }
  },
  {
    version: "1.12",
    adds: { recipe: [], advancement: [], reload: [], function: [] },
    removes: ["achievement"]
  },
  // ==================== 1.13 大改 ====================
  {
    version: "1.13",
    adds: {
      tag: [],
      team: [],
      data: [],
      datapack: [],
      bossbar: [],
      forceload: [],          // 由 /chunk 重命名而来
      enchant: []              // 重新加入
    },
    removes: [
      "stats", "testfor", "testforblock", "testforblocks", "toggledownfall",
      "entitydata", "blockdata"
    ],
    subcommandAdds: [
      { command: "time", subcommands: ["add", "query"] },
      { command: "gamerule", subcommands: ["randomTickSpeed"] }
    ]
  },
  {
    version: "1.14",
    adds: { schedule: [], loot: [], teammsg: [] },
    renames: [{ old: "drop", new: "loot" }]
  },
  {
    version: "1.16",
    adds: { locatebiome: [], attribute: [] }
  },
  {
    version: "1.17",
    adds: { item: [], perf: [] },
    removes: ["replaceitem"]
  },
  {
    version: "1.18",
    adds: { jfr: [] }
  },
  {
    version: "1.18.2",
    adds: { placefeature: [] }
  },
  {
    version: "1.19",
    adds: {
      place: [],
      warden_spawn_tracker: []
    },
    subcommandAdds: [
      { command: "locate", subcommands: ["biome", "structure"] }
    ],
    removes: ["locatebiome", "placefeature"]
  },
  {
    version: "1.19.3",
    adds: { fillbiome: [] }
  },
  {
    version: "1.19.4",
    adds: { ride: [], damage: [] }
  },
  {
    version: "1.20.2",
    adds: { random: [] }
  },
  {
    version: "1.20.3",
    adds: { tick: [] }
  },
  {
    version: "1.20.5",
    adds: { transfer: [] }
  },
  {
    version: "1.21.2",
    adds: { rotate: [] }
  },
  {
    version: "1.21.6",
    adds: { version: [], waypoint: [], dialog: [] }
  },
  {
    version: "1.21.9",
    adds: { fetchprofile: [] }
  },
  {
    version: "1.21.11",
    adds: { stopwatch: [] }
  },
  {
    version: "26.1",
    adds: { swing: [] }
  }
];

// 提取版本中的数字部分，例如 "v1.0.16" -> [1,0,16], "b1.3" -> [1,3], "1.21.4" -> [1,21,4]
function parseVersion(version: string): number[] {
  // 提取所有连续的数字，忽略非数字字符
  const matches = version.match(/\d+/g);
  return matches ? matches.map(Number) : [];
}

// 比较两个版本，如果 v1 <= v2 返回 true
function compareVersions(v1: string, v2: string): number {
  const a = parseVersion(v1);
  const b = parseVersion(v2);
  const maxLen = Math.max(a.length, b.length);
  for (let i = 0; i < maxLen; i++) {
    const ai = i < a.length ? a[i] : 0;
    const bi = i < b.length ? b[i] : 0;
    if (ai !== bi) return ai - bi;
  }
  return 0;
}

function isVersionLessOrEqual(v1: string, v2: string): boolean {
  return compareVersions(v1, v2) <= 0;
}

// 根据目标版本构建命令树
function buildCommandTree(targetVersion: string): CommandTree {
  const tree = JSON.parse(JSON.stringify(baseTree)) as CommandTree;

  // 过滤出所有版本 <= targetVersion 的 delta
  const applicableDeltas = versionDeltas
    .filter(delta => isVersionLessOrEqual(delta.version, targetVersion))
    .sort((a, b) => compareVersions(a.version, b.version));

  for (const delta of applicableDeltas) {
    // 应用移除
    if (delta.removes) {
      delta.removes.forEach(cmd => delete tree[cmd]);
    }
    // 应用重命名
    if (delta.renames) {
      delta.renames.forEach(({ old, new: newCmd }) => {
        if (tree[old]) {
          tree[newCmd] = tree[old];
          delete tree[old];
        }
      });
    }
    // 应用新增
    if (delta.adds) {
      Object.entries(delta.adds).forEach(([cmd, subs]) => {
        tree[cmd] = subs;
      });
    }
    // 应用子命令新增
    if (delta.subcommandAdds) {
      delta.subcommandAdds.forEach(({ command, subcommands }) => {
        if (tree[command]) {
          tree[command] = [...new Set([...tree[command], ...subcommands])];
        }
      });
    }
    // 应用子命令移除
    if (delta.subcommandRemoves) {
      delta.subcommandRemoves.forEach(({ command, subcommands }) => {
        if (tree[command]) {
          tree[command] = tree[command].filter(s => !subcommands.includes(s));
        }
      });
    }
  }
  return tree;
}

// 根据版本获取 gamerule 可选值
function getGameruleValues(version: string): Record<string, string[]> {
  // 此处可根据版本返回不同的规则集，示例为 1.20+ 常用规则
  return {
    keepInventory: ["true", "false"],
    doDaylightCycle: ["true", "false"],
    mobGriefing: ["true", "false"],
    randomTickSpeed: ["3", "4", "0"],
    playersSleepingPercentage: ["0", "50", "100"],
  };
}

// 根据版本获取 time set 的可选值
function getTimeValues(version: string): string[] {
  // 1.13+ 支持 add/query，但补全仍可用 day/night/noon 等
  return ["day", "night", "noon", "midnight"];
}

// 当前版本对应的命令数据
const currentCommandData = computed(() => {
  const tree = buildCommandTree(props.minecraftVersion);
  return {
    commandTree: tree,
    gameruleValues: getGameruleValues(props.minecraftVersion),
    timeValues: getTimeValues(props.minecraftVersion),
  };
});

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
function getCompletions(
  input: string,
  wordIndex: number,
  currentWord: string,
  commandTree: CommandTree,
  gameruleValues: Record<string, string[]>,
  timeValues: string[]
): string[] {
  const words = input.trim().split(/\s+/);
  const lowerWord = currentWord.toLowerCase();

  if (wordIndex === 0) {
    // 第一级：匹配命令名
    if (!currentWord) {
      return Object.keys(commandTree).toSorted();
    }
    return Object.keys(commandTree)
      .filter((cmd) => cmd.toLowerCase().startsWith(lowerWord))
      .toSorted();
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

  const { commandTree, gameruleValues, timeValues } = currentCommandData.value;
  return getCompletions(input, wordIndex, wordToMatch, commandTree, gameruleValues, timeValues);
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
  const { commandTree, gameruleValues, timeValues } = currentCommandData.value;
  const completions = getCompletions(input, wordIndex, wordToMatch, commandTree, gameruleValues, timeValues);

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
