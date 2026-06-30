<script setup lang="ts">
import { RouterView } from "vue-router";
import { provideNextHostRuntime } from "./host/runtime";

provideNextHostRuntime();
</script>

<template>
  <RouterView />
</template>
