<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import SLSelect from "@components/common/SLSelect.vue";
import { i18n } from "@language";
import { computed } from "vue";
import { getThemeOptions } from "@themes";

defineProps<{
  color: string;
  isThemeProxied: boolean;
  themeProxyPluginName: string;
}>();

const emit = defineEmits<{
  (e: "update:color", value: string): void;
  (e: "change"): void;
}>();

const colorOptions = computed(() => getThemeOptions());

function handleColorChange(value: string) {
  emit("update:color", value);
  emit("change");
}
</script>

<template>
  <SLCard>
    <template #header>
      <div class="color-theme-header">
        <div>
          <h3 class="card-title">{{ i18n.t("settings.color_theme") }}</h3>
          <p class="card-subtitle">{{ i18n.t("settings.color_theme_desc") }}</p>
        </div>
        <div class="sl-input-lg">
          <div v-if="isThemeProxied" class="theme-proxied-notice">
            <span class="proxied-text">{{
              i18n.t("settings.theme_proxied_by", { plugin: themeProxyPluginName })
            }}</span>
          </div>
          <SLSelect
            v-else
            :model-value="color"
            :options="colorOptions"
            @update:model-value="handleColorChange"
          />
        </div>
      </div>
    </template>
  </SLCard>
</template>

<style scoped>
.color-theme-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.card-title {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}

.card-subtitle {
  margin: var(--sl-space-xs) 0 0 0;
  font-size: 0.8125rem;
  color: var(--sl-text-tertiary);
}

.theme-proxied-notice {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  background: rgba(96, 165, 250, 0.1);
  border: 1px solid rgba(96, 165, 250, 0.3);
  border-radius: var(--sl-radius-md);
  color: var(--sl-primary);
  font-size: 0.875rem;
  min-width: 200px;
}

.proxied-text {
  white-space: nowrap;
}
</style>
