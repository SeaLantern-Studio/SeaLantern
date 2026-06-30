<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import type { SettingsCoverageItem } from "../../pages/settings/useSettingsPage";

interface Props {
  nextItems: SettingsCoverageItem[];
  classicItems: SettingsCoverageItem[];
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "action", actionId: string): void;
}>();
</script>

<template>
  <SLCard variant="outline" padding="lg" class="settings-classic-bridge-card">
    <div class="settings-classic-bridge-card__header">
      <div>
        <span class="settings-classic-bridge-card__eyebrow">Classic bridge</span>
        <h3 class="settings-classic-bridge-card__title">next 已聚合哪些入口，哪些仍走 classic</h3>
      </div>

      <div class="settings-classic-bridge-card__actions">
        <SLButton variant="secondary" size="sm" @click="emit('action', 'open-paint')">
          打开 classic 个性化页
        </SLButton>
        <SLButton variant="ghost" size="sm" @click="emit('action', 'open-developer')">
          尝试打开 classic 开发工具
        </SLButton>
      </div>
    </div>

    <p class="settings-classic-bridge-card__description">
      `/settings` 现在由 next shell 接管，因此这里不再直接承载 classic SettingsView 全量卡片。
      当前策略是：next 负责统一入口和说明态，复杂设置仍留在 classic 的既有页面体系中。
    </p>

    <div class="settings-classic-bridge-card__grid">
      <section class="settings-classic-bridge-card__column">
        <h4>当前已在 next 聚合</h4>
        <ul>
          <li v-for="item in props.nextItems" :key="item.title">
            <strong>{{ item.title }}</strong>
            <span>{{ item.description }}</span>
          </li>
        </ul>
      </section>

      <section class="settings-classic-bridge-card__column">
        <h4>本轮故意继续走 classic</h4>
        <ul>
          <li v-for="item in props.classicItems" :key="item.title">
            <strong>{{ item.title }}</strong>
            <span>{{ item.description }}</span>
          </li>
        </ul>
      </section>
    </div>
  </SLCard>
</template>

<style scoped>
.settings-classic-bridge-card {
  display: grid;
  gap: 18px;
}

.settings-classic-bridge-card__header {
  display: flex;
  gap: 16px;
  justify-content: space-between;
  align-items: flex-start;
}

.settings-classic-bridge-card__eyebrow {
  display: inline-block;
  margin-bottom: 8px;
  font-size: 0.75rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--sl-text-tertiary);
}

.settings-classic-bridge-card__title,
.settings-classic-bridge-card__description,
.settings-classic-bridge-card__column h4,
.settings-classic-bridge-card__column ul {
  margin: 0;
}

.settings-classic-bridge-card__title {
  font-size: 1.1rem;
}

.settings-classic-bridge-card__actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.settings-classic-bridge-card__description {
  color: var(--sl-text-secondary);
  line-height: 1.7;
}

.settings-classic-bridge-card__grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.settings-classic-bridge-card__column {
  padding: 16px;
  border-radius: 16px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 84%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 74%, transparent);
}

.settings-classic-bridge-card__column h4 {
  margin-bottom: 12px;
  font-size: 0.95rem;
  color: var(--sl-text-primary);
}

.settings-classic-bridge-card__column ul {
  padding-left: 18px;
  display: grid;
  gap: 12px;
}

.settings-classic-bridge-card__column li {
  display: grid;
  gap: 4px;
}

.settings-classic-bridge-card__column strong {
  color: var(--sl-text-primary);
}

.settings-classic-bridge-card__column span {
  color: var(--sl-text-secondary);
  line-height: 1.55;
}

@media (max-width: 900px) {
  .settings-classic-bridge-card__grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 720px) {
  .settings-classic-bridge-card__header {
    flex-direction: column;
  }

  .settings-classic-bridge-card__actions {
    justify-content: flex-start;
  }
}
</style>
