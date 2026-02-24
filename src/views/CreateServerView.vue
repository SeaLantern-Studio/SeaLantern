<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import {
  StepperDescription,
  StepperIndicator,
  StepperItem,
  StepperRoot,
  StepperSeparator,
  StepperTitle,
  StepperTrigger,
} from "reka-ui";
import SLCard from "@components/common/SLCard.vue";
import CreateServerActions from "@components/views/create/CreateServerActions.vue";
import JavaEnvironmentStep from "@components/views/create/JavaEnvironmentStep.vue";
import SourceIntakeField, { type SourceType } from "@components/views/create/SourceIntakeField.vue";
import ServerStartupConfigStep from "@components/views/create/ServerStartupConfigStep.vue";
import { serverApi } from "@api/server";
import { javaApi, type JavaInfo } from "@api/java";
import { settingsApi } from "@api/settings";
import { useServerStore } from "@stores/serverStore";
import { i18n } from "@language";
import { useMessage } from "@composables/useMessage";
import { useLoading } from "@composables/useAsync";

const router = useRouter();
const store = useServerStore();
const { error: errorMsg, showError, clearError } = useMessage();
const { loading: javaLoading, start: startJavaLoading, stop: stopJavaLoading } = useLoading();
const { loading: creating, start: startCreating, stop: stopCreating } = useLoading();

const sourcePath = ref("");
const sourceType = ref<SourceType>("");
const coreDetecting = ref(false);
const detectedCoreType = ref("");
const detectedCoreMainClass = ref("");
let coreDetectRequestId = 0;

const serverName = ref("My Server");
const maxMemory = ref("2048");
const minMemory = ref("512");
const port = ref("25565");
const selectedJava = ref("");
const onlineMode = ref(true);

const javaList = ref<JavaInfo[]>([]);

const hasSource = computed(() => sourcePath.value.trim().length > 0 && sourceType.value !== "");
const hasJava = computed(() => selectedJava.value.trim().length > 0);
const hasServerConfig = computed(() => serverName.value.trim().length > 0);

const step1Completed = computed(() => hasSource.value);
const step2Completed = computed(() => step1Completed.value && hasJava.value);
const step3Completed = computed(() => step2Completed.value && hasServerConfig.value);

const activeStep = computed(() => {
  if (!hasSource.value) {
    return 1;
  }
  if (!hasJava.value) {
    return 2;
  }
  if (!hasServerConfig.value) {
    return 3;
  }
  return 4;
});

const stepItems = computed(() => {
  return [
    {
      step: 1,
      title: i18n.t("create.step_source_title"),
      description: i18n.t("create.step_source_desc"),
      completed: step1Completed.value,
    },
    {
      step: 2,
      title: i18n.t("create.step_java_title"),
      description: i18n.t("create.step_java_desc"),
      completed: step2Completed.value,
    },
    {
      step: 3,
      title: i18n.t("create.step_config_title"),
      description: i18n.t("create.step_config_desc"),
      completed: step3Completed.value,
    },
    {
      step: 4,
      title: i18n.t("create.step_action_title"),
      description: i18n.t("create.step_action_desc"),
      completed: false,
    },
  ];
});

const canCreate = computed(() => step3Completed.value);
const canImport = computed(
  () => step3Completed.value && sourceType.value === "folder",
);

const showSourceCoreInfo = computed(
  () => sourceType.value === "archive" && sourcePath.value.trim().length > 0,
);

const sourceCoreInfoText = computed(() => {
  const coreName = detectedCoreType.value || i18n.t("create.source_core_unknown");
  const mainClass = detectedCoreMainClass.value;
  const suffix = mainClass ? `（${mainClass}）` : "（）";
  return `${i18n.t("create.source_core_label")}${coreName}${suffix}`;
});

onMounted(async () => {
  await loadDefaultSettings();
});

watch(
  [sourcePath, sourceType],
  async ([path, type]) => {
    const currentRequest = ++coreDetectRequestId;

    if (type !== "archive" || path.trim().length === 0) {
      coreDetecting.value = false;
      detectedCoreType.value = "";
      detectedCoreMainClass.value = "";
      return;
    }

    coreDetecting.value = true;
    detectedCoreType.value = "";
    detectedCoreMainClass.value = "";

    try {
      const result = await serverApi.parseServerCoreType(path);
      if (currentRequest !== coreDetectRequestId) {
        return;
      }
      detectedCoreType.value = result.coreType;
      detectedCoreMainClass.value = result.mainClass ?? "";
    } catch (error) {
      if (currentRequest !== coreDetectRequestId) {
        return;
      }
      detectedCoreType.value = i18n.t("create.source_core_unknown");
      detectedCoreMainClass.value = "";
      showError(String(error));
    } finally {
      if (currentRequest === coreDetectRequestId) {
        coreDetecting.value = false;
      }
    }
  },
  { immediate: true },
);

function parseNumber(value: string, fallbackValue: number): number {
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallbackValue;
}

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
      } else {
        const preferredJava = javaList.value.find((java) => java.is_64bit && java.major_version >= 17);
        selectedJava.value = preferredJava ? preferredJava.path : javaList.value[0].path;
      }
    }
  } catch (error) {
    console.error("Failed to load default settings:", error);
  }
}

async function detectJava() {
  startJavaLoading();
  try {
    javaList.value = await javaApi.detect();
    if (javaList.value.length > 0) {
      const preferredJava = javaList.value.find((java) => java.is_64bit && java.major_version >= 17);
      selectedJava.value = preferredJava ? preferredJava.path : javaList.value[0].path;
    }

    const settings = await settingsApi.get();
    settings.cached_java_list = javaList.value;
    await settingsApi.save(settings);
  } catch (error) {
    showError(String(error));
  } finally {
    stopJavaLoading();
  }
}

function validateBeforeSubmit(): boolean {
  clearError();

  if (!hasSource.value) {
    showError(i18n.t("create.source_required"));
    return false;
  }

  if (!selectedJava.value) {
    showError(i18n.t("common.select_java_path"));
    return false;
  }

  if (!serverName.value.trim()) {
    showError(i18n.t("common.enter_server_name"));
    return false;
  }

  return true;
}

async function handleCreate() {
  if (!validateBeforeSubmit()) {
    return;
  }

  startCreating();
  try {
    await serverApi.importModpack({
      name: serverName.value.trim(),
      modpackPath: sourcePath.value,
      javaPath: selectedJava.value,
      maxMemory: parseNumber(maxMemory.value, 2048),
      minMemory: parseNumber(minMemory.value, 512),
      port: parseNumber(port.value, 25565),
    });
    await store.refreshList();
    router.push("/");
  } catch (error) {
    showError(String(error));
  } finally {
    stopCreating();
  }
}

async function handleImport() {
  if (!validateBeforeSubmit()) {
    return;
  }

  if (sourceType.value !== "folder") {
    showError(i18n.t("create.import_requires_folder"));
    return;
  }

  startCreating();
  try {
    await serverApi.addExistingServer({
      name: serverName.value.trim(),
      serverPath: sourcePath.value,
      javaPath: selectedJava.value,
      maxMemory: parseNumber(maxMemory.value, 2048),
      minMemory: parseNumber(minMemory.value, 512),
      port: parseNumber(port.value, 25565),
      startupMode: "jar",
    });
    await store.refreshList();
    router.push("/");
  } catch (error) {
    showError(String(error));
  } finally {
    stopCreating();
  }
}
</script>

<template>
  <div class="create-view animate-fade-in-up">
    <div v-if="errorMsg" class="create-error-banner">
      <span>{{ errorMsg }}</span>
      <button class="create-error-close" @click="clearError">x</button>
    </div>

    <SLCard class="create-stepper-card" :title="i18n.t('create.title')">
      <StepperRoot
        orientation="vertical"
        :model-value="activeStep"
        :linear="false"
        class="create-stepper"
      >
        <StepperItem
          v-for="item in stepItems"
          :key="item.step"
          :step="item.step"
          :completed="item.completed"
          class="create-stepper-item"
        >
          <StepperTrigger class="create-stepper-trigger">
            <StepperIndicator class="create-stepper-indicator">
              {{ item.step }}
            </StepperIndicator>
            <div class="create-stepper-copy">
              <StepperTitle class="create-stepper-title">{{ item.title }}</StepperTitle>
              <StepperDescription class="create-stepper-description">{{ item.description }}</StepperDescription>
            </div>
          </StepperTrigger>

          <div class="create-step-panel">
            <template v-if="item.step === 1">
              <SourceIntakeField
                v-model:source-path="sourcePath"
                v-model:source-type="sourceType"
                @error="showError"
              />
              <p v-if="showSourceCoreInfo" class="create-source-core-info">
                {{
                  coreDetecting
                    ? i18n.t("create.source_detecting_core")
                    : sourceCoreInfoText
                }}
              </p>
            </template>

            <JavaEnvironmentStep
              v-else-if="item.step === 2"
              :java-list="javaList"
              :loading="javaLoading"
              :selected-java="selectedJava"
              @detect="detectJava"
              @update:selected-java="selectedJava = $event"
            />

            <ServerStartupConfigStep
              v-else-if="item.step === 3"
              :server-name="serverName"
              :max-memory="maxMemory"
              :min-memory="minMemory"
              :port="port"
              :online-mode="onlineMode"
              @update:server-name="serverName = $event"
              @update:max-memory="maxMemory = $event"
              @update:min-memory="minMemory = $event"
              @update:port="port = $event"
              @update:online-mode="onlineMode = $event"
            />

            <CreateServerActions
              v-else
              :creating="creating"
              :create-disabled="!canCreate || creating"
              :import-disabled="!canImport || creating"
              @create="handleCreate"
              @import="handleImport"
            />
          </div>

          <StepperSeparator v-if="item.step < stepItems.length" class="create-stepper-separator" />
        </StepperItem>
      </StepperRoot>
    </SLCard>
  </div>
</template>

<style src="@styles/views/CreateServerView.css" scoped></style>
