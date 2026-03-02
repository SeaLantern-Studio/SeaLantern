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
      <span class="version-badge">v{{ version }}</span>
      <span class="tech-badge">{{ i18n.t("about.tech_badge") }}</span>
      <span class="license-badge">{{ i18n.t("about.license_badge") }}</span>
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

.version-badge,
.tech-badge,
.license-badge {
  padding: 4px 14px;
  border-radius: var(--sl-radius-full);
  font-size: 0.8125rem;
  font-weight: 500;
}

.version-badge {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
}

.tech-badge {
  background: rgba(34, 197, 94, 0.1);
  color: var(--sl-success);
}

.license-badge {
  background: rgba(168, 85, 247, 0.1);
  color: #a855f7;
}

[data-theme="dark"] .license-badge {
  color: #c084fc;
}

.hero-desc {
  font-size: 1rem;
  color: var(--sl-text-secondary);
  line-height: 1.8;
  animation: sl-fade-in-up 0.6s ease 0.4s both;
}
</style>
