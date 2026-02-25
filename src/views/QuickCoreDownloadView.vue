<script setup lang="ts">
import { useRouter } from "vue-router";
import { ArrowLeft } from "lucide-vue-next";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";

const router = useRouter();

const goBack = () => {
  router.push("/core-download/mode");
};

const startQuickSetup = () => {
  router.push("/core-download/quick");
};
</script>

<template>
  <div class="quick-view animate-fade-in-up">
    <button class="quick-back-button" @click="goBack">
      <ArrowLeft :size="20" />
      <span>返回</span>
    </button>

    <SLCard class="quick-card" title="快速创建服务器">
      <div class="quick-content">
        <p class="quick-description">快速模式将自动为您推荐最适合的服务器核心和配置</p>

        <div class="quick-actions">
          <SLButton variant="primary" size="lg" @click="startQuickSetup"> 开始快速配置 </SLButton>
        </div>
      </div>
    </SLCard>
  </div>
</template>

<style scoped>
.quick-view {
  padding: var(--sl-space-xl);
  max-width: 800px;
  margin: 0 auto;
}

.quick-back-button {
  display: inline-flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: var(--sl-space-sm) var(--sl-space-md);
  margin-bottom: var(--sl-space-lg);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  color: var(--sl-text-secondary);
  font-size: 0.875rem;
  cursor: pointer;
  transition:
    background-color var(--sl-transition-fast) ease,
    border-color var(--sl-transition-fast) ease,
    color var(--sl-transition-fast) ease;
}

.quick-back-button:hover {
  background: var(--sl-surface-hover);
  border-color: var(--sl-primary);
  color: var(--sl-primary);
}

.quick-card {
  animation: sl-scale-in var(--sl-transition-normal) forwards;
}

.quick-content {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xl);
}

.quick-description {
  font-size: 1rem;
  color: var(--sl-text-secondary);
  line-height: 1.6;
  margin: 0;
}

.quick-actions {
  display: flex;
  justify-content: center;
  padding-top: var(--sl-space-lg);
}
</style>
