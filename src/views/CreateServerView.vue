<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import SLCard from "../components/common/SLCard.vue";
import SLButton from "../components/common/SLButton.vue";
import SLInput from "../components/common/SLInput.vue";
import SLSelect from "../components/common/SLSelect.vue";
import SLSwitch from "../components/common/SLSwitch.vue";
import SLSpinner from "../components/common/SLSpinner.vue";
import { serverApi } from "../api/server";
import { javaApi, type JavaInfo } from "../api/java";
import { systemApi } from "../api/system";
import { settingsApi } from "../api/settings";
import { useServerStore } from "../stores/serverStore";
import { i18n } from "../locales";

const router = useRouter();
const store = useServerStore();

const serverName = ref("My Server");
const maxMemory = ref("2048");
const minMemory = ref("512");
const port = ref("25565");
const jarPath = ref("");
const selectedJava = ref("");
const onlineMode = ref(true);

const javaList = ref<JavaInfo[]>([]);
const javaLoading = ref(false);
const creating = ref(false);
const errorMsg = ref<string | null>(null);

onMounted(async () => {
  await loadDefaultSettings();
});

async function loadDefaultSettings() {
  try {
    const settings = await settingsApi.get();

    // Load default values from settings
    maxMemory.value = String(settings.default_max_memory);
    minMemory.value = String(settings.default_min_memory);
    port.value = String(settings.default_port);

    // Load cached Java list
    if (settings.cached_java_list && settings.cached_java_list.length > 0) {
      javaList.value = settings.cached_java_list;

      // Auto-select Java: prefer default_java_path, then recommended version
      if (settings.default_java_path) {
        selectedJava.value = settings.default_java_path;
      } else if (javaList.value.length > 0) {
        const preferred = javaList.value.find((j) => j.is_64bit && j.major_version >= 17);
        selectedJava.value = preferred ? preferred.path : javaList.value[0].path;
      }
    }
  } catch (e) {
    console.error("Failed to load default settings:", e);
  }
}

async function detectJava() {
  javaLoading.value = true;
  try {
    javaList.value = await javaApi.detect();
    if (javaList.value.length > 0) {
      const preferred = javaList.value.find((j) => j.is_64bit && j.major_version >= 17);
      selectedJava.value = preferred ? preferred.path : javaList.value[0].path;
    }

    // 保存到设置中
    const settings = await settingsApi.get();
    settings.cached_java_list = javaList.value;
    await settingsApi.save(settings);
  } catch (e) {
    console.error("Java detection failed:", e);
  } finally {
    javaLoading.value = false;
  }
}

async function pickJarFile() {
  try {
    const result = await systemApi.pickJarFile();
    if (result) {
      jarPath.value = result;
    } else {
      // 重置 jarPath，防炸...
      jarPath.value = "";
    }
  } catch (e) {
    console.error("Pick file error:", e);
    jarPath.value = "";
  }
}

async function pickJavaFile() {
  try {
    const result = await systemApi.pickJavaFile();
    if (result) {
      selectedJava.value = result;
    }
  } catch (e) {
    console.error("Pick file error:", e);
  }
}

async function handleCreate() {
  errorMsg.value = null;

  // 直接弹出文件选择窗口
  await pickJarFile();

  // 选择文件后检查其他条件
  if (!jarPath.value) {
    // 用户取消了文件选择，不显示错误信息
    return;
  }
  if (!selectedJava.value) {
    errorMsg.value = "请选择 Java 路径";
    return;
  }
  if (!serverName.value.trim()) {
    errorMsg.value = "请输入服务器名称";
    return;
  }

  creating.value = true;
  try {
    await serverApi.importServer({
      name: serverName.value,
      jarPath: jarPath.value,
      javaPath: selectedJava.value,
      maxMemory: parseInt(maxMemory.value) || 2048,
      minMemory: parseInt(minMemory.value) || 512,
      port: parseInt(port.value) || 25565,
      onlineMode: onlineMode.value,
    });
    await store.refreshList();
    router.push("/");
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    creating.value = false;
  }
}

function getJavaLabel(java: JavaInfo): string {
  const type = java.major_version <= 8 ? "JRE" : "JDK";
  const arch = java.is_64bit ? "64-bit" : "32-bit";
  return `${type} ${java.major_version} (${java.version}) - ${java.vendor} [${arch}]`;
}

const javaOptions = computed(() => {
  return javaList.value.map((java) => ({
    label: getJavaLabel(java),
    value: java.path,
  }));
});
</script>

<template>
  <div class="create-view animate-fade-in-up">
    <div v-if="errorMsg" class="error-banner">
      <span>{{ errorMsg }}</span>
      <button class="error-close" @click="errorMsg = null">x</button>
    </div>

    <SLCard :title="i18n.t('create.java_env')" :subtitle="i18n.t('create.java_scan')">
      <div v-if="javaLoading" class="java-loading">
        <div class="spinner"></div>
        <span>{{ i18n.t("create.scanning") }}</span>
      </div>
      <div v-else-if="javaList.length === 0" class="java-empty">
        <p class="text-body">{{ i18n.t("create.no_java") }}</p>
        <SLButton variant="primary" @click="detectJava" style="margin-top: 12px">
          {{ i18n.t("create.browse") }}
        </SLButton>
      </div>
      <div v-else class="java-select-container">
        <div class="java-header">
          <div class="java-found text-caption">
            {{ i18n.t("create.java_found", { count: javaList.length }) }}
          </div>
          <button class="rescan-btn" @click="detectJava" :disabled="javaLoading">
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path
                d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"
              />
            </svg>
            {{ i18n.t("create.rescan") }}
          </button>
        </div>
        <SLSelect
          v-model="selectedJava"
          :options="javaOptions"
          :placeholder="i18n.t('create.select_java')"
          searchable
          maxHeight="240px"
        />
        <div v-if="selectedJava" class="selected-java-path">
          <span class="text-caption">{{ i18n.t("create.server_path") }}：</span>
          <span class="text-mono text-caption">{{ selectedJava }}</span>
        </div>
      </div>
      <div class="java-manual">
        <SLInput
          :label="i18n.t('create.java_version')"
          v-model="selectedJava"
          :placeholder="i18n.t('create.java_manual')"
        >
          <template #suffix>
            <button class="pick-btn" @click="pickJavaFile">{{ i18n.t("create.browse") }}</button>
          </template>
        </SLInput>
      </div>
    </SLCard>

    <SLCard :title="i18n.t('create.title')">
      <div class="form-grid">
        <div class="server-name-row">
          <SLInput
            :label="i18n.t('create.server_name')"
            :placeholder="i18n.t('create.server_name')"
            v-model="serverName"
          />
        </div>

        <SLInput :label="i18n.t('create.max_memory')" type="number" v-model="maxMemory" />
        <SLInput :label="i18n.t('create.min_memory')" type="number" v-model="minMemory" />
        <SLInput
          :label="i18n.t('settings.default_port')"
          type="number"
          v-model="port"
          :placeholder="i18n.t('create.default_port_placeholder')"
        />
        <div class="online-mode-cell">
          <span class="online-mode-label">{{ i18n.t("create.online_mode") }}</span>
          <div class="online-mode-box">
            <span class="online-mode-text">{{
              onlineMode ? i18n.t("create.online_mode_on") : i18n.t("create.online_mode_off")
            }}</span>
            <SLSwitch v-model="onlineMode" />
          </div>
        </div>
      </div>
    </SLCard>

    <div class="create-actions">
      <SLButton variant="secondary" size="lg" @click="router.push('/')">{{
        i18n.t("create.cancel")
      }}</SLButton>
      <SLButton variant="primary" size="lg" :loading="creating" @click="handleCreate">
        {{ i18n.t("create.create") }}
      </SLButton>
    </div>
  </div>
</template>

<style scoped>
.create-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-lg);
  max-width: 760px;
  margin: 0 auto;
}
.error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: var(--sl-radius-md);
  color: var(--sl-error);
  font-size: 0.875rem;
}
.error-close {
  color: var(--sl-error);
  font-weight: 600;
}
.java-loading {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-lg);
  color: var(--sl-text-tertiary);
}
.java-empty {
  padding: var(--sl-space-lg);
  text-align: center;
}
.java-select-container {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}
.java-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--sl-space-xs);
}
.java-found {
  margin: 0;
}
.rescan-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
  border-radius: var(--sl-radius-sm);
  cursor: pointer;
  transition: all var(--sl-transition-fast);
}
.rescan-btn:hover:not(:disabled) {
  background: var(--sl-primary);
  color: white;
}
.rescan-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.selected-java-path {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: 8px 12px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-sm);
  overflow: hidden;
}
.selected-java-path .text-mono {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.java-manual {
  padding-top: var(--sl-space-sm);
  border-top: 1px solid var(--sl-border-light);
}
.form-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--sl-space-md);
}
.server-name-row {
  grid-column: 1 / -1;
}

.pick-btn {
  padding: 4px 12px;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
  border-radius: var(--sl-radius-sm);
  cursor: pointer;
  white-space: nowrap;
}
.pick-btn:hover {
  background: var(--sl-primary);
  color: white;
}
.online-mode-cell {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
}
.online-mode-label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-text-secondary);
}
.online-mode-box {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-md);
  padding: 6px 12px;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  height: 36px;
  box-sizing: border-box;
}
.online-mode-text {
  font-size: 0.875rem;
  color: var(--sl-text-tertiary);
}
.create-actions {
  display: flex;
  justify-content: center;
  gap: var(--sl-space-md);
}
.create-actions :deep(.sl-button) {
  min-width: 120px;
}
</style>
