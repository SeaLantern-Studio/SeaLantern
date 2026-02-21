<script setup lang="ts">
import { computed } from "vue";
import { Settings } from "lucide-vue-next";
import { i18n } from "../../language";
import type { ServerCommand } from "../../types/server";

interface Props {
  serverId: string;
  currentServerCommands: ServerCommand[];
}

defineProps<Props>();

const emit = defineEmits<{
  (e: "sendCommand", cmd: string): void;
  (e: "openAddCommandModal"): void;
  (e: "openEditCommandModal", cmd: ServerCommand): void;
}>();

// 使用计算属性，确保语言切换时自动更新
const quickCommands = computed(() => [
  { label: i18n.t("common.command_day"), cmd: "time set day" },
  { label: i18n.t("common.command_night"), cmd: "time set night" },
  { label: i18n.t("common.command_clear"), cmd: "weather clear" },
  { label: i18n.t("common.command_rain"), cmd: "weather rain" },
  { label: i18n.t("common.command_save"), cmd: "save-all" },
  { label: i18n.t("common.command_list"), cmd: "list" },
  { label: "", cmd: "" },
  { label: i18n.t("common.command_keep_inventory_on"), cmd: "gamerule keepInventory true" },
  { label: i18n.t("common.command_keep_inventory_off"), cmd: "gamerule keepInventory false" },
  { label: i18n.t("common.command_mob_griefing_off"), cmd: "gamerule mobGriefing false" },
]);
</script>

<template>
  <div class="quick-commands">
    <!-- 快捷指令行 -->
    <div class="command-row">
      <span class="quick-label">{{ i18n.t("console.quick") }}</span>
      <div class="quick-groups">
        <div
          v-for="cmd in quickCommands"
          :key="cmd.cmd"
          class="quick-btn"
          @click="emit('sendCommand', cmd.cmd)"
          :title="cmd.cmd"
        >
          {{ cmd.label }}
        </div>
      </div>
    </div>

    <!-- 自定义指令行 -->
    <div v-if="serverId" class="command-row custom-commands-row">
      <div class="custom-label">{{ i18n.t("console.custom") }}</div>
      <div class="custom-buttons">
        <div
          v-for="cmd in currentServerCommands"
          :key="cmd.id"
          class="custom-btn"
          @click="emit('sendCommand', cmd.command)"
          :title="cmd.command"
        >
          <span class="custom-btn-name">{{ cmd.name }}</span>
          <span class="custom-btn-edit" @click.stop="emit('openEditCommandModal', cmd)"
            ><Settings :size="14"
          /></span>
        </div>
        <div
          class="custom-btn add-btn"
          @click="emit('openAddCommandModal')"
          :title="i18n.t('console.add_custom_command')"
        >
          <span class="add-btn-plus">+</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.quick-commands {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-sm);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  flex-shrink: 0;
}

.command-row {
  display: flex;
  align-items: flex-start;
  gap: var(--sl-space-sm);
  flex-wrap: wrap;
}

.custom-commands-row {
  margin-top: 2px;
}
.quick-label {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  white-space: nowrap;
  margin-top: 3px;
}
.quick-groups {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
  align-items: center;
  flex: 1;
  min-width: 0;
}
.quick-btn {
  padding: 3px 10px;
  border-radius: var(--sl-radius-sm);
  font-size: 0.75rem;
  cursor: pointer;
  border: 1px solid var(--sl-border);
  color: var(--sl-text-secondary);
  background: var(--sl-bg-secondary);
  white-space: nowrap;
  transition: all var(--sl-transition-fast);
}
.quick-btn:hover {
  border-color: var(--sl-primary);
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
}

/* 自定义指令样式 */
.custom-label {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  white-space: nowrap;
  margin-top: 3px;
}

.custom-buttons {
  display: flex;
  gap: 4px;
  align-items: center;
  flex-wrap: wrap;
  flex: 1;
  min-width: 0;
}

.custom-btn {
  position: relative;
  padding: 3px 10px;
  border-radius: var(--sl-radius-sm);
  font-size: 0.75rem;
  cursor: pointer;
  border: 1px solid var(--sl-border);
  color: var(--sl-text-secondary);
  background: var(--sl-bg-secondary);
  white-space: nowrap;
  transition: all var(--sl-transition-fast);
  min-width: 60px;
  text-align: center;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.custom-btn:hover {
  border-color: var(--sl-primary);
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
}

.custom-btn-name {
  font-weight: 500;
  flex: 1;
}

.custom-btn-edit {
  margin-left: 6px;
  font-size: 0.85rem;
  opacity: 0;
  transform: scale(0.8);
  transition: all var(--sl-transition-fast);
  cursor: pointer;
}

.custom-btn:hover .custom-btn-edit {
  opacity: 1;
  transform: scale(1);
}

.custom-btn.add-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  padding: 0;
  font-size: 1.2rem;
  color: var(--sl-text-tertiary);
  border: 1px dashed var(--sl-border-light);
  background: transparent;
}

.custom-btn.add-btn:hover {
  border-color: var(--sl-primary);
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
}

.add-btn-plus {
  line-height: 1;
}
</style>
