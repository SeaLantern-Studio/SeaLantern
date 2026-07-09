<script setup lang="ts">
import ServerCreatePage from "@src/pages/servers/ServerCreatePage.vue";
import NextProtectedPageView from "./NextProtectedPageView.vue";
</script>

<template>
  <NextProtectedPageView shell-header-mode="hidden">
    <ServerCreatePage />
  </NextProtectedPageView>
</template>
