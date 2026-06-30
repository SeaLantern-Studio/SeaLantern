<script setup lang="ts">
import { Blocks, DownloadIcon, Home, Info, Link2, Puzzle, Server, Settings, Sparkles, Store, Terminal, Users, type LucideIcon } from "@lucide/vue";
import type { NextSidebarHostItem } from "../../host/runtime";

interface Props {
  items: NextSidebarHostItem[];
}

defineProps<Props>();

const iconMap: Record<string, LucideIcon> = {
  home: Home,
  terminal: Terminal,
  settings: Settings,
  users: Users,
  info: Info,
  server: Server,
  blocks: Blocks,
  store: Store,
  sparkles: Sparkles,
  link2: Link2,
  download: DownloadIcon,
  puzzle: Puzzle,
};

function getNavIcon(name?: string): LucideIcon {
  if (!name) {
    return Blocks;
  }

  return iconMap[name] ?? Info;
}
</script>

<template>
  <div v-if="items.length > 0" class="next-sidebar-host-items" role="list">
    <div
      v-for="item in items"
      :key="item.pluginId"
      class="next-sidebar-host-items__item"
      role="listitem"
      :title="item.label"
    >
      <component :is="getNavIcon(item.icon)" :size="16" class="next-sidebar-host-items__icon" />
      <span class="next-sidebar-host-items__label">{{ item.label }}</span>
    </div>
  </div>
</template>

<style scoped>
.next-sidebar-host-items {
  display: grid;
  gap: 8px;
}

.next-sidebar-host-items__item {
  min-height: 40px;
  padding: 0 12px;
  display: flex;
  align-items: center;
  gap: 10px;
  line-height: 0;
  border-radius: 14px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 68%, transparent);
  color: var(--sl-text-secondary);
}

.next-sidebar-host-items__icon {
  flex: 0 0 auto;
  display: block;
}

.next-sidebar-host-items__label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--sl-font-size-sm);
}
</style>
