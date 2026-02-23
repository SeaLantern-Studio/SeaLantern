<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";
import { useAboutLinks } from "@composables/useAboutLinks";

const { openLink } = useAboutLinks();
</script>

<template>
  <div class="links-section">
    <SLButton variant="primary" size="lg" @click="openLink('https://gitee.com/fps_z/SeaLantern')">
      {{ i18n.t("about.gitee_repo") }}
    </SLButton>
    <SLButton
      variant="primary"
      size="lg"
      @click="openLink('https://github.com/SeaLantern-Studio/SeaLantern')"
    >
      {{ i18n.t("about.github_repo") }}
    </SLButton>
    <SLButton
      variant="secondary"
      size="lg"
      @click="openLink('https://space.bilibili.com/3706927622130406?spm_id_from=333.1387.0.0')"
    >
      {{ i18n.t("about.bilibili") }}
    </SLButton>
  </div>
</template>

<style scoped>
.links-section {
  display: flex;
  justify-content: center;
  gap: var(--sl-space-md);
}

.links-section :deep(.sl-button) {
  min-width: 140px;
}
</style>
