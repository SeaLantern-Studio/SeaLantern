<script setup lang="ts">
import SettingsPage from "@next-src/pages/settings/SettingsPage.vue";
import NextProtectedPageView from "@next-src/views/NextProtectedPageView.vue";
</script>

<template>
  <NextProtectedPageView shell-header-mode="hidden">
    <SettingsPage />
  </NextProtectedPageView>
</template>
