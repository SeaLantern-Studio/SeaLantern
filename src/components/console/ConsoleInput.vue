<script setup lang="ts">
import { ref, computed, nextTick } from "vue";
import SLButton from "../common/SLButton.vue";
import { i18n } from "../../locales";

interface Props {
  consoleFontSize: number;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: "sendCommand", cmd: string): void;
}>();

const commandInput = ref("");
const showSuggestions = ref(false);
const suggestionIndex = ref(0);

const allCommands = [
  "help",
  "list",
  "stop",
  "say",
  "time set day",
  "time set night",
  "time set noon",
  "weather clear",
  "weather rain",
  "weather thunder",
  "gamemode survival",
  "gamemode creative",
  "gamemode adventure",
  "gamemode spectator",
  "difficulty peaceful",
  "difficulty easy",
  "difficulty normal",
  "difficulty hard",
  "give",
  "tp",
  "teleport",
  "kill",
  "kick",
  "ban",
  "pardon",
  "op",
  "deop",
  "whitelist add",
  "whitelist remove",
  "whitelist list",
  "gamerule keepInventory true",
  "gamerule keepInventory false",
  "gamerule doDaylightCycle true",
  "gamerule doDaylightCycle false",
  "gamerule mobGriefing true",
  "gamerule mobGriefing false",
  "save-all",
  "tps",
  "plugins",
  "version",
];

const filteredSuggestions = computed(() => {
  const input = commandInput.value.trim().toLowerCase();
  if (!input) return [];
  return allCommands
    .filter((c) => c.toLowerCase().startsWith(input) && c.toLowerCase() !== input)
    .slice(0, 8);
});

function sendCommand() {
  const command = commandInput.value.trim();
  if (!command) return;
  emit("sendCommand", command);
  commandInput.value = "";
  showSuggestions.value = false;
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Enter") {
    if (showSuggestions.value && filteredSuggestions.value.length > 0) {
      commandInput.value = filteredSuggestions.value[suggestionIndex.value];
      showSuggestions.value = false;
    } else {
      sendCommand();
    }
    return;
  }
  if (e.key === "Tab") {
    e.preventDefault();
    if (filteredSuggestions.value.length > 0) {
      commandInput.value = filteredSuggestions.value[suggestionIndex.value];
      showSuggestions.value = false;
    }
    return;
  }
  if (e.key === "ArrowUp") {
    e.preventDefault();
    if (showSuggestions.value && suggestionIndex.value > 0) suggestionIndex.value--;
    return;
  }
  if (e.key === "ArrowDown") {
    e.preventDefault();
    if (showSuggestions.value && suggestionIndex.value < filteredSuggestions.value.length - 1)
      suggestionIndex.value++;
    return;
  }
  if (e.key === "Escape") {
    showSuggestions.value = false;
    return;
  }
  nextTick(() => {
    showSuggestions.value =
      commandInput.value.trim().length > 0 && filteredSuggestions.value.length > 0;
    suggestionIndex.value = 0;
  });
}
</script>

<template>
  <div class="console-input-wrapper">
    <div v-if="showSuggestions && filteredSuggestions.length > 0" class="suggestions-popup">
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
        class="console-input"
        v-model="commandInput"
        :placeholder="i18n.t('common.enter_command')"
        @keydown="handleKeydown"
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
