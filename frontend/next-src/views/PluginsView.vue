<script setup lang="ts">
import PluginsPage from "../pages/plugins/PluginsPage.vue";
import NextProtectedPageView from "./NextProtectedPageView.vue";
</script>

<template>
  <NextProtectedPageView
    page-subtitle="复用现有 plugin store / API，先把已安装插件管理页做实，并保留 classic 市场与详情桥接入口。"
  >
    <PluginsPage />
  </NextProtectedPageView>
</template>
