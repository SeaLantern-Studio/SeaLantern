<script setup lang="ts">
import AboutPage from "@next-src/pages/about/AboutPage.vue";
import NextProtectedPageView from "@next-src/views/NextProtectedPageView.vue";
</script>

<template>
  <NextProtectedPageView>
    <AboutPage />
  </NextProtectedPageView>
</template>
