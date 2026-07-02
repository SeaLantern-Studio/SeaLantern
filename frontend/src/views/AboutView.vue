<script setup lang="ts">
import AboutPage from "@src/pages/about/AboutPage.vue";
import NextProtectedPageView from "@src/views/NextProtectedPageView.vue";
</script>

<template>
  <NextProtectedPageView>
    <AboutPage />
  </NextProtectedPageView>
</template>
