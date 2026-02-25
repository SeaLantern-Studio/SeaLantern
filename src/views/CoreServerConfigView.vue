<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from "vue";
import { useRouter, useRoute } from "vue-router";
import { ArrowLeft } from "lucide-vue-next";
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import JavaEnvironmentStep from "@components/views/create/JavaEnvironmentStep.vue";
import { systemApi } from "@api/system";
import { serverApi } from "@api/server";
import { javaApi, type JavaInfo } from "@api/java";
import { fetchSystemInfo } from "@utils/statsUtils";
import { systemInfo } from "@utils/statsUtils";

const router = useRouter();
const route = useRoute();

const version = ref<string>("");
const build = ref<string>("");
const serverName = ref<string>("");
const source = ref<string>("official");
const sha256 = ref<string>("");
const serverNameInput = ref<string>("");
const serverPath = ref<string>("");
const pathError = ref<string>("");
const maxMemory = ref<number>(2048);
const minMemory = ref<number>(1024);
const maxMemoryInput = ref<string>("2048");
const minMemoryInput = ref<string>("1024");
const javaList = ref<JavaInfo[]>([]);
const selectedJava = ref<string>("");
const javaLoading = ref<boolean>(false);
const systemMemory = ref<number>(8192);

const goBack = () => {
  router.push({
    path: `/core-download/builds/${serverName.value}/${version.value}`,
    query: { source: source.value },
  });
};

const pickPath = async () => {
  const selected = await systemApi.pickFolder();
  if (selected) {
    try {
      const isEmpty = await serverApi.isDirectoryEmpty(selected);
      if (!isEmpty) {
        pathError.value = "所选目录不为空，请选择一个空目录";
        serverPath.value = "";
        return;
      }
      serverPath.value = selected;
      pathError.value = "";
    } catch (err) {
      console.error("Failed to check directory:", err);
      pathError.value = "检查目录失败";
      serverPath.value = "";
    }
  }
};

const handleMaxMemoryInput = (value: string) => {
  maxMemoryInput.value = value;
  const num = parseInt(value);
  if (!isNaN(num) && num >= 512 && num <= systemMemory.value) {
    maxMemory.value = num;
    if (minMemory.value > num) {
      minMemory.value = Math.max(512, Math.floor(num * 0.5));
      minMemoryInput.value = minMemory.value.toString();
    }
    updateSliders();
  }
};

const handleMinMemoryInput = (value: string) => {
  minMemoryInput.value = value;
  const num = parseInt(value);
  if (!isNaN(num) && num >= 512) {
    if (num > maxMemory.value) {
      minMemory.value = maxMemory.value;
      minMemoryInput.value = maxMemory.value.toString();
    } else {
      minMemory.value = num;
    }
    updateSliders();
  }
};

const handleMaxMemorySlider = (e: Event) => {
  const target = e.target as HTMLInputElement;
  maxMemory.value = parseInt(target.value);
  maxMemoryInput.value = target.value;
  if (minMemory.value > maxMemory.value) {
    minMemory.value = Math.max(512, Math.floor(maxMemory.value * 0.5));
    minMemoryInput.value = minMemory.value.toString();
  }
  updateSliders();
};

const handleMinMemorySlider = (e: Event) => {
  const target = e.target as HTMLInputElement;
  minMemory.value = parseInt(target.value);
  minMemoryInput.value = target.value;
  updateSliders();
};

const updateSliders = () => {
  nextTick(() => {
    const maxSlider = document.querySelector(
      ".config-slider:not(.config-slider-secondary)",
    ) as HTMLInputElement;
    const minSlider = document.querySelector(".config-slider-secondary") as HTMLInputElement;
    if (maxSlider) {
      const percent = ((maxMemory.value - 512) / (systemMemory.value - 512)) * 100;
      maxSlider.style.setProperty("--slider-percent", `${percent}%`);
    }
    if (minSlider) {
      const percent = ((minMemory.value - 512) / (maxMemory.value - 512)) * 100;
      minSlider.style.setProperty("--slider-percent", `${percent}%`);
    }
  });
};

const detectJava = async () => {
  javaLoading.value = true;
  try {
    const list = await javaApi.detect();
    javaList.value = list;
    if (list.length > 0 && !selectedJava.value) {
      selectedJava.value = list[0].path;
    }
  } catch (err) {
    console.error("Failed to detect Java:", err);
  } finally {
    javaLoading.value = false;
  }
};

const canSubmit = computed(() => {
  return (
    serverNameInput.value &&
    serverPath.value &&
    selectedJava.value &&
    maxMemory.value >= minMemory.value
  );
});

const handleSubmit = () => {
  router.push({
    path: `/core-download/download/${serverName.value}/${version.value}/${build.value}`,
    query: {
      sha256: sha256.value,
      name: serverNameInput.value,
      path: serverPath.value,
      maxMemory: maxMemory.value.toString(),
      minMemory: minMemory.value.toString(),
      java: selectedJava.value,
      source: source.value,
    },
  });
};

onMounted(async () => {
  const serverParam = route.params.coreName as string;
  const versionParam = route.params.version as string;
  const buildParam = route.params.build as string;
  const sha256Param = route.query.sha256 as string;
  const sourceParam = route.query.source as string;

  if (serverParam && versionParam && buildParam) {
    serverName.value = serverParam;
    version.value = versionParam;
    build.value = buildParam;
    sha256.value = sha256Param || "";
    source.value = sourceParam || "official";
  }

  await fetchSystemInfo();
  if (systemInfo.value) {
    systemMemory.value = Math.floor(systemInfo.value.memory.total / (1024 * 1024));
    maxMemory.value = Math.min(2048, Math.floor(systemMemory.value * 0.5));
    minMemory.value = Math.min(1024, Math.floor(maxMemory.value * 0.5));
    maxMemoryInput.value = maxMemory.value.toString();
    minMemoryInput.value = minMemory.value.toString();

    updateSliders();
  }

  detectJava();
});

// 监听路由参数变化
watch(
  () => [route.params.coreName, route.params.version, route.params.build, route.query.source],
  ([newServer, newVersion, newBuild, newSource]) => {
    if (newServer && typeof newServer === "string") {
      serverName.value = newServer;
    }
    if (newVersion && typeof newVersion === "string") {
      version.value = newVersion;
    }
    if (newBuild && typeof newBuild === "string") {
      build.value = newBuild;
    }
    if (newSource && typeof newSource === "string") {
      source.value = newSource;
    }
  },
);
</script>

<template>
  <div :key="`${serverName}-${version}-${build}-${source}`" class="config-view">
    <button class="config-back-button" @click="goBack">
      <ArrowLeft :size="20" />
      <span>返回上一页</span>
    </button>

    <div class="config-container">
      <h1 class="config-title">
        配置 {{ serverName.charAt(0).toUpperCase() + serverName.slice(1) }} {{ version }} #{{
          build
        }}
      </h1>
      <p class="config-subtitle">配置服务器参数</p>

      <div class="config-section">
        <h3 class="config-section-title">基本信息</h3>
        <div class="config-field">
          <label class="config-label">服务器名称</label>
          <SLInput
            :model-value="serverNameInput"
            placeholder="输入服务器名称"
            @update:model-value="serverNameInput = $event"
          />
        </div>
        <div class="config-field">
          <label class="config-label">服务器目录</label>
          <SLInput :model-value="serverPath" placeholder="选择服务器目录（必须为空）" readonly>
            <template #suffix>
              <button class="config-browse-button" @click="pickPath">浏览</button>
            </template>
          </SLInput>
          <p v-if="pathError" class="config-error">{{ pathError }}</p>
        </div>
      </div>

      <div class="config-section">
        <h3 class="config-section-title">内存配置</h3>
        <p class="config-memory-info">系统总内存: {{ systemMemory }} MB</p>

        <div class="config-memory-group">
          <div class="config-memory-item">
            <label class="config-label">最大内存 (MB)</label>
            <div class="config-memory-control">
              <div class="config-slider-wrapper">
                <input
                  type="range"
                  :min="512"
                  :max="systemMemory"
                  :value="maxMemory"
                  class="config-slider"
                  @input="handleMaxMemorySlider"
                />
              </div>
              <SLInput
                :model-value="maxMemoryInput"
                type="text"
                class="config-memory-input"
                @update:model-value="handleMaxMemoryInput"
              />
            </div>
          </div>

          <div class="config-memory-item">
            <label class="config-label">最小内存 (MB)</label>
            <div class="config-memory-control">
              <div class="config-slider-wrapper">
                <input
                  type="range"
                  :min="512"
                  :max="maxMemory"
                  :value="minMemory"
                  class="config-slider config-slider-secondary"
                  @input="handleMinMemorySlider"
                />
              </div>
              <SLInput
                :model-value="minMemoryInput"
                type="text"
                class="config-memory-input"
                @update:model-value="handleMinMemoryInput"
              />
            </div>
          </div>
        </div>
      </div>

      <div class="config-section">
        <h3 class="config-section-title">Java 环境</h3>
        <JavaEnvironmentStep
          :java-list="javaList"
          :selected-java="selectedJava"
          :loading="javaLoading"
          @detect="detectJava"
          @update:selected-java="selectedJava = $event"
        />
      </div>

      <div class="config-actions">
        <SLButton variant="secondary" size="lg" @click="goBack">取消</SLButton>
        <SLButton variant="primary" size="lg" :disabled="!canSubmit" @click="handleSubmit">
          开始下载
        </SLButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.config-view {
  padding: var(--sl-space-xl);
  max-width: 800px;
  margin: 0 auto;
  animation: sl-fade-in-up var(--sl-transition-normal) forwards;
}

.config-back-button {
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

.config-back-button:hover {
  background: var(--sl-surface-hover);
  border-color: var(--sl-primary);
  color: var(--sl-primary);
}

.config-container {
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-lg);
  padding: var(--sl-space-xl);
}

.config-title {
  font-size: 1.75rem;
  font-weight: 700;
  color: var(--sl-text-primary);
  margin: 0 0 var(--sl-space-sm) 0;
}

.config-subtitle {
  font-size: 1rem;
  color: var(--sl-text-secondary);
  margin: 0 0 var(--sl-space-2xl) 0;
}

.config-section {
  margin-bottom: var(--sl-space-2xl);
}

.config-section-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 var(--sl-space-md) 0;
}

.config-field {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
  margin-bottom: var(--sl-space-md);
}

.config-field:last-child {
  margin-bottom: 0;
}

.config-browse-button {
  padding: var(--sl-space-xs) var(--sl-space-md);
  background: var(--sl-primary);
  color: var(--sl-text-inverse);
  border: none;
  border-radius: var(--sl-radius-sm);
  font-size: 0.875rem;
  cursor: pointer;
  transition: background-color var(--sl-transition-fast) ease;
}

.config-browse-button:hover {
  background: var(--sl-primary-dark);
}

.config-path-row {
  display: flex;
  gap: var(--sl-space-sm);
  align-items: center;
}

.config-error {
  color: var(--sl-error);
  font-size: 0.875rem;
  margin-top: var(--sl-space-xs);
}

.config-memory-info {
  font-size: 0.875rem;
  color: var(--sl-text-tertiary);
  margin-bottom: var(--sl-space-md);
}

.config-memory-group {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-lg);
}

.config-memory-item {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}

.config-label {
  font-size: 0.9375rem;
  font-weight: 500;
  color: var(--sl-text-primary);
}

.config-memory-control {
  display: flex;
  gap: var(--sl-space-md);
  align-items: center;
}

.config-slider-wrapper {
  position: relative;
  flex: 1;
  height: 8px;
  background: var(--sl-bg-secondary);
  border-radius: var(--sl-radius-full);
}

.config-slider {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 100%;
  height: 8px;
  -webkit-appearance: none;
  appearance: none;
  --slider-percent: 0%;
  background: linear-gradient(
    to right,
    var(--sl-primary) 0%,
    var(--sl-primary) var(--slider-percent),
    transparent var(--slider-percent)
  );
  border-radius: var(--sl-radius-full);
  outline: none;
  cursor: pointer;
}

.config-slider-secondary {
  --slider-percent: 0%;
  background: linear-gradient(
    to right,
    var(--sl-success) 0%,
    var(--sl-success) var(--slider-percent),
    transparent var(--slider-percent)
  );
}

.config-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 20px;
  height: 20px;
  background: var(--sl-primary);
  border: 3px solid var(--sl-surface);
  border-radius: 50%;
  cursor: pointer;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  transition: transform var(--sl-transition-fast) ease;
}

.config-slider::-webkit-slider-thumb:hover {
  transform: scale(1.1);
}

.config-slider-secondary::-webkit-slider-thumb {
  background: var(--sl-success);
}

.config-slider::-moz-range-thumb {
  width: 20px;
  height: 20px;
  background: var(--sl-primary);
  border: 3px solid var(--sl-surface);
  border-radius: 50%;
  cursor: pointer;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  transition: transform var(--sl-transition-fast) ease;
}

.config-slider::-moz-range-thumb:hover {
  transform: scale(1.1);
}

.config-slider-secondary::-moz-range-thumb {
  background: var(--sl-success);
}

.config-slider::-moz-range-track {
  background: transparent;
  border: none;
}

.config-memory-input {
  width: 120px;
}

.config-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--sl-space-md);
  margin-top: var(--sl-space-2xl);
  padding-top: var(--sl-space-xl);
  border-top: 1px solid var(--sl-border);
}
</style>
