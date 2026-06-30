<script setup lang="ts">
import ServersPage from "../pages/servers/ServersPage.vue";
import NextProtectedPageView from "./NextProtectedPageView.vue";
</script>

<template>
  <NextProtectedPageView
    page-subtitle="复用现有 server store，提供服务器概览、状态信息与 classic 页面快捷入口。"
  >
    <ServersPage />
  </NextProtectedPageView>
</template>
