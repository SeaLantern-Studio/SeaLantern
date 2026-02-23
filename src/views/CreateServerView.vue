<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { useRouter } from "vue-router";
import { serverApi } from "@api/server";
import { javaApi, type JavaInfo } from "@api/java";
import { systemApi } from "@api/system";
import { settingsApi } from "@api/settings";
import { useServerStore } from "@stores/serverStore";
import { i18n } from "@language";
import { useMessage } from "@composables/useMessage";
import { useLoading } from "@composables/useAsync";
import JavaEnvironmentCard from "@components/views/create/JavaEnvironmentCard.vue";
import ServerConfigCard from "@components/views/create/ServerConfigCard.vue";
import CreateServerActions from "@components/views/create/CreateServerActions.vue";

type StartupMode = "jar" | "bat" | "sh";

const router = useRouter();
const store = useServerStore();
const { error: errorMsg, showError, clearError } = useMessage();
const { loading: javaLoading, start: startJavaLoading, stop: stopJavaLoading } = useLoading();
const { loading: creating, start: startCreating, stop: stopCreating } = useLoading();

const serverName = ref("My Server");
const maxMemory = ref("2048");
const minMemory = ref("512");
const port = ref("25565");
const jarPath = ref("");
const startupMode = ref<StartupMode>("jar");
const selectedJava = ref("");
const onlineMode = ref(true);

const javaList = ref<JavaInfo[]>([]);

watch(startupMode, () => {
  jarPath.value = "";
});

onMounted(async () => {
  await loadDefaultSettings();
});

async function loadDefaultSettings() {
  try {
    const settings = await settingsApi.get();

    maxMemory.value = String(settings.default_max_memory);
    minMemory.value = String(settings.default_min_memory);
    port.value = String(settings.default_port);

    if (settings.cached_java_list && settings.cached_java_list.length > 0) {
      javaList.value = settings.cached_java_list;

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
  startJavaLoading();
  try {
    javaList.value = await javaApi.detect();
    if (javaList.value.length > 0) {
      const preferred = javaList.value.find((j) => j.is_64bit && j.major_version >= 17);
      selectedJava.value = preferred ? preferred.path : javaList.value[0].path;
    }

    const settings = await settingsApi.get();
    settings.cached_java_list = javaList.value;
    await settingsApi.save(settings);
  } catch (e) {
    console.error("Java detection failed:", e);
    showError(String(e));
  } finally {
    stopJavaLoading();
  }
}

async function pickJarFile() {
  try {
    const mode = startupMode.value;
    const result = await systemApi.pickStartupFile(mode);
    if (result) {
      jarPath.value = result;
    } else {
      jarPath.value = "";
    }
  } catch (e) {
    console.error("Pick file error:", e);
    jarPath.value = "";
  }
}

async function handleCreate() {
  clearError();

  await pickJarFile();

  if (!jarPath.value) {
    return;
  }
  if (!selectedJava.value) {
    showError(i18n.t("common.select_java_path"));
    return;
  }
  if (!serverName.value.trim()) {
    showError(i18n.t("common.enter_server_name"));
    return;
  }

  startCreating();
  try {
    const mode = startupMode.value;
    await serverApi.importServer({
      name: serverName.value,
      jarPath: jarPath.value,
      startupMode: mode,
      javaPath: selectedJava.value,
      maxMemory: parseInt(maxMemory.value) || 2048,
      minMemory: parseInt(minMemory.value) || 512,
      port: parseInt(port.value) || 25565,
      onlineMode: onlineMode.value,
    });
    await store.refreshList();
    router.push("/");
  } catch (e) {
    showError(String(e));
  } finally {
    stopCreating();
  }
}

async function handleImport() {
  clearError();

  if (!selectedJava.value) {
    showError(i18n.t("common.select_java_path"));
    return;
  }
  if (!serverName.value.trim()) {
    showError(i18n.t("common.enter_server_name"));
    return;
  }

  const result = await systemApi.pickStartupFile(startupMode.value);
  if (!result) {
    return;
  }

  const serverPath = result.substring(0, result.lastIndexOf("\\") || result.lastIndexOf("/"));

  startCreating();
  try {
    const mode = startupMode.value;
    await serverApi.addExistingServer({
      name: serverName.value,
      serverPath: serverPath,
      javaPath: selectedJava.value,
      maxMemory: parseInt(maxMemory.value) || 2048,
      minMemory: parseInt(minMemory.value) || 512,
      port: parseInt(port.value) || 25565,
      startupMode: mode,
      executablePath: result,
    });
    await store.refreshList();
    router.push("/");
  } catch (e) {
    showError(String(e));
  } finally {
    stopCreating();
  }
}
</script>

<template>
  <div class="create-view animate-fade-in-up">
    <div v-if="errorMsg" class="error-banner">
      <span>{{ errorMsg }}</span>
      <button class="error-close" @click="clearError()">x</button>
    </div>

    <JavaEnvironmentCard
      :java-list="javaList"
      v-model:selected-java="selectedJava"
      :loading="javaLoading"
      @detect="detectJava"
    />

    <ServerConfigCard
      v-model:server-name="serverName"
      v-model:startup-mode="startupMode"
      v-model:max-memory="maxMemory"
      v-model:min-memory="minMemory"
      v-model:port="port"
      v-model:online-mode="onlineMode"
    />

    <CreateServerActions :creating="creating" @create="handleCreate" @import="handleImport" />
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
</style>
