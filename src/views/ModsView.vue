<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute } from "vue-router";
import SLCard from "../components/common/SLCard.vue";
import SLInput from "../components/common/SLInput.vue";
import SLButton from "../components/common/SLButton.vue";
import SLSelect from "../components/common/SLSelect.vue";
import { modsApi, type ModInfo, type ModLoader, type ModProjectType } from "../api/mods";
import { useServerStore } from "../stores/serverStore";
import { i18n } from "../locales";

const route = useRoute();
const serverStore = useServerStore();

const query = ref("");
const gameVersion = ref("1.20.1");
const projectType = ref<ModProjectType>("mod");
const loader = ref<ModLoader>("fabric");
const loading = ref(false);
const error = ref<string | null>(null);
const mods = ref<ModInfo[]>([]);
const installingId = ref<string | null>(null);
const successMessage = ref<string | null>(null);
const currentPage = ref(1);
const pageSize = ref("10");
const total = ref(0);

function formatDownloads(value: number): string {
  return new Intl.NumberFormat().format(value);
}

const totalPages = computed(() => Math.max(1, Math.ceil(total.value / Number(pageSize.value))));
const pageStart = computed(() => (total.value === 0 ? 0 : (currentPage.value - 1) * Number(pageSize.value) + 1));
const pageEnd = computed(() => Math.min(currentPage.value * Number(pageSize.value), total.value));

const projectTypeOptions = [
  { label: i18n.t("mods.project_type_mod"), value: "mod" },
  { label: i18n.t("mods.project_type_plugin"), value: "plugin" },
];

const loaderOptions = computed(() =>
  projectType.value === "plugin"
    ? [{ label: "Paper", value: "paper" }]
    : [
        { label: "Fabric", value: "fabric" },
        { label: "Forge", value: "forge" },
        { label: "Quilt", value: "quilt" },
        { label: "NeoForge", value: "neoforge" },
      ],
);

const pageSizeOptions = [
  { label: "10", value: "10" },
  { label: "20", value: "20" },
  { label: "30", value: "30" },
];

const selectedServerId = computed(() => {
  const routeServerId = route.params.id;
  if (typeof routeServerId === "string" && routeServerId.length > 0) {
    return routeServerId;
  }
  return serverStore.currentServerId;
});

const selectedServer = computed(() => {
  const id = selectedServerId.value;
  if (!id) return null;
  return serverStore.servers.find((server) => server.id === id) ?? null;
});

watch(
  () => route.params.id,
  (id) => {
    if (typeof id === "string" && id.length > 0) {
      serverStore.setCurrentServer(id);
    }
  },
  { immediate: true },
);

onMounted(async () => {
  await serverStore.refreshList();
});


watch(projectType, async (nextType) => {
  loader.value = nextType === "plugin" ? "paper" : "fabric";
  if (!query.value.trim()) {
    return;
  }
  await searchMods(1);
});

watch(pageSize, async () => {
  if (!query.value.trim()) {
    return;
  }
  await searchMods(1);
});

async function searchMods(page = 1) {
  if (!query.value.trim()) {
    error.value = i18n.t("mods.enter_keyword");
    return;
  }

  loading.value = true;
  error.value = null;
  successMessage.value = null;

  try {
    const result = await modsApi.searchMods({
      query: query.value.trim(),
      gameVersion: gameVersion.value.trim(),
      loader: loader.value,
      projectType: projectType.value,
      page,
      pageSize: Number(pageSize.value),
    });

    mods.value = result.items;
    total.value = result.total;
    currentPage.value = page;
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

async function goToPage(page: number) {
  if (page < 1 || page > totalPages.value || page === currentPage.value || loading.value) {
    return;
  }
  await searchMods(page);
}

async function installMod(mod: ModInfo) {
  if (!selectedServerId.value) {
    error.value = i18n.t("mods.select_server_first");
    return;
  }

  installingId.value = mod.id;
  error.value = null;
  successMessage.value = null;

  try {
    await modsApi.installMod({
      serverId: selectedServerId.value,
      downloadUrl: mod.download_url,
      fileName: mod.file_name,
      projectId: mod.id,
      gameVersion: gameVersion.value.trim(),
      loader: loader.value,
    });

    successMessage.value = i18n.t("mods.install_with_dependencies_success", { name: mod.name });
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    installingId.value = null;
  }
}
</script>

<template>
  <div class="mods-view animate-fade-in-up">
    <SLCard :title="i18n.t('mods.title')" :subtitle="i18n.t('mods.subtitle')">
      <div class="search-grid">
        <SLInput v-model="query" :label="i18n.t('mods.keyword')" :placeholder="i18n.t('mods.keyword_placeholder')" />
        <SLInput v-model="gameVersion" :label="i18n.t('mods.game_version')" placeholder="1.20.1" />
        <SLSelect
          v-model="projectType"
          :label="i18n.t('mods.project_type')"
          :options="projectTypeOptions"
          maxHeight="220px"
        />
        <SLSelect
          v-model="loader"
          :label="i18n.t('mods.loader')"
          :options="loaderOptions"
          maxHeight="220px"
        />
      </div>

      <div class="search-actions">
        <SLButton variant="primary" :loading="loading" @click="searchMods(1)">{{ i18n.t("mods.search") }}</SLButton>
      </div>

      <p v-if="selectedServer" class="selected-server text-caption">
        {{ i18n.t("mods.target_server") }}: <strong>{{ selectedServer.name }}</strong>
      </p>
      <p v-else class="selected-server text-caption warning">{{ i18n.t("mods.select_server_first") }}</p>

      <p v-if="error" class="text-caption error">{{ error }}</p>
      <p v-if="successMessage" class="text-caption success">{{ successMessage }}</p>

      <div v-if="mods.length > 0" class="mods-list">
        <div v-for="mod in mods" :key="mod.id" class="mod-item">
          <div class="mod-meta">
            <div class="mod-cover">
              <img v-if="mod.icon_url" :src="mod.icon_url" :alt="mod.name" loading="lazy" />
              <div v-else class="mod-cover-placeholder">M</div>
            </div>
            <div class="mod-title-row">
              <h4>{{ mod.name }}</h4>
              <span class="mod-source">{{ mod.source }}</span>
              <span class="mod-downloads">{{ i18n.t("mods.downloads") }} {{ formatDownloads(mod.downloads) }}</span>
            </div>
            <p>{{ mod.summary || i18n.t("mods.no_summary") }}</p>
          </div>
          <SLButton
            variant="secondary"
            :loading="installingId === mod.id"
            :disabled="!selectedServerId"
            @click="installMod(mod)"
          >
            {{ i18n.t("mods.install") }}
          </SLButton>
        </div>
      </div>

      <div v-if="total > 0" class="pagination-row text-caption">
        <span>{{ i18n.t("mods.result_range", { start: pageStart, end: pageEnd, total }) }}</span>
        <div class="pagination-controls">
          <SLButton variant="ghost" :disabled="currentPage <= 1 || loading" @click="goToPage(currentPage - 1)">
            {{ i18n.t("mods.prev_page") }}
          </SLButton>
          <span>{{ currentPage }} / {{ totalPages }}</span>
          <SLButton
           
            variant="ghost"
            :disabled="currentPage >= totalPages || loading"
            @click="goToPage(currentPage + 1)"
          >
            {{ i18n.t("mods.next_page") }}
          </SLButton>
          <SLSelect v-model="pageSize" :options="pageSizeOptions" maxHeight="180px" />
        </div>
      </div>

      <div v-else-if="!loading" class="empty text-caption">{{ i18n.t("mods.empty_hint") }}</div>
    </SLCard>
  </div>
</template>

<style scoped>
.mods-view {
  padding: 0;
}

.search-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.search-actions {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
}

.selected-server {
  margin-top: 10px;
}

.warning {
  color: #f59e0b;
}

.error {
  margin-top: 8px;
  color: #ef4444;
}

.success {
  margin-top: 8px;
  color: #10b981;
}

.mods-list {
  margin-top: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.mod-item {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  align-items: center;
  border: 1px solid var(--sl-border);
  border-radius: 10px;
  padding: 12px;
  background: var(--sl-bg-secondary);
}

.mod-meta {
  min-width: 0;
}

.mod-cover {
  width: 42px;
  height: 42px;
  border-radius: 8px;
  overflow: hidden;
  border: 1px solid var(--sl-border);
  margin-bottom: 8px;
}

.mod-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.mod-cover-placeholder {
  width: 100%;
  height: 100%;
  display: grid;
  place-items: center;
  font-size: 13px;
  color: var(--sl-text-secondary);
}

.mod-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.mod-title-row h4 {
  margin: 0;
}

.mod-source {
  border: 1px solid var(--sl-border);
  border-radius: 999px;
  padding: 1px 8px;
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.mod-downloads {
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.mod-meta p {
  margin: 6px 0 0;
  color: var(--sl-text-secondary);
}

.pagination-row {
  margin-top: 14px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.pagination-controls {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.empty {
  margin-top: 16px;
}

@media (max-width: 1024px) {
  .search-grid {
    grid-template-columns: 1fr;
  }

  .mod-item {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
