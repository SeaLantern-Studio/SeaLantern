<script setup lang="ts">
import PaintPage from "@next-src/pages/paint/PaintPage.vue";
import NextProtectedPageView from "@next-src/views/NextProtectedPageView.vue";
</script>

<template>
  <NextProtectedPageView>
    <PaintPage />
  </NextProtectedPageView>
</template>
