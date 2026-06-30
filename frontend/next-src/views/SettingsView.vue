<script setup lang="ts">
import SettingsPage from "../pages/settings/SettingsPage.vue";
import NextProtectedPageView from "./NextProtectedPageView.vue";
</script>

<template>
  <NextProtectedPageView
    page-subtitle="复用现有 settings/auth 语义，先把 next 下的设置入口聚合页做实，并保留 classic 能力说明与桥接入口。"
  >
    <SettingsPage />
  </NextProtectedPageView>
</template>
