<script setup lang="ts">
import { i18n } from "@language";
import SLCard from "@components/common/SLCard.vue";
</script>

<template>
  <SLCard>
    <div class="manifesto">
      <h3 class="manifesto-title">{{ i18n.t("about.manifesto_title") }}</h3>
      <p class="manifesto-text">
        {{ i18n.t("about.manifesto_text1") }}
      </p>
      <p class="manifesto-text">
        {{ i18n.t("about.manifesto_text2") }}
      </p>
    </div>
  </SLCard>
</template>

<style scoped>
.manifesto {
  text-align: center;
  padding: var(--sl-space-lg);
}

.manifesto-title {
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--sl-text-primary);
  margin-bottom: var(--sl-space-md);
}

.manifesto-text {
  font-size: 0.9375rem;
  color: var(--sl-text-secondary);
  line-height: 1.8;
  margin-bottom: var(--sl-space-sm);
}
</style>
