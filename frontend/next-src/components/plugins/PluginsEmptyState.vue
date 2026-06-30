<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";

defineProps<{
  loading: boolean;
}>();

defineEmits<{
  (e: "refresh"): void;
  (e: "open-market"): void;
}>();
</script>

<template>
  <SLCard class="plugins-empty-state" variant="outline">
    <div class="plugins-empty-state__content">
      <span class="plugins-empty-state__eyebrow">Plugins / next</span>
      <h2 class="plugins-empty-state__title">当前还没有已安装插件</h2>
      <p class="plugins-empty-state__description">
        这页已经接入真实 plugin store。你可以先去 classic
        市场安装插件，或者刷新一次重新扫描本地插件目录。
      </p>

      <div class="plugins-empty-state__actions">
        <SLButton variant="primary" size="sm" @click="$emit('open-market')">
          前往 classic 市场
        </SLButton>
        <SLButton variant="secondary" size="sm" :loading="loading" @click="$emit('refresh')">
          重新扫描插件
        </SLButton>
      </div>
    </div>
  </SLCard>
</template>

<style scoped>
.plugins-empty-state__content {
  padding: 8px;
  display: grid;
  gap: 12px;
}

.plugins-empty-state__eyebrow {
  font-size: 0.78rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--sl-text-tertiary);
}

.plugins-empty-state__title {
  margin: 0;
  color: var(--sl-text-primary);
}

.plugins-empty-state__description {
  margin: 0;
  color: var(--sl-text-secondary);
  line-height: 1.7;
}

.plugins-empty-state__actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}
</style>
