<script setup lang="ts">
import SettingsPage from "@src/pages/settings/SettingsPage.vue";
import NextProtectedPageView from "@src/views/NextProtectedPageView.vue";
</script>

<template>
  <NextProtectedPageView shell-header-mode="hidden">
    <SettingsPage />
  </NextProtectedPageView>
</template>
