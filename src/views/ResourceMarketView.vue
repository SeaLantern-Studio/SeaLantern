<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { i18n } from "@language";
import { searchResources, type ResourceSearchResult, type ResourceSource } from "@api/resource";
import { useToast } from "cmzya-modern-ui";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Download, User } from "lucide-vue-next";
import logoUrl from "@assets/logo.svg";

const PAGE_SIZE = 24;

const toast = useToast();
const keyword = ref("");
// 顶部筛选占位:真实筛选维度接入前先用占位标签撑起胶囊 tab,后续替换为 i18n 文案
const activeFilter = ref("filter1");
const filterTabs = [
  { key: "filter1", label: "筛选一" },
  { key: "filter2", label: "筛选二" },
];
const results = ref<ResourceSearchResult[]>([]);
const activeResult = ref<ResourceSearchResult | null>(null);
const loading = ref(false);
const loadingMore = ref(false);
const hasMore = ref(true);
// 跨来源去重:同一资源可能同时出现在两个平台,按 source+id 记录
const seenKeys = new Set<string>();
// 已加载完成的资源图标,加载前先用软件 logo 占位
const loadedIcons = reactive(new Set<string>());

// 当前分页偏移,搜索重置,加载更多递增
let offset = 0;

function itemKey(item: ResourceSearchResult): string {
  return `${item.source}-${item.id}`;
}

function appendResults(items: ResourceSearchResult[]): number {
  let added = 0;
  for (const item of items) {
    const key = itemKey(item);
    if (seenKeys.has(key)) continue;
    seenKeys.add(key);
    results.value.push(item);
    added++;
  }
  return added;
}

async function loadPage(
  query: string,
  pageOffset: number,
): Promise<{ added: number; hasMore: boolean }> {
  const page = await searchResources(query, PAGE_SIZE, pageOffset);
  if (pageOffset === 0) {
    seenKeys.clear();
    results.value = [];
  }
  const added = appendResults(page.results);
  return { added, hasMore: page.hasMore };
}

// 进入页面先拉一批默认热门内容,两个来源合并按下载量排序
async function loadInitial() {
  loading.value = true;
  try {
    offset = 0;
    const { added, hasMore: more } = await loadPage("", 0);
    hasMore.value = more && added > 0;
  } catch (err) {
    console.error(err);
    toast.error(errorMessage(err));
  } finally {
    loading.value = false;
  }
}

onMounted(loadInitial);

// 清空搜索词时自动回到默认热门列表
watch(keyword, (value) => {
  if (value.trim() === "") {
    loadInitial();
  }
});

const resultCountText = computed(() =>
  i18n.t("common.resourceMarket.result_count", { count: results.value.length }),
);

// 来源展示名与胶囊主色:CurseForge 橙、Modrinth 绿,底色由 badge 自动派生
function sourceLabel(source: ResourceSource): string {
  return source === "curseforge" ? "CurseForge" : "Modrinth";
}

function sourceColor(source: ResourceSource): string {
  return source === "curseforge" ? "var(--sl-warning)" : "var(--sl-success)";
}

function formatDownloads(count: number): string {
  return new Intl.NumberFormat().format(count);
}

function errorMessage(err: unknown): string {
  return err instanceof Error ? err.message : i18n.t("common.message_unknown_error");
}

function isActive(item: ResourceSearchResult): boolean {
  return activeResult.value?.id === item.id && activeResult.value?.source === item.source;
}

async function handleSearch() {
  if (!keyword.value.trim()) {
    loadInitial();
    return;
  }
  loading.value = true;
  offset = 0;
  try {
    const { added, hasMore: more } = await loadPage(keyword.value.trim(), 0);
    hasMore.value = more && added > 0;
    if (added === 0) {
      toast.info(i18n.t("common.resourceMarket.no_results"));
    }
  } catch (err) {
    console.error(err);
    toast.error(errorMessage(err));
  } finally {
    loading.value = false;
  }
}

async function loadMore() {
  if (loading.value || loadingMore.value || !hasMore.value) return;
  loadingMore.value = true;
  try {
    const nextOffset = offset + PAGE_SIZE;
    const { added, hasMore: more } = await loadPage(keyword.value.trim(), nextOffset);
    hasMore.value = more && added > 0;
    offset = nextOffset;
  } catch (err) {
    console.error(err);
    toast.error(errorMessage(err));
  } finally {
    loadingMore.value = false;
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Enter") {
    handleSearch();
  }
}

async function openSourceUrl(item: ResourceSearchResult) {
  if (!item.sourceUrl) return;
  try {
    await openUrl(item.sourceUrl);
  } catch (error) {
    console.error("Failed to open resource source URL:", error);
  }
}

function selectResult(item: ResourceSearchResult) {
  activeResult.value = item;
}
</script>

<template>
  <div class="resource-market-view animate-stagger-in">
    <!-- 顶部工具条:横向胶囊 tab(筛选占位)+ 尾部搜索框,页面标题由顶栏统一承担 -->
    <section class="resource-market-toolbar">
      <cmz-tab-bar v-model="activeFilter" :tabs="filterTabs" :level="2">
        <template #extra>
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
        </template>
      </cmz-tab-bar>
    </section>

    <section class="resource-market-body">
      <div v-if="results.length" class="resource-market-summary">
        <span>{{ resultCountText }}</span>
      </div>

      <div class="resource-market-list">
        <template v-if="results.length">
          <cmz-card
            v-for="item in results"
            :key="item.source + '-' + item.id"
            class="resource-card"
            :class="{ 'is-active': isActive(item) }"
            padding="md"
            hoverable
            @click="selectResult(item)"
          >
            <div v-if="item.iconUrl" class="resource-card-blur" aria-hidden="true">
              <img :src="item.iconUrl" alt="" />
            </div>
            <div class="resource-card-body">
              <div class="resource-card-head">
                <div class="resource-card-icon">
                  <img class="resource-card-icon-placeholder" :src="logoUrl" alt="" />
                  <img
                    v-if="item.iconUrl"
                    class="resource-card-icon-img"
                    :class="{ 'is-loaded': loadedIcons.has(itemKey(item)) }"
                    :src="item.iconUrl"
                    :alt="item.name"
                    @load="loadedIcons.add(itemKey(item))"
                  />
                </div>
                <div class="resource-card-titleblock">
                  <span class="resource-card-name" :title="item.name">{{ item.name }}</span>
                  <cmz-badge
                    class="resource-card-pill"
                    :text="sourceLabel(item.source)"
                    :color="sourceColor(item.source)"
                    size="large"
                  />
                </div>
              </div>
              <p class="resource-card-summary">{{ item.summary }}</p>
              <div class="resource-card-foot">
                <div class="resource-card-meta">
                  <span v-if="item.author" class="resource-card-meta-item">
                    <User class="resource-card-meta-icon" />
                    <span class="resource-card-meta-text">{{ item.author }}</span>
                  </span>
                  <span v-if="item.downloads !== undefined" class="resource-card-meta-item">
                    <Download class="resource-card-meta-icon" />
                    <span class="resource-card-meta-text">{{
                      i18n.t("common.resourceMarket.downloads", {
                        count: formatDownloads(item.downloads),
                      })
                    }}</span>
                  </span>
                </div>
                <cmz-button size="sm" variant="outline" @click.stop="openSourceUrl(item)">
                  {{ i18n.t("common.resourceMarket.view") }}
                </cmz-button>
              </div>
            </div>
          </cmz-card>
        </template>

        <div v-else class="resource-market-empty">
          <div class="empty-state">
            <p>{{ i18n.t("common.resourceMarket.no_results") }}</p>
            <span>{{ i18n.t("common.search") }}</span>
          </div>
        </div>
      </div>

      <cmz-button
        v-if="hasMore && results.length"
        class="resource-market-load-more"
        variant="outline"
        :loading="loadingMore"
        @click="loadMore"
      >
        {{ i18n.t("common.load_more") }}
      </cmz-button>
    </section>
  </div>
</template>

<style scoped>
.resource-market-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-lg);
}

/* 顶部工具条:筛选胶囊 tab 与尾部搜索框同行。
   库对横向 tab 默认把 extra 槽压到下一行(flex-direction: column),
   这里拉回同一行让搜索框贴在胶囊尾部;特异性拔到 (0,4,0)/(0,5,0) 压过库规则 */
.resource-market-toolbar {
  display: flex;
}

.resource-market-toolbar :deep(.cmz-tab-bar.cmz-tab-bar--level-2) {
  flex: 1;
  min-width: 0;
  flex-direction: row;
  align-items: center;
  margin-bottom: 0;
}

.resource-market-toolbar :deep(.cmz-tab-bar.cmz-tab-bar--level-2 .cmz-tab-bar__tabs) {
  flex-shrink: 0;
}

.resource-market-toolbar :deep(.cmz-tab-bar.cmz-tab-bar--level-2 .cmz-tab-bar__extra) {
  flex: 1;
  min-width: 0;
  margin: 0;
  justify-content: flex-end;
}

/* 胶囊 tab 的材质托底已提到 cmz-fallback.css 全局统一,此处不再重复声明 */

.resource-search-input {
  width: 100%;
}

.resource-search-input :deep(.cmz-input-container) {
  width: 100%;
}

.resource-market-body {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}

.resource-market-summary {
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-sm);
}

.resource-market-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: var(--sl-space-md);
}

/* 单个资源一张独立卡片,悬浮/选中态交给 cmz-card 自带特效 */
.resource-card {
  height: 100%;
  cursor: pointer;
}

.resource-card :deep(.cmz-card-body) {
  flex: 1;
  height: 100%;
  min-height: 0;
}

/* 图标虚化铺满整卡当氛围底,关掉材质模式时卡片也不单调;cmz-card 的
   overflow hidden 会把虚化裁进圆角里 */
.resource-card-blur {
  position: absolute;
  inset: 0;
  z-index: 0;
  overflow: hidden;
  pointer-events: none;
  border-radius: inherit;
}

.resource-card-blur img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transform: scale(1.28);
  filter: blur(8px) saturate(1.25);
  opacity: 0.4;
}

/* 用主题 surface 色从上往下压虚化图,保住文字对比度,明暗主题自动适配 */
.resource-card-blur::after {
  content: "";
  position: absolute;
  inset: 0;
  background: linear-gradient(
    180deg,
    color-mix(in srgb, var(--sl-surface) 82%, transparent),
    color-mix(in srgb, var(--sl-surface) 66%, transparent)
  );
}

.resource-card-body {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
  height: 100%;
  min-height: 0;
}

.resource-card.is-active {
  border-color: var(--sl-primary);
}

.resource-card-head {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
}

.resource-card-icon {
  width: 56px;
  height: 56px;
  flex-shrink: 0;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--sl-radius-lg);
  background: var(--sl-bg-tertiary);
  overflow: hidden;
}

/* 软件 logo 占位:资源图标加载出来前垫底,失败时也回落到它 */
.resource-card-icon-placeholder {
  position: absolute;
  inset: 0;
  margin: auto;
  width: 60%;
  height: 60%;
  object-fit: contain;
  opacity: 0.4;
}

.resource-card-icon-img {
  position: relative;
  z-index: 1;
  width: 100%;
  height: 100%;
  object-fit: cover;
  opacity: 0;
  transition: opacity var(--sl-transition-normal);
}

.resource-card-icon-img.is-loaded {
  opacity: 1;
}

/* 标题块纵向分层:名称独占一行,来源胶囊沉到下一行当元数据 */
.resource-card-titleblock {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--sl-space-xs);
}

.resource-card-name {
  min-width: 0;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--sl-font-size-lg);
  font-weight: 600;
  color: var(--sl-text-primary);
}

.resource-card-pill {
  flex-shrink: 0;
}

.resource-card-summary {
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-sm);
  line-height: 1.75;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.resource-card-foot {
  margin-top: auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-md);
}

.resource-card-meta {
  min-width: 0;
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--sl-space-xs) var(--sl-space-md);
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-xs);
}

/* meta 项:图标锚点 + 可截断文字,方便快速扫读 */
.resource-card-meta-item {
  display: inline-flex;
  align-items: center;
  gap: var(--sl-space-xs);
  min-width: 0;
}

.resource-card-meta-icon {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  color: var(--sl-text-tertiary);
}

.resource-card-meta-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.resource-market-empty {
  grid-column: 1 / -1;
  padding: var(--sl-space-lg);
  text-align: center;
  color: var(--sl-text-tertiary);
}

.empty-state span {
  display: inline-block;
  margin-top: var(--sl-space-sm);
  color: var(--sl-text-secondary);
}

/* 加载更多:横贯整行,继续往下拉取下一页 */
.resource-market-load-more {
  width: 100%;
}

@media (max-width: 900px) {
  /* 窄屏放不下就退回库的纵向堆叠:筛选一行,搜索框一行 */
  .resource-market-toolbar :deep(.cmz-tab-bar.cmz-tab-bar--level-2) {
    flex-direction: column;
    align-items: stretch;
  }

  .resource-market-toolbar :deep(.cmz-tab-bar.cmz-tab-bar--level-2 .cmz-tab-bar__extra) {
    margin: var(--sl-space-xs) 0 0;
  }

  .resource-market-list {
    grid-template-columns: 1fr;
  }
}
</style>
