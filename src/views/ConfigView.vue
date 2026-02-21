<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, onActivated } from "vue";
import { useRoute } from "vue-router";
import SLSpinner from "../components/common/SLSpinner.vue";
import SLSwitch from "../components/common/SLSwitch.vue";
import SLInput from "../components/common/SLInput.vue";
import SLSelect from "../components/common/SLSelect.vue";
import { configApi } from "../api/config";
import type { ConfigEntry as ConfigEntryType } from "../api/config";
import { useServerStore } from "../stores/serverStore";
import { i18n } from "../language";

// 导入拆分后的组件
import ConfigToolbar from "../components/config/ConfigToolbar.vue";
import ConfigCategories from "../components/config/ConfigCategories.vue";
import ConfigEntry from "../components/config/ConfigEntry.vue";

const route = useRoute();
const store = useServerStore();

const entries = ref<ConfigEntryType[]>([]);
const editValues = ref<Record<string, string>>({});
const loading = ref(false);
const saving = ref(false);
const error = ref<string | null>(null);
const successMsg = ref<string | null>(null);
const searchQuery = ref("");
const activeCategory = ref("all");
const serverPath = computed(() => {
  const server = store.servers.find((s) => s.id === store.currentServerId);
  return server?.path || "";
});

// 自动保存相关
const autoSaveDebounceTimer = ref<ReturnType<typeof setTimeout> | null>(null);
const AUTO_SAVE_DELAY = 1000; // 1秒防抖延迟

const currentServerId = computed(() => store.currentServerId);

const categories = computed(() => {
  const cats = new Set(entries.value.map((e) => e.category));
  return ["all", ...Array.from(cats)];
});

const filteredEntries = computed(() => {
  return entries.value.filter((e: ConfigEntryType) => {
    const matchCat = activeCategory.value === "all" || e.category === activeCategory.value;
    const matchSearch =
      !searchQuery.value ||
      e.key.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      (e.description ?? "").toLowerCase().includes(searchQuery.value.toLowerCase());
    return matchCat && matchSearch;
  });
});

onMounted(async () => {
  await store.refreshList();
  const routeId = route.params.id as string;
  if (routeId) {
    store.setCurrentServer(routeId);
  } else if (!store.currentServerId && store.servers.length > 0) {
    store.setCurrentServer(store.servers[0].id);
  }
  await loadProperties();
});

onUnmounted(() => {
  // 清理防抖计时器
  if (autoSaveDebounceTimer.value) {
    clearTimeout(autoSaveDebounceTimer.value);
  }
});

onActivated(async () => {
  // 当组件被激活时自动刷新配置
  await loadProperties();
});

watch(
  () => store.currentServerId,
  async () => {
    if (store.currentServerId) {
      await loadProperties();
    }
  },
);

async function loadProperties() {
  if (!serverPath.value) return;
  loading.value = true;
  error.value = null;
  try {
    const result = await configApi.readServerProperties(serverPath.value);
    entries.value = result.entries as ConfigEntryType[];
    editValues.value = { ...result.raw };
  } catch (e) {
    error.value = String(e);
    entries.value = [];
    editValues.value = {};
  } finally {
    loading.value = false;
  }
}

async function saveProperties() {
  if (!serverPath.value) return;
  saving.value = true;
  error.value = null;
  successMsg.value = null;
  try {
    await configApi.writeServerProperties(serverPath.value, editValues.value);
    successMsg.value = i18n.t("common.config_saved");
    setTimeout(() => (successMsg.value = null), 3000);
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}

function updateValue(key: string, value: string | boolean) {
  editValues.value[key] = String(value);

  // 启动自动保存防抖
  if (autoSaveDebounceTimer.value) {
    clearTimeout(autoSaveDebounceTimer.value);
  }

  autoSaveDebounceTimer.value = window.setTimeout(() => {
    autoSaveProperties();
  }, AUTO_SAVE_DELAY);
}

function autoSaveProperties() {
  if (!serverPath.value) return;

  saving.value = true;
  error.value = null;
  successMsg.value = null;

  configApi
    .writeServerProperties(serverPath.value, editValues.value)
    .then(() => {
      successMsg.value = i18n.t("config.saved");
      setTimeout(() => (successMsg.value = null), 3000);
      return Promise.resolve();
    })
    .catch((e) => {
      error.value = String(e);
      return Promise.resolve();
    })
    .finally(() => {
      saving.value = false;
    });
}

function handleCategoryChange(category: string) {
  activeCategory.value = category;
  // 滚动到顶部
  window.scrollTo({ top: 0, behavior: "smooth" });
}

function handleSearchUpdate(value: string) {
  searchQuery.value = value;
}

const gamemodeOptions = computed(() => [
  { label: i18n.t("config.gamemode.survival"), value: "survival" },
  { label: i18n.t("config.gamemode.creative"), value: "creative" },
  { label: i18n.t("config.gamemode.adventure"), value: "adventure" },
  { label: i18n.t("config.gamemode.spectator"), value: "spectator" },
]);

const difficultyOptions = computed(() => [
  { label: i18n.t("config.difficulty.peaceful"), value: "peaceful" },
  { label: i18n.t("config.difficulty.easy"), value: "easy" },
  { label: i18n.t("config.difficulty.normal"), value: "normal" },
  { label: i18n.t("config.difficulty.hard"), value: "hard" },
]);
</script>

<template>
  <div class="config-view animate-fade-in-up">
    <!-- 服务器配置编辑 -->
    <div class="config-header">
      <div class="server-path-display text-mono text-caption">
        {{ serverPath }}/server.properties
      </div>
    </div>

    <div v-if="!currentServerId" class="empty-state">
      <p class="text-body">{{ i18n.t("config.no_server") }}</p>
    </div>

    <template v-else>
      <div v-if="error" class="error-banner">
        <span>{{ error }}</span>
        <button class="banner-close" @click="error = null">x</button>
      </div>
      <div v-if="successMsg" class="success-banner">
        <span>{{ i18n.t("config.saved") }}</span>
      </div>

      <!-- 分类选择和搜索 -->
      <ConfigCategories
        :categories="categories"
        :activeCategory="activeCategory"
        :searchQuery="searchQuery"
        @updateCategory="handleCategoryChange"
        @updateSearch="handleSearchUpdate"
      />

      <div v-if="loading" class="loading-state">
        <SLSpinner size="lg" />
        <span>{{ i18n.t("config.loading") }}</span>
      </div>

      <div v-else class="config-entries">
        <div v-for="entry in filteredEntries" :key="entry.key" class="config-entry glass-card">
          <div class="entry-header">
            <div class="entry-key-row">
              <span class="entry-key text-mono">{{ entry.key }}</span>
            </div>
            <p v-if="i18n.t(`config.properties.${entry.key}`)" class="entry-desc text-caption">
              {{ i18n.t(`config.properties.${entry.key}`) }}
            </p>
          </div>
          <div class="entry-control">
            <template
              v-if="
                entry.value_type === 'boolean' ||
                editValues[entry.key] === 'true' ||
                editValues[entry.key] === 'false'
              "
            >
              <SLSwitch
                :modelValue="editValues[entry.key] === 'true'"
                @update:modelValue="updateValue(entry.key, $event)"
              />
            </template>
            <template v-else-if="entry.key === 'gamemode'">
              <SLSelect
                :modelValue="editValues[entry.key]"
                :options="gamemodeOptions"
                @update:modelValue="updateValue(entry.key, $event)"
              />
            </template>
            <template v-else-if="entry.key === 'difficulty'">
              <SLSelect
                :modelValue="editValues[entry.key]"
                :options="difficultyOptions"
                @update:modelValue="updateValue(entry.key, $event)"
              />
            </template>
            <template v-else>
              <SLInput
                :modelValue="editValues[entry.key]"
                :type="entry.value_type === 'number' ? 'number' : 'text'"
                :placeholder="entry.default_value"
                @update:modelValue="updateValue(entry.key, $event)"
              />
            </template>
          </div>
        </div>
        <div v-if="filteredEntries.length === 0 && !loading" class="empty-state">
          <p class="text-caption">{{ i18n.t("config.no_config") }}</p>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.config-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}
.config-header {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}
.server-path-display {
  color: var(--sl-text-tertiary);
  font-size: 0.75rem;
}
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--sl-space-2xl);
}
.error-banner,
.success-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-radius: var(--sl-radius-md);
  font-size: 0.875rem;
}
.error-banner {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: var(--sl-error);
}
.success-banner {
  background: rgba(34, 197, 94, 0.1);
  border: 1px solid rgba(34, 197, 94, 0.2);
  color: var(--sl-success);
}
.banner-close {
  font-weight: 600;
  background: none;
  border: none;
  cursor: pointer;
  color: inherit;
}
.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-2xl);
  color: var(--sl-text-tertiary);
}
.config-entries {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}
.config-entry {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sl-space-md);
  gap: var(--sl-space-lg);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  transition: all var(--sl-transition-fast);
}
.config-entry:hover {
  border-color: var(--sl-border);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}
.entry-header {
  flex: 1;
  min-width: 0;
}
.entry-key-row {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}
.entry-key {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}
.entry-desc {
  margin-top: 2px;
}
.entry-control {
  flex-shrink: 0;
  min-width: 200px;
}
</style>
