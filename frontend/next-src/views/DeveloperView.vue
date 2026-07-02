<script setup lang="ts">
import DeveloperPage from "@next-src/pages/developer/DeveloperPage.vue";
import NextProtectedPageView from "@next-src/views/NextProtectedPageView.vue";
</script>

<template>
  <NextProtectedPageView>
    <DeveloperPage />
  </NextProtectedPageView>
</template>
