<script setup lang="ts">
import { i18n } from "@language";
import { useAboutLinks } from "@composables/useAboutLinks";

const { openLink } = useAboutLinks();
</script>

<template>
  <div class="links-section">
    <cmz-button variant="solid" size="lg" @click="openLink('https://ideaflash.cn/')">
      {{ i18n.t("about.official_site") }}
    </cmz-button>
    <cmz-button
      variant="solid"
      size="lg"
      @click="openLink('https://github.com/SeaLantern-Studio/SeaLantern')"
    >
      {{ i18n.t("about.github_repo") }}
    </cmz-button>
    <cmz-button
      variant="outline"
      size="lg"
      @click="openLink('https://space.bilibili.com/3706927622130406?spm_id_from=333.1387.0.0')"
    >
      {{ i18n.t("about.bilibili") }}
    </cmz-button>
  </div>
</template>

<style scoped>
.links-section {
  display: flex;
  justify-content: center;
  gap: var(--sl-space-md);
}

.links-section :deep(.cmz-button) {
  min-width: 140px;
}
</style>
