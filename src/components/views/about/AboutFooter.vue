<script setup lang="ts">
import { i18n } from "@language";
</script>

<template>
  <div class="about-footer">
    <p class="footer-text">{{ i18n.t("about.footer_text1") }}</p>
    <p class="footer-text">{{ i18n.t("about.footer_text2") }}</p>
    <p class="footer-quote">{{ i18n.t("about.footer_quote") }}</p>
  </div>
</template>

<style scoped>
.about-footer {
  text-align: center;
  padding: var(--sl-space-xl) 0;
  border-top: 1px solid var(--sl-border-light);
}

.footer-text {
  font-size: 0.8125rem;
  color: var(--sl-text-tertiary);
  margin-bottom: var(--sl-space-xs);
}

.footer-quote {
  font-size: 1rem;
  font-weight: 500;
  color: var(--sl-primary);
  font-style: italic;
  margin-top: var(--sl-space-md);
}
</style>
