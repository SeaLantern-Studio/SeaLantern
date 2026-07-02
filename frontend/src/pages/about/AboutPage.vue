<script setup lang="ts">
import AboutFooter from "@src/components/about/AboutFooter.vue";
import ContributorWall from "@src/components/about/ContributorWall.vue";
import ContributeWays from "@src/components/about/ContributeWays.vue";
import LinksSection from "@src/components/about/LinksSection.vue";
import ManifestoCard from "@src/components/about/ManifestoCard.vue";
import ProjectInfo from "@src/components/about/ProjectInfo.vue";
import WorkbenchFactGrid from "@src/components/workbench/WorkbenchFactGrid.vue";
import WorkbenchPageIntro from "@src/components/workbench/WorkbenchPageIntro.vue";
import { i18n } from "@language";
import { useAboutPage } from "./useAboutPage";

const { version, summaryFacts } = useAboutPage();
</script>

<template>
  <div class="about-page">
    <WorkbenchPageIntro
      :eyebrow="i18n.t('common.about')"
      :title="i18n.t('about.title')"
      :description="i18n.t('about.hero_desc')"
    />

    <WorkbenchFactGrid :items="summaryFacts" />

    <ManifestoCard />

    <div class="about-page__info-grid">
      <ProjectInfo :version="version" />
      <ContributeWays />
    </div>

    <ContributorWall />
    <LinksSection />
    <AboutFooter />
  </div>
</template>

<style scoped>
.about-page {
  min-width: 0;
  display: grid;
  gap: 16px;
  padding-bottom: 12px;
}

.about-page__info-grid {
  min-width: 0;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

@media (max-width: 900px) {
  .about-page__info-grid {
    grid-template-columns: 1fr;
  }
}
</style>
