<script setup lang="ts">
import { ref, computed } from "vue";
import { AvatarImage, AvatarRoot, AvatarFallback } from "reka-ui";
import { Plus, Link, ExternalLink, Check } from "lucide-vue-next";
import SLButton from "@components/common/SLButton.vue";
import BrandIcon from "@components/common/BrandIcon.vue";
import { contributors as contributorsList, type SocialLinks } from "@data/contributors";
import { i18n } from "@language";
import tauriIcon64 from "@src-tauri/icons/64x64.png";
import { useAboutLinks } from "@composables/useAboutLinks";

const contributors = ref(contributorsList);

const PAGE_SIZE = 9;
const currentPage = ref(1);

const displayedContributors = computed(() => {
  return contributors.value.slice(0, currentPage.value * PAGE_SIZE);
});

const hasMore = computed(() => {
  return displayedContributors.value.length < contributors.value.length;
});

function loadMore() {
  currentPage.value++;
}

const { copiedQQ, openLink, copyQQ, openSocialLink } = useAboutLinks();

function isSocialLinks(url: string | SocialLinks | undefined): url is SocialLinks {
  return typeof url === "object" && url !== null;
}

function getCustomLinks(links: SocialLinks): [string, string][] {
  const predefined = ["gitee", "github", "bilibili", "qq", "tiktok"];
  return Object.entries(links).filter(([key, value]) => !predefined.includes(key) && value) as [
    string,
    string,
  ][];
}
</script>

<template>
  <div class="contributor-section">
    <div class="section-header">
      <h2 class="section-title">{{ i18n.t("about.contributor_wall") }}</h2>
      <p class="section-desc">{{ i18n.t("about.contributor_desc") }}</p>
    </div>

    <div class="contributor-grid">
      <div v-for="c in displayedContributors" :key="c.name" class="contributor-card glass-card">
        <AvatarRoot class="contributor-avatar" :alt="c.name">
          <AvatarImage :src="c.avatar" :alt="c.name" />
          <AvatarFallback>
            <img :src="tauriIcon64" />
          </AvatarFallback>
        </AvatarRoot>

        <div class="contributor-right">
          <div class="contributor-info" :title="c.name + ' - ' + c.role">
            <span class="contributor-name">{{ c.name }}</span>
            <span class="contributor-role">{{ c.role }}</span>
          </div>

          <div v-if="c.url" class="contributor-social">
            <template v-if="!isSocialLinks(c.url)">
              <a
                :href="c.url"
                target="_blank"
                rel="noopener noreferrer"
                class="social-icon"
                :title="i18n.t('about.personal_page')"
              >
                <Link :size="16" />
              </a>
            </template>

            <template v-else>
              <a
                v-if="c.url.gitee"
                :href="c.url.gitee"
                target="_blank"
                rel="noopener noreferrer"
                class="social-icon"
                title="Gitee"
              >
                <BrandIcon name="gitee" :size="16" />
              </a>

              <a
                v-if="c.url.github"
                :href="c.url.github"
                target="_blank"
                rel="noopener noreferrer"
                class="social-icon"
                title="GitHub"
              >
                <BrandIcon name="github" :size="16" />
              </a>

              <a
                v-if="c.url.bilibili"
                :href="c.url.bilibili"
                target="_blank"
                rel="noopener noreferrer"
                class="social-icon"
                title="Bilibili"
              >
                <BrandIcon name="bilibili" :size="16" />
              </a>

              <a
                v-if="c.url.tiktok"
                :href="c.url.tiktok"
                target="_blank"
                rel="noopener noreferrer"
                class="social-icon"
                title="TikTok"
              >
                <BrandIcon name="tiktok" :size="16" />
              </a>

              <button
                v-if="c.url.qq"
                @click="openSocialLink('qq', c.url.qq)"
                class="social-icon"
                :class="{ copied: copiedQQ === c.url.qq }"
                :title="
                  copiedQQ === c.url.qq ? i18n.t('about.copied') : i18n.t('about.qq_click_copy')
                "
              >
                <Check v-if="copiedQQ === c.url.qq" :size="16" />
                <BrandIcon v-else name="qq" :size="16" />
              </button>

              <a
                v-for="[key, value] in getCustomLinks(c.url)"
                :key="key"
                :href="value"
                target="_blank"
                rel="noopener noreferrer"
                class="social-icon"
                :title="key"
              >
                <ExternalLink :size="16" />
              </a>
            </template>
          </div>
        </div>
      </div>

      <!-- Join Card -->
      <div class="contributor-card glass-card join-card">
        <div class="join-icon">
          <Plus :size="40" :stroke-width="1.5" />
        </div>
        <div class="contributor-right">
          <div class="contributor-info">
            <span class="contributor-name join-text">{{ i18n.t("about.join_text") }}</span>
            <span class="contributor-role">{{ i18n.t("about.join_desc") }}</span>
          </div>
        </div>
      </div>
    </div>

    <div v-if="hasMore" class="load-more-section">
      <SLButton variant="ghost" @click="loadMore">
        {{ i18n.t("about.load_more") }} ({{ contributors.length - displayedContributors.length }})
      </SLButton>
    </div>
  </div>
</template>

<style scoped>
.contributor-section {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}

.section-header {
  text-align: center;
}

.section-title {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--sl-text-primary);
  margin-bottom: var(--sl-space-xs);
}

.section-desc {
  font-size: 0.9375rem;
  color: var(--sl-text-tertiary);
}

.contributor-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: var(--sl-space-md);
}

.contributor-card {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
  padding: var(--sl-space-md);
  transition: all var(--sl-transition-normal);
}

.contributor-card.clickable {
  cursor: pointer;
}

.contributor-card.clickable:hover {
  transform: translateY(-4px);
  box-shadow: var(--sl-shadow-lg);
}

.contributor-avatar {
  width: 48px;
  height: 48px;
  border-radius: var(--sl-radius-md);
  flex-shrink: 0;
  image-rendering: pixelated;
}

.contributor-info {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
  min-width: 0;
  overflow: hidden;
}

.contributor-name {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  flex-shrink: 0;
}

.contributor-role {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  line-height: 1.3;
}

.join-card {
  border: 2px dashed var(--sl-border);
  background: rgba(255, 255, 255, 0.6);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  cursor: default;
}

.join-card:hover {
  border-color: var(--sl-primary-light);
  background: var(--sl-primary-bg);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  transform: none;
  box-shadow: none;
}

[data-theme="dark"] .join-card {
  background: rgba(15, 17, 23, 0.7);
}

[data-theme="dark"] .join-card:hover {
  background: var(--sl-primary-bg);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

[data-acrylic="true"] .join-card {
  background: rgba(255, 255, 255, 0.35);
  backdrop-filter: blur(6px);
  -webkit-backdrop-filter: blur(6px);
}

[data-acrylic="true"] .join-card:hover {
  background: var(--sl-primary-bg);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

[data-theme="dark"][data-acrylic="true"] .join-card {
  background: rgba(15, 17, 23, 0.35);
}

[data-theme="dark"][data-acrylic="true"] .join-card:hover {
  background: var(--sl-primary-bg);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

.join-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.join-text {
  color: var(--sl-primary);
}

.contributor-right {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
  flex: 1;
  min-width: 0;
}

.contributor-social {
  display: flex;
  gap: var(--sl-space-xs);
  flex-wrap: wrap;
}

.social-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  padding: 4px;
  border-radius: var(--sl-radius-sm);
  color: var(--sl-text-secondary);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all var(--sl-transition-fast);
}

.social-icon:hover {
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
  transform: scale(1.1);
}

.social-icon svg {
  width: 16px;
  height: 16px;
}

.social-icon.copied {
  color: var(--sl-success);
  background: rgba(34, 197, 94, 0.1);
}

.load-more-section {
  display: flex;
  justify-content: center;
  margin-top: var(--sl-space-lg);
}

@media (max-width: 768px) {
  .contributor-grid {
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  }
}
</style>
