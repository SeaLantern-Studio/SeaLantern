<script setup lang="ts">
import { useRouter } from "vue-router";
import { Zap, Settings } from "lucide-vue-next";

const router = useRouter();

const modes = [
  {
    id: "quick",
    icon: Zap,
    title: "快速模式",
    description: "图形化配置服务器，简单快速",
    route: "/core-download/quick",
  },
  {
    id: "manual",
    icon: Settings,
    title: "手动模式",
    description: "自定义选择核心类型、版本等详细配置",
    route: "/create",
  },
];

const selectMode = (route: string) => {
  router.push(route);
};
</script>

<template>
  <div class="mode-view animate-fade-in-up">
    <div class="mode-container">
      <h1 class="mode-title">选择创建方式</h1>
      <p class="mode-subtitle">选择最适合你的服务器创建方式</p>

      <div class="mode-cards">
        <button
          v-for="mode in modes"
          :key="mode.id"
          class="mode-card"
          @click="selectMode(mode.route)"
        >
          <div class="mode-card-icon">
            <component :is="mode.icon" :size="48" />
          </div>
          <h2 class="mode-card-title">{{ mode.title }}</h2>
          <p class="mode-card-description">{{ mode.description }}</p>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.mode-view {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: calc(100vh - var(--sl-header-height));
  padding: var(--sl-space-2xl);
}

.mode-container {
  max-width: 900px;
  width: 100%;
  text-align: center;
}

.mode-title {
  font-size: 2rem;
  font-weight: 700;
  color: var(--sl-text-primary);
  margin: 0 0 var(--sl-space-sm) 0;
}

.mode-subtitle {
  font-size: 1rem;
  color: var(--sl-text-secondary);
  margin: 0 0 var(--sl-space-2xl) 0;
}

.mode-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: var(--sl-space-xl);
}

.mode-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--sl-space-2xl);
  background: var(--sl-surface);
  border: 2px solid var(--sl-border);
  border-radius: var(--sl-radius-lg);
  cursor: pointer;
  transition:
    transform 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94),
    border-color 0.3s ease,
    box-shadow 0.3s ease;
  will-change: transform, box-shadow;
}

.mode-card:hover {
  transform: translateY(-12px);
  border-color: var(--sl-primary);
  box-shadow:
    0 12px 32px rgba(14, 165, 233, 0.15),
    0 0 0 1px var(--sl-primary),
    0 0 24px rgba(14, 165, 233, 0.3);
}

.mode-card:active {
  transform: translateY(-8px);
}

.mode-card-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 96px;
  height: 96px;
  margin-bottom: var(--sl-space-lg);
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
  border-radius: var(--sl-radius-xl);
  transition:
    background-color 0.3s ease,
    box-shadow 0.3s ease;
}

.mode-card:hover .mode-card-icon {
  background: rgba(14, 165, 233, 0.15);
  box-shadow:
    0 0 32px rgba(14, 165, 233, 0.4),
    0 0 16px rgba(14, 165, 233, 0.2) inset;
}

.mode-card-title {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 var(--sl-space-sm) 0;
}

.mode-card-description {
  font-size: 0.9375rem;
  color: var(--sl-text-secondary);
  line-height: 1.6;
  margin: 0;
}
</style>
