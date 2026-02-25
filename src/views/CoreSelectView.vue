<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { ArrowLeft } from "lucide-vue-next";
import SLSpinner from "@components/common/SLSpinner.vue";
import { getServerTypes, getServerDescription } from "@utils/networkapi/fastmirrorapi";

const router = useRouter();

const source = ref<"official" | "msl">("official");
const loading = ref<boolean>(false);
const error = ref<string | null>(null);
const servers = ref<Array<{ name: string; originalName?: string; description?: string }>>([]);

const officialServers = [
  {
    name: "Paper",
    description: "高性能的 Minecraft 服务端，旨在提升服务端的性能",
  },
];

const goBack = () => {
  router.push("/core-download/mode");
};

const switchSource = (newSource: "official" | "msl") => {
  source.value = newSource;
  if (newSource === "msl") {
    loadMSLServers();
  } else {
    servers.value = officialServers;
  }
};

const loadMSLServers = async () => {
  try {
    loading.value = true;
    error.value = null;
    const serverTypes = await getServerTypes();

    // 获取每个服务端的简介，并将首字母大写
    const serversWithDesc = await Promise.all(
      serverTypes.map(async (name) => {
        try {
          const description = await getServerDescription(name);
          // 首字母大写
          const displayName = name.charAt(0).toUpperCase() + name.slice(1);
          return { name: displayName, originalName: name, description };
        } catch {
          const displayName = name.charAt(0).toUpperCase() + name.slice(1);
          return { name: displayName, originalName: name };
        }
      }),
    );

    servers.value = serversWithDesc;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "加载失败";
  } finally {
    loading.value = false;
  }
};

const selectServer = (server: { name: string; originalName?: string }) => {
  // 使用原始名称（小写）传递给 API
  const serverName = server.originalName || server.name.toLowerCase();
  router.push({
    path: `/core-download/versions/${serverName}`,
    query: { source: source.value },
  });
};

onMounted(() => {
  servers.value = officialServers;
});
</script>

<template>
  <div class="core-select-view animate-fade-in-up">
    <button class="core-back-button" @click="goBack">
      <ArrowLeft :size="20" />
      <span>返回上一页</span>
    </button>

    <div class="core-container">
      <h1 class="core-title">选择服务器核心</h1>
      <p class="core-subtitle">选择适合你的 Minecraft 服务器核心</p>

      <div class="source-switch">
        <button
          class="source-button"
          :class="{ active: source === 'official' }"
          @click="switchSource('official')"
        >
          官方源
        </button>
        <button
          class="source-button"
          :class="{ active: source === 'msl' }"
          @click="switchSource('msl')"
        >
          MSL镜像站
        </button>
      </div>

      <div v-if="loading" class="core-loading">
        <SLSpinner />
        <p>正在加载服务端列表...</p>
      </div>

      <div v-else-if="error" class="core-error">
        <p>{{ error }}</p>
        <button class="core-retry-button" @click="loadMSLServers">重试</button>
      </div>

      <div v-else class="core-list">
        <button
          v-for="server in servers"
          :key="server.originalName || server.name"
          class="core-card"
          @click="selectServer(server)"
        >
          <div class="core-info">
            <h2 class="core-name">{{ server.name }}</h2>
            <p v-if="server.description" class="core-description">{{ server.description }}</p>
          </div>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.core-select-view {
  padding: var(--sl-space-xl);
  max-width: 1000px;
  margin: 0 auto;
}

.core-back-button {
  display: inline-flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: var(--sl-space-sm) var(--sl-space-md);
  margin-bottom: var(--sl-space-lg);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  color: var(--sl-text-secondary);
  font-size: 0.875rem;
  cursor: pointer;
  transition:
    background-color var(--sl-transition-fast) ease,
    border-color var(--sl-transition-fast) ease,
    color var(--sl-transition-fast) ease;
}

.core-back-button:hover {
  background: var(--sl-surface-hover);
  border-color: var(--sl-primary);
  color: var(--sl-primary);
}

.core-container {
  text-align: center;
}

.core-title {
  font-size: 1.75rem;
  font-weight: 700;
  color: var(--sl-text-primary);
  margin: 0 0 var(--sl-space-sm) 0;
}

.core-subtitle {
  font-size: 1rem;
  color: var(--sl-text-secondary);
  margin: 0 0 var(--sl-space-xl) 0;
}

.source-switch {
  display: flex;
  justify-content: center;
  gap: var(--sl-space-sm);
  margin-bottom: var(--sl-space-2xl);
}

.source-button {
  padding: var(--sl-space-sm) var(--sl-space-xl);
  background: var(--sl-surface);
  border: 2px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  color: var(--sl-text-secondary);
  font-size: 0.9375rem;
  font-weight: 500;
  cursor: pointer;
  transition:
    background-color var(--sl-transition-fast) ease,
    border-color var(--sl-transition-fast) ease,
    color var(--sl-transition-fast) ease;
}

.source-button:hover {
  border-color: var(--sl-primary);
  color: var(--sl-primary);
}

.source-button.active {
  background: var(--sl-primary);
  border-color: var(--sl-primary);
  color: var(--sl-text-inverse);
}

.core-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sl-space-md);
  padding: var(--sl-space-2xl);
  color: var(--sl-text-secondary);
}

.core-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sl-space-md);
  padding: var(--sl-space-2xl);
  color: var(--sl-error);
}

.core-retry-button {
  padding: var(--sl-space-sm) var(--sl-space-lg);
  background: var(--sl-primary);
  color: var(--sl-text-inverse);
  border: none;
  border-radius: var(--sl-radius-md);
  font-size: 0.875rem;
  cursor: pointer;
  transition: background-color var(--sl-transition-fast) ease;
}

.core-retry-button:hover {
  background: var(--sl-primary-dark);
}

.core-list {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}

.core-card {
  display: flex;
  align-items: center;
  gap: var(--sl-space-lg);
  padding: var(--sl-space-lg);
  background: var(--sl-surface);
  border: 2px solid var(--sl-border);
  border-radius: var(--sl-radius-lg);
  cursor: pointer;
  text-align: left;
  transition:
    transform 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94),
    border-color 0.3s ease,
    box-shadow 0.3s ease;
  will-change: transform, box-shadow;
}

.core-card:hover {
  transform: translateY(-4px) scale(1.01);
  border-color: var(--sl-primary);
  box-shadow:
    0 8px 24px rgba(14, 165, 233, 0.15),
    0 0 0 1px var(--sl-primary),
    0 0 20px rgba(14, 165, 233, 0.25);
}

.core-card:active {
  transform: translateY(-2px) scale(1.005);
}

.core-info {
  flex: 1;
  min-width: 0;
}

.core-name {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 var(--sl-space-xs) 0;
}

.core-description {
  font-size: 0.875rem;
  color: var(--sl-text-secondary);
  line-height: 1.5;
  margin: 0;
}
</style>
