<script setup lang="ts">
import { ref, watch, onBeforeUnmount } from "vue";
import { useRouter, useRoute } from "vue-router";
import { ArrowLeft } from "lucide-vue-next";
import SLSpinner from "@components/common/SLSpinner.vue";
import { getPaperBuilds, getPaperBuildInfo, type PaperBuildInfo } from "@utils/networkapi/paperapi";
import { getServerBuilds } from "@utils/networkapi/mslapi.ts";

const router = useRouter();
const route = useRoute();
const loading = ref<boolean>(false);
const error = ref<string | null>(null);
const builds = ref<string[]>([]);
const buildInfoMap = ref<Map<string, PaperBuildInfo>>(new Map());
const version = ref<string>("");
const serverName = ref<string>("");
const source = ref<string>("official");
const isNavigating = ref<boolean>(false);

const goBack = () => {
  isNavigating.value = true;
  router.push({
    path: `/core-download/versions/${serverName.value}`,
    query: { source: source.value },
  });
};

const formatTime = (timeStr: string): string => {
  const date = new Date(timeStr);
  return date.toLocaleString("zh-CN");
};

const selectBuild = (build: string) => {
  if (source.value === "official") {
    const buildInfo = buildInfoMap.value.get(build);
    if (buildInfo) {
      router.push({
        path: `/core-download/config/${serverName.value}/${version.value}/${build}`,
        query: {
          sha256: buildInfo.downloads.application.sha256,
          source: source.value,
        },
      });
    }
  } else {
    // MSL 镜像站直接使用 build 字符串（通常是 "latest"）
    router.push({
      path: `/core-download/config/${serverName.value}/${version.value}/${build}`,
      query: {
        source: source.value,
      },
    });
  }
};

const loadBuilds = async (targetVersion: string, targetServer: string, targetSource: string) => {
  if (isNavigating.value || !targetVersion || !targetServer) {
    return;
  }

  try {
    loading.value = true;
    error.value = null;
    version.value = targetVersion;
    serverName.value = targetServer;
    source.value = targetSource;
    builds.value = [];
    buildInfoMap.value.clear();

    if (targetSource === "official") {
      const buildList = await getPaperBuilds(targetVersion);

      if (isNavigating.value) {
        return;
      }

      builds.value = buildList.map(String);

      const buildInfoPromises = buildList.map((build) =>
        getPaperBuildInfo(targetVersion, build)
          .then((buildInfo) => {
            if (!isNavigating.value && version.value === targetVersion) {
              buildInfoMap.value.set(String(build), buildInfo);
            }
            return buildInfo;
          })
          .catch((err) => {
            console.error(`Failed to load build ${build} info:`, err);
            return null;
          }),
      );

      await Promise.all(buildInfoPromises);
    } else {
      // MSL 镜像站
      const buildList = await getServerBuilds(targetServer, targetVersion);

      if (isNavigating.value) {
        return;
      }

      builds.value = buildList;
    }
  } catch (err) {
    if (!isNavigating.value) {
      error.value = err instanceof Error ? err.message : "加载失败";
    }
  } finally {
    if (!isNavigating.value) {
      loading.value = false;
    }
  }
};

watch(
  () => [route.params.version, route.params.coreName, route.query.source],
  ([newVersion, newServerName, newSource]) => {
    if (
      newVersion &&
      typeof newVersion === "string" &&
      newServerName &&
      typeof newServerName === "string"
    ) {
      isNavigating.value = false;
      loadBuilds(newVersion, newServerName, (newSource as string) || "official");
    }
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  isNavigating.value = true;
});
</script>

<template>
  <div v-if="version" :key="`${serverName}-${version}-${source}`" class="build-view">
    <button class="build-back-button" @click="goBack">
      <ArrowLeft :size="20" />
      <span>返回上一页</span>
    </button>

    <div class="build-container">
      <h1 class="build-title">
        选择 {{ serverName.charAt(0).toUpperCase() + serverName.slice(1) }} {{ version }} 构建版本
      </h1>
      <p class="build-subtitle">选择你需要的构建版本</p>

      <div v-if="loading" class="build-loading">
        <SLSpinner />
        <p>正在加载构建版本...</p>
      </div>

      <div v-else-if="error" class="build-error">
        <p>{{ error }}</p>
        <button class="build-retry-button" @click="loadBuilds(version, serverName, source)">
          重试
        </button>
      </div>

      <div v-else class="build-list">
        <button v-for="build in builds" :key="build" class="build-item" @click="selectBuild(build)">
          <div class="build-header">
            <span class="build-text">构建 #{{ build }}</span>
          </div>
          <div v-if="source === 'official' && buildInfoMap.has(build)" class="build-meta">
            <span class="build-time">{{ formatTime(buildInfoMap.get(build)!.time) }}</span>
          </div>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.build-view {
  padding: var(--sl-space-xl);
  max-width: 800px;
  margin: 0 auto;
  animation: sl-fade-in-up var(--sl-transition-normal) forwards;
}

.build-back-button {
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

.build-back-button:hover {
  background: var(--sl-surface-hover);
  border-color: var(--sl-primary);
  color: var(--sl-primary);
}

.build-container {
  text-align: center;
}

.build-title {
  font-size: 1.75rem;
  font-weight: 700;
  color: var(--sl-text-primary);
  margin: 0 0 var(--sl-space-sm) 0;
}

.build-subtitle {
  font-size: 1rem;
  color: var(--sl-text-secondary);
  margin: 0 0 var(--sl-space-2xl) 0;
}

.build-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sl-space-md);
  padding: var(--sl-space-2xl);
  color: var(--sl-text-secondary);
}

.build-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sl-space-md);
  padding: var(--sl-space-2xl);
  color: var(--sl-error);
}

.build-retry-button {
  padding: var(--sl-space-sm) var(--sl-space-lg);
  background: var(--sl-primary);
  color: var(--sl-text-inverse);
  border: none;
  border-radius: var(--sl-radius-md);
  font-size: 0.875rem;
  cursor: pointer;
  transition: background-color var(--sl-transition-fast) ease;
}

.build-retry-button:hover {
  background: var(--sl-primary-dark);
}

.build-list {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
  max-height: 600px;
  overflow-y: auto;
  padding-right: var(--sl-space-xs);
}

.build-list::-webkit-scrollbar {
  width: 8px;
}

.build-list::-webkit-scrollbar-track {
  background: var(--sl-bg-secondary);
  border-radius: var(--sl-radius-sm);
}

.build-list::-webkit-scrollbar-thumb {
  background: var(--sl-border);
  border-radius: var(--sl-radius-sm);
}

.build-list::-webkit-scrollbar-thumb:hover {
  background: var(--sl-primary);
}

.build-item {
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

.build-item:hover {
  background: var(--sl-primary-bg);
  border-color: var(--sl-primary);
  box-shadow: 0 0 16px rgba(14, 165, 233, 0.2);
  transform: translateX(4px);
}

.build-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.build-text {
  font-size: 0.9375rem;
  font-weight: 500;
  color: var(--sl-text-primary);
}

.build-meta {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
  margin-top: var(--sl-space-xs);
}

.build-time {
  font-size: 0.8125rem;
  color: var(--sl-text-tertiary);
}
</style>
