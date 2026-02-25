<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { useRouter, useRoute } from "vue-router";
import { ArrowLeft } from "lucide-vue-next";
import SLSpinner from "@components/common/SLSpinner.vue";
import { getPaperVersions } from "@utils/networkapi/paperapi";
import { getServerVersions } from "@utils/networkapi/mslapi.ts";

const router = useRouter();
const route = useRoute();

const loading = ref<boolean>(true);
const error = ref<string | null>(null);
const versions = ref<string[]>([]);
const serverName = ref<string>("");
const source = ref<"official" | "msl">("official");

const goBack = () => {
  router.push("/core-download/quick");
};

const selectVersion = (version: string) => {
  router.push({
    path: `/core-download/builds/${serverName.value}/${version}`,
    query: { source: source.value },
  });
};

const loadVersions = async () => {
  try {
    loading.value = true;
    error.value = null;

    console.log("loadVersions - source:", source.value, "serverName:", serverName.value);

    if (source.value === "official") {
      versions.value = await getPaperVersions();
    } else {
      versions.value = await getServerVersions(serverName.value);
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : "加载失败";
    console.error("loadVersions error:", err);
  } finally {
    loading.value = false;
  }
};

onMounted(async () => {
  const serverParam = route.params.coreName as string;
  const sourceParam = route.query.source as string;

  console.log("CoreVersionSelectView - serverName:", serverParam, "source:", sourceParam);

  if (serverParam) {
    serverName.value = serverParam;
  }

  if (sourceParam === "msl") {
    source.value = "msl";
  } else {
    source.value = "official";
  }

  await loadVersions();
});

// 监听路由参数变化，强制重新加载
watch(
  () => [route.params.coreName, route.query.source],
  async ([newServerName, newSource]) => {
    console.log("Route changed - serverName:", newServerName, "source:", newSource);

    if (newServerName && typeof newServerName === "string") {
      serverName.value = newServerName;
    }

    if (newSource === "msl") {
      source.value = "msl";
    } else {
      source.value = "official";
    }

    await loadVersions();
  },
);
</script>

<template>
  <div :key="`${serverName}-${source}`" class="version-view">
    <button class="version-back-button" @click="goBack">
      <ArrowLeft :size="20" />
      <span>返回上一页</span>
    </button>

    <div class="version-container">
      <h1 class="version-title">选择 {{ serverName }} 版本</h1>
      <p class="version-subtitle">选择你需要的 Minecraft 版本</p>

      <div v-if="loading" class="version-loading">
        <SLSpinner />
        <p>正在加载版本列表...</p>
      </div>

      <div v-else-if="error" class="version-error">
        <p>{{ error }}</p>
        <button class="version-retry-button" @click="loadVersions">重试</button>
      </div>

      <div v-else class="version-list">
        <button
          v-for="version in versions"
          :key="version"
          class="version-item"
          @click="selectVersion(version)"
        >
          <span class="version-text">{{ version }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.version-view {
  padding: var(--sl-space-xl);
  max-width: 800px;
  margin: 0 auto;
  animation: sl-fade-in-up var(--sl-transition-normal) forwards;
}

.version-back-button {
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

.version-back-button:hover {
  background: var(--sl-surface-hover);
  border-color: var(--sl-primary);
  color: var(--sl-primary);
}

.version-container {
  text-align: center;
}

.version-title {
  font-size: 1.75rem;
  font-weight: 700;
  color: var(--sl-text-primary);
  margin: 0 0 var(--sl-space-sm) 0;
}

.version-subtitle {
  font-size: 1rem;
  color: var(--sl-text-secondary);
  margin: 0 0 var(--sl-space-2xl) 0;
}

.version-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sl-space-md);
  padding: var(--sl-space-2xl);
  color: var(--sl-text-secondary);
}

.version-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sl-space-md);
  padding: var(--sl-space-2xl);
  color: var(--sl-error);
}

.version-retry-button {
  padding: var(--sl-space-sm) var(--sl-space-lg);
  background: var(--sl-primary);
  color: var(--sl-text-inverse);
  border: none;
  border-radius: var(--sl-radius-md);
  font-size: 0.875rem;
  cursor: pointer;
  transition: background-color var(--sl-transition-fast) ease;
}

.version-retry-button:hover {
  background: var(--sl-primary-dark);
}

.version-list {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}

.version-item {
  padding: var(--sl-space-md) var(--sl-space-lg);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  cursor: pointer;
  text-align: left;
  transition:
    background-color var(--sl-transition-fast) ease,
    border-color var(--sl-transition-fast) ease,
    box-shadow var(--sl-transition-fast) ease,
    transform var(--sl-transition-fast) ease;
}

.version-item:hover {
  background: var(--sl-primary-bg);
  border-color: var(--sl-primary);
  box-shadow: 0 0 16px rgba(14, 165, 233, 0.2);
  transform: translateX(4px);
}

.version-text {
  font-size: 0.9375rem;
  font-weight: 500;
  color: var(--sl-text-primary);
}
</style>
