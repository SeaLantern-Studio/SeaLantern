<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import AppearanceSection from "@src/components/settings/appearance/AppearanceSection.vue";
import DeveloperManagementSection from "@src/components/settings/developer/DeveloperManagementSection.vue";
import GeneralSection from "@src/components/settings/general/GeneralSection.vue";
import { i18n } from "@language";
import { NEXT_ABOUT_ROUTE_NAME } from "@src/router/pageMeta";
import { useSettingsPage } from "./useSettingsPage";

const router = useRouter();

const {
  bootstrapping,
  refreshing,
  hasError,
  sectionItems,
  currentSectionId,
  currentSection,
  loadPage,
  selectSection,
} = useSettingsPage();

const activeSectionComponent = computed(() => {
  switch (currentSectionId.value) {
    case "appearance":
      return AppearanceSection;
    case "developer-management":
      return DeveloperManagementSection;
    default:
      return GeneralSection;
  }
});

function openAboutPage(): void {
  void router.push({ name: NEXT_ABOUT_ROUTE_NAME });
}
</script>

<template>
  <div class="settings-page">
    <header class="settings-page__intro">
      <div class="settings-page__intro-copy">
        <h2 class="settings-page__title">{{ i18n.t("settings.next.page_title") }}</h2>
        <p class="settings-page__description">{{ i18n.t("settings.next.page_description") }}</p>
      </div>

      <SLButton variant="secondary" size="sm" :loading="refreshing" @click="loadPage(true)">
        {{ i18n.t("settings.next.refresh") }}
      </SLButton>

      <SLButton variant="ghost" size="sm" @click="openAboutPage">
        {{ i18n.t("common.about") }}
      </SLButton>
    </header>

    <section v-if="hasError" class="settings-page__error-banner" role="alert" aria-live="polite">
      <div class="settings-page__error-copy">
        <strong>{{ i18n.t("settings.next.error_title") }}</strong>
        <span>{{ i18n.t("settings.next.error_description") }}</span>
      </div>

      <div class="settings-page__error-actions">
        <SLButton variant="secondary" size="sm" :loading="refreshing" @click="loadPage(true)">
          {{ i18n.t("settings.next.retry") }}
        </SLButton>
      </div>
    </section>

    <section
      v-if="bootstrapping"
      class="settings-page__workspace settings-page__workspace--loading"
    >
      <div class="settings-page__loading-directory" aria-hidden="true">
        <div class="settings-page__loading-pill"></div>
        <div class="settings-page__loading-pill"></div>
      </div>

      <SLCard variant="outline" padding="lg" class="settings-page__loading-content">
        <div class="settings-page__loading-heading"></div>
        <div class="settings-page__loading-line settings-page__loading-line--wide"></div>
        <div class="settings-page__loading-line"></div>
        <div class="settings-page__loading-grid">
          <div class="settings-page__loading-block"></div>
          <div class="settings-page__loading-block"></div>
        </div>
      </SLCard>
    </section>

    <section v-else class="settings-page__workspace">
      <aside class="settings-page__directory">
        <nav class="settings-directory" :aria-label="i18n.t('settings.next.nav_aria_label')">
          <button
            v-for="section in sectionItems"
            :key="section.id"
            type="button"
            class="settings-directory__item"
            :class="{
              'settings-directory__item--active': currentSectionId === section.id,
            }"
            :aria-current="currentSectionId === section.id ? 'location' : undefined"
            @click="selectSection(section.id)"
          >
            <span class="settings-directory__item-label">{{ section.label }}</span>
          </button>
        </nav>
      </aside>

      <div class="settings-page__content">
        <header class="settings-page__section-header">
          <div class="settings-page__section-copy">
            <h3 class="settings-page__section-title">{{ currentSection.title }}</h3>
            <p class="settings-page__section-description">{{ currentSection.description }}</p>
          </div>
        </header>

        <component :is="activeSectionComponent" />
      </div>
    </section>
  </div>
</template>

<style scoped>
.settings-page {
  min-width: 0;
  display: grid;
  gap: 16px;
}

.settings-page__intro {
  display: flex;
  gap: 16px;
  align-items: center;
  justify-content: space-between;
}

.settings-page__intro-copy {
  min-width: 0;
  display: grid;
  gap: 3px;
}

.settings-page__title,
.settings-page__description,
.settings-page__section-title,
.settings-page__section-description {
  margin: 0;
}

.settings-page__title {
  font-size: 1.22rem;
  font-weight: 600;
  line-height: 1.12;
  letter-spacing: -0.02em;
  color: var(--sl-text-primary);
}

.settings-page__description {
  font-size: 0.87rem;
  color: var(--sl-text-secondary);
  line-height: 1.45;
}

.settings-page__error-banner {
  display: flex;
  gap: 12px;
  justify-content: space-between;
  align-items: flex-start;
  padding: 14px 16px;
  border-radius: 18px;
  border: 1px solid rgba(239, 68, 68, 0.22);
  background: rgba(239, 68, 68, 0.1);
  color: var(--sl-error);
}

.settings-page__error-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.settings-page__error-copy strong {
  font-size: 0.95rem;
}

.settings-page__error-copy span {
  line-height: 1.5;
  word-break: break-word;
}

.settings-page__error-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.settings-page__workspace {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(168px, 212px) minmax(0, 1fr);
  gap: 28px;
  align-items: start;
}

.settings-page__directory {
  min-width: 0;
  position: sticky;
  top: 2px;
}

.settings-directory {
  min-width: 0;
  position: relative;
  display: grid;
  gap: 2px;
  padding: 4px 0;
}

.settings-directory::before {
  content: "";
  position: absolute;
  inset: 6px auto 6px 0;
  width: 1px;
  background: color-mix(in srgb, var(--sl-border) 62%, transparent);
}

.settings-page__content {
  min-width: 0;
  display: grid;
  gap: 14px;
}

.settings-directory__item {
  position: relative;
  width: 100%;
  min-height: 40px;
  padding: 0 14px 0 16px;
  display: inline-flex;
  align-items: center;
  justify-content: flex-start;
  border-radius: 12px;
  border: none;
  background: transparent;
  color: var(--sl-text-secondary);
  transition:
    background-color 0.2s ease,
    color 0.2s ease;
}

.settings-directory__item::before {
  content: "";
  position: absolute;
  left: 0;
  top: 7px;
  bottom: 7px;
  width: 2px;
  border-radius: 999px;
  background: transparent;
  transition: background-color 0.2s ease;
}

.settings-directory__item:hover {
  background: color-mix(in srgb, var(--sl-primary) 5%, transparent);
  color: var(--sl-text-primary);
}

.settings-directory__item--active {
  background: color-mix(in srgb, var(--sl-primary) 8%, transparent);
  color: var(--sl-text-primary);
}

.settings-directory__item--active::before {
  background: var(--sl-primary);
}

.settings-directory__item:focus-visible {
  outline: none;
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--sl-primary) 18%, transparent);
}

.settings-directory__item-label {
  font-size: 0.9rem;
  font-weight: 500;
}

.settings-directory__item--active .settings-directory__item-label {
  font-weight: 600;
}

.settings-page__section-header {
  min-width: 0;
  padding: 0 2px;
}

.settings-page__section-copy {
  min-width: 0;
  display: grid;
  gap: 4px;
}

.settings-page__section-title {
  font-size: 1.02rem;
  font-weight: 600;
  line-height: 1.2;
  color: var(--sl-text-primary);
}

.settings-page__section-description {
  max-width: 52ch;
  color: var(--sl-text-secondary);
  font-size: 0.84rem;
  line-height: 1.45;
}

.settings-page__workspace--loading {
  grid-template-columns: minmax(180px, 220px) minmax(0, 1fr);
}

.settings-page__loading-directory,
.settings-page__loading-content {
  display: grid;
  gap: 12px;
}

.settings-page__loading-directory {
  position: relative;
  padding: 4px 0;
}

.settings-page__loading-directory::before {
  content: "";
  position: absolute;
  inset: 6px auto 6px 0;
  width: 1px;
  background: color-mix(in srgb, var(--sl-border) 62%, transparent);
}

.settings-page__loading-pill,
.settings-page__loading-heading,
.settings-page__loading-line,
.settings-page__loading-block {
  border-radius: 12px;
  background: linear-gradient(
    90deg,
    color-mix(in srgb, var(--sl-bg-secondary) 86%, transparent) 0%,
    color-mix(in srgb, var(--sl-surface) 92%, transparent) 50%,
    color-mix(in srgb, var(--sl-bg-secondary) 86%, transparent) 100%
  );
  background-size: 200% 100%;
  animation: settings-page-skeleton 1.2s ease-in-out infinite;
}

.settings-page__loading-pill {
  width: calc(100% - 10px);
  height: 40px;
  margin-left: 10px;
  border-radius: 12px;
}

.settings-page__loading-heading {
  width: 120px;
  height: 16px;
}

.settings-page__loading-line {
  width: 48%;
  height: 12px;
}

.settings-page__loading-line--wide {
  width: 62%;
}

.settings-page__loading-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.settings-page__loading-block {
  min-height: 180px;
}

@keyframes settings-page-skeleton {
  0% {
    background-position: 100% 0;
  }

  100% {
    background-position: -100% 0;
  }
}

@media (max-width: 900px) {
  .settings-page__workspace,
  .settings-page__workspace--loading,
  .settings-page__loading-grid {
    grid-template-columns: 1fr;
  }

  .settings-page__directory {
    position: static;
  }
}

@media (max-width: 720px) {
  .settings-page__intro,
  .settings-page__error-banner {
    flex-direction: column;
  }
}
</style>
