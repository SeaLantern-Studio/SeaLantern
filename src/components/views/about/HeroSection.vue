<script setup lang="ts">
import { i18n } from "@language";
import logo from "@assets/logo.svg";

defineProps<{
  version: string;
}>();
</script>

<template>
  <div class="hero-section">
    <div class="hero-logo">
      <img :src="logo" alt="Sea Lantern" width="72" height="72" />
    </div>
    <h1 class="hero-title">Sea Lantern</h1>
    <p class="hero-subtitle">{{ i18n.t("about.subtitle") }}</p>
    <div class="hero-badges">
      <cmz-badge variant="primary">{{ "v" + version }}</cmz-badge>
      <cmz-badge variant="success">{{ i18n.t("about.tech_badge") }}</cmz-badge>
      <cmz-badge variant="default">{{ i18n.t("about.license_badge") }}</cmz-badge>
    </div>
    <p class="hero-desc">
      {{ i18n.t("about.hero_desc") }}
    </p>
  </div>
</template>

<style scoped>
.hero-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: var(--sl-space-2xl) 0;
}

.hero-logo {
  margin-bottom: var(--sl-space-md);
  animation: sl-fade-in-up 0.6s ease forwards;
}

.hero-title {
  font-size: 2.5rem;
  font-weight: 800;
  color: var(--sl-text-primary);
  letter-spacing: -0.03em;
  margin-bottom: var(--sl-space-xs);
  animation: sl-fade-in-up 0.6s ease 0.1s both;
}

.hero-subtitle {
  font-size: 1.125rem;
  color: var(--sl-text-secondary);
  margin-bottom: var(--sl-space-md);
  animation: sl-fade-in-up 0.6s ease 0.2s both;
}

.hero-badges {
  display: flex;
  gap: var(--sl-space-sm);
  margin-bottom: var(--sl-space-lg);
  animation: sl-fade-in-up 0.6s ease 0.3s both;
}

.hero-desc {
  font-size: 1rem;
  color: var(--sl-text-secondary);
  line-height: 1.8;
  animation: sl-fade-in-up 0.6s ease 0.4s both;
}
</style>
