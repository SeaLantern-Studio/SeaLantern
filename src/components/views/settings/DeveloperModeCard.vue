<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import { i18n } from "@language";

defineProps<{
  developerMode: boolean;
}>();

const emit = defineEmits<{
  (e: "update:developerMode", value: boolean): void;
  (e: "change"): void;
}>();

// 检测是否在 Tauri 环境中运行
const isTauri = () => {
  return (
    typeof window !== "undefined" && typeof (window as any).__TAURI_INTERNALS__ !== "undefined"
  );
};

const handleSwitchChange = (value: boolean) => {
  emit("update:developerMode", value);
  emit("change");

  // 在非 Tauri 环境（Docker/浏览器）下，点击开关后刷新页面
  // 刷新后会重新读取设置文件，确保状态同步
  if (!isTauri()) {
    setTimeout(() => {
      window.location.reload();
    }, 300);
  }
};
</script>

<template>
  <SLCard
    :title="i18n.t('settings.developer_mode')"
    :subtitle="i18n.t('settings.developer_mode_desc')"
  >
    <div class="sl-settings-group">
      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.developer_mode_toggle") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.developer_mode_toggle_desc") }}</span>
        </div>
        <SLSwitch :model-value="developerMode" @update:model-value="handleSwitchChange" />
      </div>
    </div>
  </SLCard>
</template>
