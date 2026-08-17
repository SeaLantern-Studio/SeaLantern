<script setup lang="ts">
import { RouterLink } from "vue-router";
import { Wrench, MessageSquareText, type LucideIcon } from "lucide-vue-next";
import { i18n } from "@language";

/**
 *
 * - 卡片网格展示各独立小工具，点击进入对应工具页
 * - 首个版本仅 MOTD 一个工具卡，结构预留扩展
 */
interface ToolMeta {
  to: string;
  icon: LucideIcon;
  titleKey: string;
  descKey: string;
  tagKey: string;
}

const tools: ToolMeta[] = [
  {
    to: "/motd",
    icon: MessageSquareText,
    titleKey: "config.motd.title",
    descKey: "config.motd.tool_desc",
    tagKey: "common.available",
  },
];
</script>

<template>
  <div class="tools-view">
    <header class="tools-view__header">
      <h1 class="tools-view__title">
        <Wrench :size="22" :stroke-width="1.8" />
        {{ i18n.t("common.tools") }}
      </h1>
      <p class="tools-view__desc">{{ i18n.t("common.tools_desc") }}</p>
    </header>

    <section class="tools-grid" aria-label="工具列表">
      <RouterLink v-for="tool in tools" :key="tool.to" :to="tool.to" class="tool-card-link">
        <article class="tool-card">
          <div class="tool-card__cover">
            <component
              :is="tool.icon"
              class="tool-card__cover-icon"
              :size="40"
              :stroke-width="1.5"
            />
          </div>
          <div class="tool-card__content">
            <div class="tool-card__header">
              <h3 class="tool-card__title">{{ i18n.t(tool.titleKey) }}</h3>
              <span class="tool-card__tag">{{ i18n.t(tool.tagKey) }}</span>
            </div>
            <p class="tool-card__desc">{{ i18n.t(tool.descKey) }}</p>
          </div>
        </article>
      </RouterLink>
    </section>
  </div>
</template>

<style scoped>
.tools-view {
  display: flex;
  flex-direction: column;
  padding: var(--sl-space-lg);
  max-width: 1200px;
  margin: 0 auto;
}

.tools-view__header {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
  margin-bottom: var(--sl-space-lg);
}

.tools-view__title {
  margin: 0;
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}

.tools-view__desc {
  margin: 0;
  font-size: 0.875rem;
  line-height: 1.5;
  color: var(--sl-text-secondary);
}

.tools-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: var(--sl-space-md);
}

.tool-card-link {
  text-decoration: none;
  color: inherit;
}

.tool-card {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
  padding: var(--sl-space-sm);
  overflow: hidden;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  background: var(--sl-bg-secondary);
  transition:
    border-color 0.18s ease,
    box-shadow 0.18s ease,
    transform 0.18s ease;
}

.tool-card-link:hover .tool-card {
  border-color: var(--sl-primary);
  box-shadow: var(--sl-shadow-card);
  transform: translateY(-2px);
}

.tool-card__cover {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  aspect-ratio: 16 / 9;
  background: var(--sl-bg-tertiary);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  color: var(--sl-primary);
}

.tool-card__cover-icon {
  opacity: 0.9;
}

.tool-card__content {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
  min-width: 0;
  flex: 1;
}

.tool-card__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--sl-space-sm);
}

.tool-card__title {
  margin: 0;
  flex: 1;
  min-width: 0;
  font-size: 0.9375rem;
  font-weight: 600;
  line-height: 1.35;
  color: var(--sl-text-primary);
}

.tool-card__tag {
  padding: var(--sl-space-xs) var(--sl-space-sm);
  border-radius: var(--sl-radius-full);
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-tertiary);
  font-size: 0.75rem;
  font-weight: 500;
  flex-shrink: 0;
}

.tool-card__desc {
  margin: 0;
  font-size: 0.8125rem;
  line-height: 1.5;
  color: var(--sl-text-secondary);
}
</style>
