<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { Download, Search, Loader2, Check } from "lucide-vue-next";
import SLModal from "@components/common/SLModal.vue";
import SLButton from "@components/common/SLButton.vue";
import SLSpinner from "@components/common/SLSpinner.vue";
import { mslApi, type ServerClassify } from "@api/msl";
import { systemApi } from "@api/system";
import { i18n } from "@language";

interface Props {
  visible: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  close: [];
  select: [path: string, serverName: string, version: string];
}>();

const CATEGORY_LABELS: Record<string, string> = {
  pluginsCore: "server_download.category_plugins",
  pluginsAndModsCore: "server_download.category_hybrid",
  modsCore_Forge: "server_download.category_forge",
  modsCore_Fabric: "server_download.category_fabric",
  vanillaCore: "server_download.category_vanilla",
  bedrockCore: "server_download.category_bedrock",
  proxyCore: "server_download.category_proxy",
};

const CATEGORY_ORDER = [
  "pluginsCore",
  "pluginsAndModsCore",
  "modsCore_Forge",
  "modsCore_Fabric",
  "vanillaCore",
  "bedrockCore",
  "proxyCore",
];

const loading = ref(false);
const error = ref("");
const classify = ref<ServerClassify | null>(null);
const activeCategory = ref("pluginsCore");
const searchQuery = ref("");
const selectedServer = ref("");
const versions = ref<string[]>([]);
const versionsLoading = ref(false);
const selectedVersion = ref("");
const builds = ref<{ build: string; date?: string }[]>([]);
const buildsLoading = ref(false);
const selectedBuild = ref("");
const downloadLoading = ref(false);

const categoryOptions = computed(() => {
  if (!classify.value) return [];
  return CATEGORY_ORDER.filter((cat) => {
    const servers = classify.value![cat as keyof ServerClassify];
    return servers && servers.length > 0;
  }).map((cat) => ({
    value: cat,
    label: i18n.t(CATEGORY_LABELS[cat] || cat),
  }));
});

const filteredServers = computed(() => {
  if (!classify.value) return [];
  const servers = classify.value[activeCategory.value as keyof ServerClassify] || [];
  if (!searchQuery.value) return servers;
  const query = searchQuery.value.toLowerCase();
  return servers.filter((s) => s.toLowerCase().includes(query));
});

function setActiveCategory(cat: string) {
  activeCategory.value = cat;
  selectedServer.value = "";
  selectedVersion.value = "";
  selectedBuild.value = "";
  versions.value = [];
  builds.value = [];
}

async function loadClassify() {
  loading.value = true;
  error.value = "";
  try {
    classify.value = await mslApi.getServerClassify();
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function selectServer(server: string) {
  selectedServer.value = server;
  selectedVersion.value = "";
  selectedBuild.value = "";
  versions.value = [];
  builds.value = [];
  versionsLoading.value = true;
  try {
    versions.value = await mslApi.getServerVersions(server);
    if (versions.value.length > 0) {
      selectedVersion.value = versions.value[0];
      await loadBuilds();
    }
  } catch (e) {
    console.error("Failed to load versions:", e);
  } finally {
    versionsLoading.value = false;
  }
}

async function loadBuilds() {
  if (!selectedServer.value || !selectedVersion.value) return;
  buildsLoading.value = true;
  builds.value = [];
  selectedBuild.value = "";
  try {
    builds.value = await mslApi.getServerBuilds(selectedServer.value, selectedVersion.value);
    if (builds.value.length > 0) {
      selectedBuild.value = builds.value[0].build;
    }
  } catch (e) {
    console.error("Failed to load builds:", e);
  } finally {
    buildsLoading.value = false;
  }
}

async function handleDownload() {
  if (!selectedServer.value || !selectedVersion.value) return;
  downloadLoading.value = true;
  try {
    const result = await mslApi.getServerDownloadUrl(
      selectedServer.value,
      selectedVersion.value,
      selectedBuild.value || "latest",
    );
    const downloadResult = await systemApi.downloadServerCore({
      url: result.url,
      serverName: selectedServer.value,
      version: selectedVersion.value,
    });
    if (downloadResult) {
      emit("select", downloadResult, selectedServer.value, selectedVersion.value);
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    downloadLoading.value = false;
  }
}

function handleClose() {
  emit("close");
}

watch(
  () => props.visible,
  (val) => {
    if (val && !classify.value) {
      loadClassify();
    }
  },
);

watch(selectedVersion, () => {
  loadBuilds();
});

onMounted(() => {
  if (props.visible) {
    loadClassify();
  }
});
</script>

<template>
  <SLModal
    :visible="visible"
    :title="i18n.t('server_download.title')"
    width="560px"
    @close="handleClose"
  >
    <div class="server-download-modal">
      <div v-if="loading" class="loading-state">
        <SLSpinner />
        <span>{{ i18n.t("common.loading") }}</span>
      </div>

      <div v-else-if="error" class="error-state">
        <span>{{ error }}</span>
        <SLButton variant="primary" size="sm" @click="loadClassify">
          {{ i18n.t("market.retry") }}
        </SLButton>
      </div>

      <template v-else-if="classify">
        <div class="select-row">
          <div class="select-group">
            <label>{{ i18n.t("server_download.category") }}</label>
            <select v-model="activeCategory">
              <option v-for="opt in categoryOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </div>

          <div class="select-group">
            <label>{{ i18n.t("server_download.mc_version") }}</label>
            <select v-model="selectedVersion" :disabled="!selectedServer || versionsLoading">
              <option v-for="v in versions" :key="v" :value="v">{{ v }}</option>
            </select>
            <Loader2 v-if="versionsLoading" :size="14" class="spin-icon" />
          </div>

          <div class="select-group" v-if="builds.length > 0">
            <label>{{ i18n.t("server_download.build_version") }}</label>
            <select v-model="selectedBuild" :disabled="buildsLoading">
              <option v-for="b in builds" :key="b.build" :value="b.build">
                {{ b.build }}{{ b.date ? ` (${b.date})` : "" }}
              </option>
            </select>
            <Loader2 v-if="buildsLoading" :size="14" class="spin-icon" />
          </div>
        </div>

        <div class="search-bar">
          <Search :size="16" class="search-icon" />
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="i18n.t('common.search')"
            class="search-input"
          />
        </div>

        <div class="server-list">
          <div class="list-header">{{ i18n.t("server_download.select_server") }}</div>
          <div class="list-content">
            <button
              v-for="server in filteredServers"
              :key="server"
              class="server-item"
              :class="{ selected: selectedServer === server }"
              @click="selectServer(server)"
            >
              <span class="server-name">{{ server }}</span>
              <Check v-if="selectedServer === server" :size="16" class="check-icon" />
            </button>
            <div v-if="filteredServers.length === 0" class="empty-hint">
              {{ i18n.t("common.no_match") }}
            </div>
          </div>
        </div>

        <div class="modal-footer">
          <SLButton variant="secondary" @click="handleClose">
            {{ i18n.t("common.cancel") }}
          </SLButton>
          <SLButton
            variant="primary"
            :loading="downloadLoading"
            :disabled="!selectedServer || !selectedVersion"
            @click="handleDownload"
          >
            <template #icon>
              <Download :size="16" />
            </template>
            {{ i18n.t("server_download.download") }}
          </SLButton>
        </div>
      </template>
    </div>
  </SLModal>
</template>

<style scoped>
.server-download-modal {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
  min-height: 360px;
}

.loading-state,
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-xl);
  color: var(--sl-text-tertiary);
}

.error-state {
  color: var(--sl-error);
}

.select-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: var(--sl-space-sm);
}

.select-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
  position: relative;
}

.select-group label {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--sl-text-tertiary);
}

.select-group select {
  padding: 8px 32px 8px 12px;
  font-size: 0.875rem;
  color: var(--sl-text-primary);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  cursor: pointer;
  outline: none;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%2364748b' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
}

.select-group select:focus {
  border-color: var(--sl-primary);
}

.select-group select:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.spin-icon {
  position: absolute;
  right: 28px;
  bottom: 10px;
  color: var(--sl-primary);
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.search-bar {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  padding: 8px 12px;
  background: var(--sl-bg-secondary);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
}

.search-icon {
  color: var(--sl-text-tertiary);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  border: none;
  background: transparent;
  color: var(--sl-text-primary);
  font-size: 0.875rem;
  outline: none;
}

.search-input::placeholder {
  color: var(--sl-text-tertiary);
}

.server-list {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  overflow: hidden;
  flex: 1;
  min-height: 160px;
}

.list-header {
  padding: 8px 12px;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--sl-text-tertiary);
  background: var(--sl-bg-secondary);
  border-bottom: 1px solid var(--sl-border);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.list-content {
  flex: 1;
  overflow-y: auto;
  max-height: 200px;
}

.server-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 10px 12px;
  font-size: 0.875rem;
  color: var(--sl-text-secondary);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all 0.15s ease;
  text-align: left;
}

.server-item:hover {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
}

.server-item.selected {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
  font-weight: 500;
}

.server-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.check-icon {
  flex-shrink: 0;
  color: var(--sl-primary);
}

.empty-hint {
  padding: var(--sl-space-lg);
  text-align: center;
  color: var(--sl-text-tertiary);
  font-size: 0.875rem;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--sl-space-sm);
  padding-top: var(--sl-space-sm);
  border-top: 1px solid var(--sl-border-light);
}
</style>
