<script setup lang="ts">
import { ref, onMounted } from "vue";
import HeroSection from "@components/views/about/HeroSection.vue";
import ManifestoCard from "@components/views/about/ManifestoCard.vue";
import ContributorWall from "@components/views/about/ContributorWall.vue";
import ProjectInfo from "@components/views/about/ProjectInfo.vue";
import ContributeWays from "@components/views/about/ContributeWays.vue";
import LinksSection from "@components/views/about/LinksSection.vue";
import AboutFooter from "@components/views/about/AboutFooter.vue";
import { getAppVersion } from "@utils/version";

const version = ref("Loading...");

onMounted(async () => {
  version.value = await getAppVersion();
});
</script>

<template>
  <div class="about-view animate-fade-in-up">
    <HeroSection :version="version" />
    <ManifestoCard />
    <ContributorWall />
    <div class="info-grid">
      <ProjectInfo :version="version" />
      <ContributeWays />
    </div>
    <LinksSection />
    <AboutFooter />
  </div>
</template>

<style scoped>
.about-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xl);
  max-width: 900px;
  margin: 0 auto;
  padding-bottom: var(--sl-space-2xl);
}

.info-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--sl-space-md);
}

@media (max-width: 768px) {
  .info-grid {
    grid-template-columns: 1fr;
  }
}
</style>
