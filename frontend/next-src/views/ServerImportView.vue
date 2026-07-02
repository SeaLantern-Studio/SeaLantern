<script setup lang="ts">
import ServerImportPage from "@next-src/pages/servers/ServerImportPage.vue";
import NextProtectedPageView from "./NextProtectedPageView.vue";
</script>

<template>
  <NextProtectedPageView shell-header-mode="hidden">
    <ServerImportPage />
  </NextProtectedPageView>
</template>
