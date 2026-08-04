<script setup lang="ts">
import { computed, ref } from "vue";
import { i18n } from "@language";
import { searchResources, type ResourceSearchResult } from "@api/resource";
import { useToast } from "cmzya-modern-ui";
import { useLoading } from "@composables/useAsync";

const toast = useToast();
const keyword = ref("");
const results = ref<ResourceSearchResult[]>([]);
const activeResult = ref<ResourceSearchResult | null>(null);

const { loading, execute: executeSearch } = useLoading(async () => {
  if (!keyword.value.trim()) {
    results.value = [];
    return [];
  }

  const found = await searchResources(keyword.value.trim(), 24);
  results.value = found;
  if (found.length === 0) {
    toast.info(i18n.t("common.resourceMarket.no_results"));
  }
  return found;
});

const resultCountText = computed(() => {
  if (!results.value.length) {
    return i18n.t("common.resourceMarket.no_results");
  }
  return i18n.t("common.resourceMarket.result_count", { count: results.value.length });
});

function handleSearch() {
  executeSearch().catch((err) => {
    console.error(err);
    toast.error(err?.message ?? i18n.t("common.message_unknown_error"));
  });
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Enter") {
    handleSearch();
  }
}

function openSourceUrl(item: ResourceSearchResult) {
  if (!item.sourceUrl) return;
  window.open(item.sourceUrl, "_blank");
}

function selectResult(item: ResourceSearchResult) {
  activeResult.value = item;
}
</script>

<template>
  <div class="resource-market-view animate-stagger-in">
    <section class="resource-market-header">
      <div>
        <h2>{{ i18n.t("common.resourceMarket.title") }}</h2>
        <p class="resource-market-tip">{{ i18n.t("common.resourceMarket.tip") }}</p>
      </div>
      <div class="resource-market-search">
        <cmz-input
          v-model="keyword"
          :placeholder="i18n.t('common.resourceMarket.search_placeholder')"
          @keydown="handleKeydown"
          clearable
          class="resource-search-input"
        >
          <template #append>
            <cmz-button
              :loading="loading"
              :disabled="keyword.trim() === ''"
              variant="solid"
              @click="handleSearch"
            >
              {{ i18n.t("common.resourceMarket.search") }}
            </cmz-button>
          </template>
        </cmz-input>
      </div>
    </section>

    <section class="resource-market-body">
      <div class="resource-market-summary">
        <span>{{ resultCountText }}</span>
      </div>

      <div class="resource-market-list">
        <template v-if="results.length">
          <div
            v-for="item in results"
            :key="item.source + '-' + item.id"
            class="resource-market-card"
            :class="{
              active: activeResult?.id === item.id && activeResult?.source === item.source,
            }"
            @click="selectResult(item)"
          >
            <div class="resource-card-avatar">
              <img v-if="item.iconUrl" :src="item.iconUrl" alt="icon" />
              <div v-else class="resource-card-avatar-fallback">{{ item.name.slice(0, 1) }}</div>
            </div>
            <div class="resource-card-content">
              <div class="resource-card-title">
                <span>{{ item.name }}</span>
                <span class="resource-card-badge">{{ item.source.toUpperCase() }}</span>
              </div>
              <p class="resource-card-summary">{{ item.summary }}</p>
              <div class="resource-card-meta">
                <span v-if="item.author">{{ item.author }}</span>
                <span v-if="item.downloads !== undefined">{{
                  i18n.t("common.resourceMarket.downloads", { count: item.downloads })
                }}</span>
              </div>
            </div>
            <cmz-button size="sm" variant="ghost" @click.stop="openSourceUrl(item)">
              {{ i18n.t("common.resourceMarket.view") }}
            </cmz-button>
          </div>
        </template>

        <div v-else class="resource-market-empty">
          <div class="empty-state">
            <p>{{ i18n.t("common.resourceMarket.no_results") }}</p>
            <span>{{ i18n.t("common.search") }}</span>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.resource-market-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-lg);
}

.resource-market-header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: var(--sl-space-md);
}

.resource-market-tip {
  margin-top: 8px;
  color: var(--sl-text-muted);
}

.resource-market-search {
  min-width: 320px;
  width: 100%;
  max-width: 640px;
}

.resource-search-input :deep(.cmz-input-container) {
  width: 100%;
}

.resource-market-body {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}

.resource-market-summary {
  color: var(--sl-text-muted);
}

.resource-market-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: var(--sl-space-md);
}

.resource-market-card {
  display: flex;
  align-items: stretch;
  gap: var(--sl-space-md);
  padding: var(--sl-space-md);
  border-radius: var(--sl-radius-lg);
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.06);
  transition:
    transform 0.2s ease,
    border-color 0.2s ease,
    background 0.2s ease;
  cursor: pointer;
}

.resource-market-card:hover,
.resource-market-card.active {
  transform: translateY(-2px);
  border-color: rgba(255, 255, 255, 0.14);
  background: rgba(255, 255, 255, 0.12);
}

.resource-card-avatar {
  width: 56px;
  min-width: 56px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.08);
  overflow: hidden;
}

.resource-card-avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.resource-card-avatar-fallback {
  font-weight: 700;
  color: var(--sl-text-primary);
}

.resource-card-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.resource-card-title {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--sl-space-sm);
  font-weight: 600;
}

.resource-card-badge {
  padding: 2px 10px;
  border-radius: 999px;
  font-size: 12px;
  color: var(--sl-text-muted);
  background: rgba(255, 255, 255, 0.08);
}

.resource-card-summary {
  color: var(--sl-text-muted);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.resource-card-meta {
  display: flex;
  gap: var(--sl-space-md);
  flex-wrap: wrap;
  color: var(--sl-text-secondary);
  font-size: 13px;
}

.resource-market-empty {
  grid-column: 1 / -1;
  padding: var(--sl-space-lg);
  text-align: center;
  color: var(--sl-text-muted);
}

.empty-state span {
  display: inline-block;
  margin-top: 8px;
  color: var(--sl-text-secondary);
}

@media (max-width: 900px) {
  .resource-market-header {
    flex-direction: column;
    align-items: stretch;
  }

  .resource-market-list {
    grid-template-columns: 1fr;
  }
}
</style>
