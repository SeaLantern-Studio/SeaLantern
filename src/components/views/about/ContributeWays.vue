<script setup lang="ts">
import { Code, Feather, Lightbulb, BookOpen, Globe, Rocket } from "lucide-vue-next";
import SLCard from "@components/common/SLCard.vue";
import { i18n } from "@language";
</script>

<template>
  <SLCard :title="i18n.t('about.contribute_ways')">
    <div class="contribute-ways">
      <div class="way-item">
        <div class="way-icon">
          <Code :size="20" :stroke-width="2" />
        </div>
        <div class="way-info">
          <span class="way-title">{{ i18n.t("about.way_code") }}</span>
          <span class="way-desc">{{ i18n.t("about.way_code_desc") }}</span>
        </div>
      </div>
      <div class="way-item">
        <div class="way-icon">
          <Feather :size="20" :stroke-width="2" />
        </div>
        <div class="way-info">
          <span class="way-title">{{ i18n.t("about.way_design") }}</span>
          <span class="way-desc">{{ i18n.t("about.way_design_desc") }}</span>
        </div>
      </div>
      <div class="way-item">
        <div class="way-icon">
          <Lightbulb :size="20" :stroke-width="2" />
        </div>
        <div class="way-info">
          <span class="way-title">{{ i18n.t("about.way_idea") }}</span>
          <span class="way-desc">{{ i18n.t("about.way_idea_desc") }}</span>
        </div>
      </div>
      <div class="way-item">
        <div class="way-icon">
          <BookOpen :size="20" :stroke-width="2" />
        </div>
        <div class="way-info">
          <span class="way-title">{{ i18n.t("about.way_doc") }}</span>
          <span class="way-desc">{{ i18n.t("about.way_doc_desc") }}</span>
        </div>
      </div>
      <div class="way-item">
        <div class="way-icon">
          <Globe :size="20" :stroke-width="2" />
        </div>
        <div class="way-info">
          <span class="way-title">{{ i18n.t("about.way_translate") }}</span>
          <span class="way-desc">{{ i18n.t("about.way_translate_desc") }}</span>
        </div>
      </div>
      <div class="way-item">
        <div class="way-icon">
          <Rocket :size="20" :stroke-width="2" />
        </div>
        <div class="way-info">
          <span class="way-title">{{ i18n.t("about.way_promote") }}</span>
          <span class="way-desc">{{ i18n.t("about.way_promote_desc") }}</span>
        </div>
      </div>
    </div>
  </SLCard>
</template>

<style scoped>
.contribute-ways {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--sl-space-sm);
}

.way-item {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-sm);
  border-radius: var(--sl-radius-md);
  transition: background var(--sl-transition-fast);
}

.way-item:hover {
  background: var(--sl-bg-secondary);
}

.way-icon {
  flex-shrink: 0;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
  border-radius: var(--sl-radius-md);
  transition: all var(--sl-transition-fast);
}

.way-item:hover .way-icon {
  background: var(--sl-primary);
  color: white;
  transform: scale(1.05);
}

.way-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.way-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}

.way-desc {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
}

@media (max-width: 768px) {
  .contribute-ways {
    grid-template-columns: 1fr;
  }
}
</style>
