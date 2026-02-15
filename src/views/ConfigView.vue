<script setup lang="ts">
import { ref, onMounted, computed, watch } from "vue";
import { useRoute } from "vue-router";
import SLCard from "../components/common/SLCard.vue";
import SLButton from "../components/common/SLButton.vue";
import SLInput from "../components/common/SLInput.vue";
import SLSwitch from "../components/common/SLSwitch.vue";
import SLSelect from "../components/common/SLSelect.vue";
import SLBadge from "../components/common/SLBadge.vue";
import SLSpinner from "../components/common/SLSpinner.vue";
import { configApi, type ConfigEntry } from "../api/config";
import { useServerStore } from "../stores/serverStore";

const route = useRoute();
const store = useServerStore();

const entries = ref<ConfigEntry[]>([]);
const editValues = ref<Record<string, string>>({});
const loading = ref(false);
const saving = ref(false);
const error = ref<string | null>(null);
const successMsg = ref<string | null>(null);
const searchQuery = ref("");
const activeCategory = ref("all");
const selectedServerId = ref("");

const serverOptions = computed(() => store.servers.map((s) => ({ label: s.name, value: s.id })));

const serverPath = computed(() => {
  const server = store.servers.find((s) => s.id === selectedServerId.value);
  return server?.path || "";
});

const categories = computed(() => {
  const cats = new Set(entries.value.map((e) => e.category));
  return ["all", ...Array.from(cats)];
});

const categoryLabels: Record<string, string> = {
  all: "全部",
  network: "网络",
  player: "玩家",
  game: "游戏",
  world: "世界",
  performance: "性能",
  display: "显示",
  other: "其他",
};

const filteredEntries = computed(() => {
  return entries.value.filter((e) => {
    const matchCat = activeCategory.value === "all" || e.category === activeCategory.value;
    const matchSearch =
      !searchQuery.value ||
      e.key.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      e.description.toLowerCase().includes(searchQuery.value.toLowerCase());
    return matchCat && matchSearch;
  });
});

onMounted(async () => {
  await store.refreshList();
  const routeId = route.params.id as string;
  if (routeId) {
    selectedServerId.value = routeId;
  } else if (store.currentServerId) {
    selectedServerId.value = store.currentServerId;
  } else if (store.servers.length > 0) {
    selectedServerId.value = store.servers[0].id;
  }
});

watch(selectedServerId, async () => {
  if (selectedServerId.value) {
    await loadProperties();
  }
});

async function loadProperties() {
  if (!serverPath.value) return;
  loading.value = true;
  error.value = null;
  try {
    const result = await configApi.readServerProperties(serverPath.value);
    entries.value = result.entries;
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
    successMsg.value = "配置已保存";
    setTimeout(() => (successMsg.value = null), 3000);
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}

function updateValue(key: string, value: string | boolean) {
  editValues.value[key] = String(value);
}

function getBoolValue(key: string): boolean {
  return editValues.value[key] === "true";
}

function getServerName(): string {
  const s = store.servers.find((s) => s.id === selectedServerId.value);
  return s ? s.name : "";
}
</script>

<template>
  <div class="config-view animate-fade-in-up">
    <!-- Server Selector -->
    <div class="config-header">
      <div class="server-picker">
        <SLSelect
          label="选择服务器"
          :options="serverOptions"
          v-model="selectedServerId"
          placeholder="选择要编辑配置的服务器"
        />
      </div>
      <div v-if="selectedServerId" class="server-path-display text-mono text-caption">
        {{ serverPath }}/server.properties
      </div>
    </div>

    <div v-if="!selectedServerId" class="empty-state">
      <p class="text-body">请选择一个服务器来编辑配置</p>
    </div>

    <template v-else>
      <div v-if="error" class="error-banner">
        <span>{{ error }}</span>
        <button class="banner-close" @click="error = null">x</button>
      </div>
      <div v-if="successMsg" class="success-banner">
        <span>{{ successMsg }}</span>
      </div>

      <div class="config-toolbar">
        <div class="toolbar-left">
          <SLInput placeholder="搜索配置项..." v-model="searchQuery" />
        </div>
        <div class="toolbar-right">
          <SLButton variant="secondary" size="sm" @click="loadProperties">刷新</SLButton>
          <SLButton variant="primary" size="sm" :loading="saving" @click="saveProperties"
            >保存配置</SLButton
          >
        </div>
      </div>

      <div class="category-tabs">
        <button
          v-for="cat in categories"
          :key="cat"
          class="category-tab"
          :class="{ active: activeCategory === cat }"
          @click="activeCategory = cat"
        >
          {{ categoryLabels[cat] || cat }}
        </button>
      </div>

      <div v-if="loading" class="loading-state">
        <SLSpinner />
        <span>加载配置中...</span>
      </div>

      <div v-else class="config-entries">
        <div v-for="entry in filteredEntries" :key="entry.key" class="config-entry glass-card">
          <div class="entry-header">
            <div class="entry-key-row">
              <span class="entry-key text-mono">{{ entry.key }}</span>
              <SLBadge :text="categoryLabels[entry.category] || entry.category" variant="neutral" />
            </div>
            <p v-if="entry.description" class="entry-desc text-caption">{{ entry.description }}</p>
          </div>
          <div class="entry-control">
            <SLSwitch
              v-if="entry.value_type === 'boolean'"
              :modelValue="getBoolValue(entry.key)"
              @update:modelValue="updateValue(entry.key, $event)"
            />
            <SLSelect
              v-else-if="entry.key === 'gamemode'"
              :modelValue="editValues[entry.key]"
              :options="[
                { label: '生存', value: 'survival' },
                { label: '创造', value: 'creative' },
                { label: '冒险', value: 'adventure' },
                { label: '旁观', value: 'spectator' },
              ]"
              @update:modelValue="updateValue(entry.key, $event as string)"
            />
            <SLSelect
              v-else-if="entry.key === 'difficulty'"
              :modelValue="editValues[entry.key]"
              :options="[
                { label: '和平', value: 'peaceful' },
                { label: '简单', value: 'easy' },
                { label: '普通', value: 'normal' },
                { label: '困难', value: 'hard' },
              ]"
              @update:modelValue="updateValue(entry.key, $event as string)"
            />
            <SLInput
              v-else
              :modelValue="editValues[entry.key]"
              :type="entry.value_type === 'number' ? 'number' : 'text'"
              :placeholder="entry.default_value"
              @update:modelValue="updateValue(entry.key, $event)"
            />
          </div>
        </div>
        <div v-if="filteredEntries.length === 0 && !loading" class="empty-state">
          <p class="text-caption">没有找到配置文件，请先启动一次服务器以生成 server.properties</p>
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
.server-picker {
  max-width: 400px;
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
}
.config-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-md);
}
.toolbar-left {
  flex: 1;
  max-width: 360px;
}
.toolbar-right {
  display: flex;
  gap: var(--sl-space-xs);
}
.category-tabs {
  display: flex;
  gap: 2px;
  background: var(--sl-bg-secondary);
  border-radius: var(--sl-radius-md);
  padding: 3px;
  width: fit-content;
  flex-wrap: wrap;
}
.category-tab {
  padding: 6px 14px;
  border-radius: var(--sl-radius-sm);
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-text-secondary);
  transition: all var(--sl-transition-fast);
}
.category-tab.active {
  background: var(--sl-surface);
  color: var(--sl-primary);
  box-shadow: var(--sl-shadow-sm);
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
