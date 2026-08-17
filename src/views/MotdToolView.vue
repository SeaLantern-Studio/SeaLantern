<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { Server } from "lucide-vue-next";
import { i18n } from "@language";
import { useServerStore } from "@stores/serverStore";
import { configApi } from "@api/config";
import MotdEditorBody from "@components/config/MotdEditorBody.vue";
import { DEFAULT_MOTD } from "@utils/motdCodes";

/**
 *
 * - 独立的侧栏入口 / 路由
 * - 选定服务器后读取其 server.properties 的 motd 进行编辑
 * - 一键将结果写回已管理服务器（apply to server）
 */
const serverStore = useServerStore();

const selectedServerId = ref<string | null>(
  serverStore.currentServerId ?? serverStore.servers[0]?.id ?? null,
);
const loadedMotd = ref("");
const statusMsg = ref<string | null>(null);
const applying = ref(false);

let statusTimer: ReturnType<typeof setTimeout> | null = null;

const serverOptions = computed(() =>
  serverStore.servers.map((s) => ({ label: s.name, value: s.id })),
);

const selectedServer = computed(() =>
  selectedServerId.value ? serverStore.getServerById(selectedServerId.value) : null,
);

function setStatus(message: string) {
  statusMsg.value = message;
  if (statusTimer) clearTimeout(statusTimer);
  statusTimer = setTimeout(() => {
    statusMsg.value = null;
  }, 2500);
}

async function loadMotd() {
  const server = selectedServer.value;
  if (!server) {
    loadedMotd.value = DEFAULT_MOTD;
    return;
  }
  try {
    const data = await configApi.readServerProperties(server.path);
    loadedMotd.value = data.raw["motd"] ?? DEFAULT_MOTD;
    statusMsg.value = null;
  } catch {
    loadedMotd.value = DEFAULT_MOTD;
    setStatus(i18n.t("config.motd.load_failed"));
  }
}

onMounted(loadMotd);
watch(selectedServerId, loadMotd);

async function onApply(value: string) {
  const server = selectedServer.value;
  if (!server) {
    setStatus(i18n.t("common.message_select_server"));
    return;
  }
  applying.value = true;
  try {
    await configApi.writeServerProperties(server.path, { motd: value });
    setStatus(i18n.t("config.motd.saved"));
  } catch {
    setStatus(i18n.t("config.motd.save_failed"));
  } finally {
    applying.value = false;
  }
}
</script>

<template>
  <div class="motd-tool-view">
    <header class="motd-tool-view__header">
      <h1 class="motd-tool-view__title">{{ i18n.t("config.motd.title") }}</h1>
      <p class="motd-tool-view__desc">{{ i18n.t("config.motd.tool_desc") }}</p>
    </header>

    <div class="motd-tool-view__bar">
      <cmz-select
        v-model="selectedServerId"
        :options="serverOptions"
        :icon="Server"
        :placeholder="i18n.t('common.select_server')"
        variant="server"
        dropdown-align="right"
        class="motd-tool-view__select"
      />
    </div>

    <MotdEditorBody
      :key="selectedServerId ?? 'none'"
      embedded
      :modelValue="loadedMotd"
      :server-name="selectedServer?.name ?? 'Minecraft Server'"
      :apply-text="i18n.t('config.motd.apply_to_server')"
      :disabled="applying"
      @apply="onApply"
    />

    <p v-if="statusMsg" class="motd-tool-view__status">{{ statusMsg }}</p>
  </div>
</template>

<style scoped>
.motd-tool-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
  max-width: 1000px;
  margin: 0 auto;
}

.motd-tool-view__header {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
}

.motd-tool-view__title {
  margin: 0;
  color: var(--sl-text-primary);
  font-size: 1.25rem;
  font-weight: 600;
}

.motd-tool-view__desc {
  margin: 0;
  color: var(--sl-text-secondary);
  font-size: 0.875rem;
  line-height: 1.5;
}

.motd-tool-view__bar {
  display: flex;
  align-items: center;
}

.motd-tool-view__select {
  min-width: 220px;
  max-width: 320px;
}

.motd-tool-view__status {
  min-height: 1.2em;
  margin: 0;
  color: var(--sl-text-secondary);
  font-size: 0.8125rem;
}
</style>
