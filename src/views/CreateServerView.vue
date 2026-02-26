<script setup lang="ts">
import {
  StepperDescription,
  StepperIndicator,
  StepperItem,
  StepperRoot,
  StepperSeparator,
  StepperTitle,
  StepperTrigger,
} from "reka-ui";
import { computed, onMounted, ref, watch } from "vue";
import { Download } from "lucide-vue-next";
import { useRoute, useRouter } from "vue-router";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import SLInput from "@components/common/SLInput.vue";
import SLProgress from "@components/common/SLProgress.vue";
import SLSelect from "@components/common/SLSelect.vue";
import JavaEnvironmentStep from "@components/views/create/JavaEnvironmentStep.vue";
import RunPathStep from "@components/views/create/RunPathStep.vue";
import ServerStartupConfigStep from "@components/views/create/ServerStartupConfigStep.vue";
import SourceIntakeField from "@components/views/create/SourceIntakeField.vue";
import StartupSelectionStep from "@components/views/create/StartupSelectionStep.vue";
import { downloadApi } from "@api/downloader";
import { serverApi } from "@api/server";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import { STARTER_SERVER_JAR_NAME } from "@components/views/create/constants";
import { joinPath, normalizePathForMatch } from "@components/views/create/startupUtils";
import { useCreateServerPage } from "@components/views/create/useCreateServerPage";

const {
  errorMsg,
  clearError,
  showError,
  javaLoading,
  creating,
  sourcePath,
  sourceType,
  runPath,
  runPathOverwriteRisk,
  useSoftwareDataDir,
  coreDetecting,
  startupDetecting,
  startupCandidates,
  selectedStartupId,
  customStartupCommand,
  starterSelected,
  detectedCoreTypeKey,
  coreTypeOptions,
  selectedCoreType,
  detectedMcVersion,
  mcVersionOptions,
  selectedMcVersion,
  mcVersionDetectionFailed,
  customCommandHasRedirect,
  serverName,
  maxMemory,
  minMemory,
  port,
  selectedJava,
  onlineMode,
  javaList,
  stepItems,
  canSubmit,
  validateStep,
  pickRunPath,
  updateRunPath,
  toggleUseSoftwareDataDir,
  rescanStartupCandidates,
  detectJava,
  handleSubmit,
} = useCreateServerPage();

const router = useRouter();
const route = useRoute();

const coreDownloadTargetPath = ref("");
const coreDownloadCoreType = ref("");
const coreDownloadMcVersion = ref("");
const coreDownloadCoreOptions = ref<string[]>([]);
const coreDownloadMcVersionOptions = ref<string[]>([]);
const coreDownloadLaunching = ref(false);
const coreDownloadResolvedSourcePath = ref("");
const coreDownloadResolvedSavePath = ref("");
const lastCoreDownloadTargetPath = ref("");
const lastCoreDownloadSavePath = ref("");
const lastCoreDownloadCoreType = ref("");
const lastCoreDownloadMcVersion = ref("");

const {
  taskInfo: coreDownloadTaskInfo,
  start: startCoreDownload,
  reset: resetCoreDownload,
  errorMessage: coreDownloadError,
} = downloadApi.useDownload();

const coreTypeSelectOptions = computed(() =>
  coreDownloadCoreOptions.value.map((value) => ({
    value,
    label: value,
  })),
);
const mcVersionSelectOptions = computed(() =>
  coreDownloadMcVersionOptions.value.map((value) => ({
    value,
    label: value,
  })),
);
const isCoreDownloading = computed(
  () =>
    coreDownloadLaunching.value ||
    (coreDownloadTaskInfo.id !== "" && !coreDownloadTaskInfo.isFinished),
);
const isCoreDownloadCompleted = computed(
  () => coreDownloadTaskInfo.isFinished && coreDownloadTaskInfo.status === "Completed",
);

onMounted(async () => {
  try {
    const options = await serverApi.getStarterDownloadOptions();
    coreDownloadCoreOptions.value = options.coreTypeOptions;
    coreDownloadMcVersionOptions.value = options.mcVersionOptions;
  } catch (error) {
    showError(String(error));
  }
});

watch(coreDownloadError, (value) => {
  if (value) {
    showError(value);
  }
});

watch(
  [coreDownloadTargetPath, coreDownloadCoreType, coreDownloadMcVersion],
  ([target, coreType, mcVersion], previous = []) => {
    if (!previous.length) {
      return;
    }
    const [prevTarget, prevCoreType, prevMcVersion] = previous as [string, string, string];
    if (isCoreDownloading.value) {
      return;
    }
    if (target === prevTarget && coreType === prevCoreType && mcVersion === prevMcVersion) {
      return;
    }
    if (coreDownloadTaskInfo.id === "" && !isCoreDownloadCompleted.value) {
      return;
    }

    resetCoreDownload();
    coreDownloadResolvedSourcePath.value = "";
    coreDownloadResolvedSavePath.value = "";
  },
);

watch(isCoreDownloadCompleted, (completed) => {
  if (!completed) {
    return;
  }
  const resolvedPath = coreDownloadResolvedSourcePath.value.trim();
  const resolvedSavePath = coreDownloadResolvedSavePath.value.trim();
  if (!resolvedPath || !resolvedSavePath) {
    return;
  }

  lastCoreDownloadTargetPath.value = resolvedPath;
  lastCoreDownloadSavePath.value = resolvedSavePath;
  lastCoreDownloadCoreType.value = coreDownloadCoreType.value;
  lastCoreDownloadMcVersion.value = coreDownloadMcVersion.value;
  // 方式二下载完成后，自动将下载目录作为来源，允许直接进入下一步。
  sourcePath.value = resolvedPath;
  sourceType.value = "folder";
});

const totalSteps = computed(() => Math.max(stepItems.value.length, 1));

const currentStep = computed(() => {
  const rawStep = Number(route.params.step ?? 1);
  const parsedStep = Number.isFinite(rawStep) ? Math.trunc(rawStep) : 1;
  return parsedStep >= 1 && parsedStep <= totalSteps.value ? parsedStep : 1;
});

function goToStep(step: number) {
  if (step < 1 || step > stepItems.value.length) {
    return;
  }

  if (step > currentStep.value) {
    clearError();
    for (let s = currentStep.value; s < step; s += 1) {
      if (!validateStep(s)) {
        return;
      }
    }
  }

  router.push({
    name: "create-server-step",
    params: { step: String(step) },
  });
}

function goPrevStep() {
  if (currentStep.value <= 1) {
    return;
  }
  void router.push({
    name: "create-server-step",
    params: { step: String(currentStep.value - 1) },
  });
}

function goNextStep() {
  if (currentStep.value >= totalSteps.value) {
    return;
  }
  goToStep(currentStep.value + 1);
}

async function pickCoreDownloadTargetPath() {
  const selected = await systemApi.pickFolder();
  if (selected) {
    coreDownloadTargetPath.value = selected;
  }
}

async function handleCoreDownload() {
  clearError();
  if (
    !coreDownloadTargetPath.value.trim() ||
    !coreDownloadCoreType.value ||
    !coreDownloadMcVersion.value
  ) {
    showError(i18n.t("create.source_method_two_required"));
    return;
  }

  try {
    coreDownloadLaunching.value = true;
    const downloadUrl = await serverApi.resolveStarterDownloadUrl(
      coreDownloadCoreType.value,
      coreDownloadMcVersion.value,
    );

    const normalizedTargetPath = coreDownloadTargetPath.value.trim().replace(/[\\/]+$/, "");
    const fileName = STARTER_SERVER_JAR_NAME;

    const savePath = joinPath(normalizedTargetPath, fileName);
    const sameDownloadFolder =
      normalizePathForMatch(normalizedTargetPath) ===
      normalizePathForMatch(lastCoreDownloadTargetPath.value);
    const sameCoreSelection =
      coreDownloadCoreType.value === lastCoreDownloadCoreType.value &&
      coreDownloadMcVersion.value === lastCoreDownloadMcVersion.value;
    const previousCorePath = lastCoreDownloadSavePath.value.trim();

    if (sameDownloadFolder && !sameCoreSelection && previousCorePath) {
      await systemApi.removeFileIfExists(previousCorePath);
    }

    resetCoreDownload();
    coreDownloadResolvedSourcePath.value = normalizedTargetPath;
    coreDownloadResolvedSavePath.value = savePath;
    await startCoreDownload({
      url: downloadUrl,
      savePath,
    });
  } catch (error) {
    showError(String(error));
  } finally {
    coreDownloadLaunching.value = false;
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
        :model-value="currentStep"
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
          <StepperTrigger class="create-stepper-trigger" @click="goToStep(item.step)">
            <StepperIndicator class="create-stepper-indicator">{{ item.step }}</StepperIndicator>
            <div class="create-stepper-copy">
              <StepperTitle class="create-stepper-title">{{ item.title }}</StepperTitle>
              <StepperDescription class="create-stepper-description">{{
                item.description
              }}</StepperDescription>
            </div>
          </StepperTrigger>

          <div v-if="item.step === currentStep" class="create-step-panel">
            <template v-if="currentStep === 1">
              <SourceIntakeField
                v-model:source-path="sourcePath"
                v-model:source-type="sourceType"
                @error="showError"
              />
              <div class="create-download-card">
                <span class="create-download-card-main">
                  <span class="create-download-card-icon">
                    <Download :size="16" class="create-download-card-icon-svg" />
                  </span>
                  <span class="create-download-card-title">
                    {{ i18n.t("create.source_method_two_title") }}
                  </span>
                </span>

                <div class="create-download-card-form">
                  <SLInput
                    :label="i18n.t('create.source_method_two_target_path')"
                    :model-value="coreDownloadTargetPath"
                    :disabled="isCoreDownloading"
                    @update:model-value="coreDownloadTargetPath = String($event)"
                  >
                    <template #suffix>
                      <button
                        type="button"
                        class="create-download-path-picker"
                        :disabled="isCoreDownloading"
                        @click="pickCoreDownloadTargetPath"
                      >
                        {{ i18n.t("create.source_method_two_pick_path") }}
                      </button>
                    </template>
                  </SLInput>

                  <div class="create-download-card-selects">
                    <div class="create-download-card-field">
                      <p class="create-download-card-label">
                        {{ i18n.t("create.source_method_two_core_type") }}
                      </p>
                      <SLSelect
                        :model-value="coreDownloadCoreType"
                        :options="coreTypeSelectOptions"
                        :disabled="isCoreDownloading || coreTypeSelectOptions.length === 0"
                        searchable
                        max-height="220px"
                        @update:model-value="coreDownloadCoreType = String($event)"
                      />
                    </div>
                    <div class="create-download-card-field">
                      <p class="create-download-card-label">
                        {{ i18n.t("create.source_method_two_game_version") }}
                      </p>
                      <SLSelect
                        :model-value="coreDownloadMcVersion"
                        :options="mcVersionSelectOptions"
                        :disabled="isCoreDownloading || mcVersionSelectOptions.length === 0"
                        searchable
                        max-height="220px"
                        @update:model-value="coreDownloadMcVersion = String($event)"
                      />
                    </div>
                  </div>

                  <div class="create-download-card-action">
                    <SLButton
                      v-if="!isCoreDownloading && !isCoreDownloadCompleted"
                      variant="primary"
                      size="lg"
                      @click="handleCoreDownload"
                    >
                      {{ i18n.t("create.source_method_two_download") }}
                    </SLButton>
                    <SLProgress
                      v-else-if="isCoreDownloading"
                      :value="coreDownloadTaskInfo.progress"
                      :label="i18n.t('create.source_method_two_downloading')"
                    />
                    <p v-else class="create-download-done">
                      {{ i18n.t("create.source_method_two_done") }}
                    </p>
                  </div>
                </div>
              </div>
            </template>

            <RunPathStep
              v-else-if="currentStep === 2"
              :source-type="sourceType"
              :source-path="sourcePath"
              :run-path="runPath"
              :show-overwrite-warning="runPathOverwriteRisk"
              :use-software-data-dir="useSoftwareDataDir"
              :disabled="creating"
              @pick-path="pickRunPath"
              @update:run-path="updateRunPath"
              @toggle-use-software-data-dir="toggleUseSoftwareDataDir"
            />

            <StartupSelectionStep
              v-else-if="currentStep === 3"
              :loading="startupDetecting"
              :candidates="startupCandidates"
              :selected-startup-id="selectedStartupId"
              :custom-startup-command="customStartupCommand"
              :custom-command-has-redirect="customCommandHasRedirect"
              :starter-selected="starterSelected"
              :core-detecting="coreDetecting"
              :detected-core-type-key="detectedCoreTypeKey"
              :core-type-options="coreTypeOptions"
              :selected-core-type="selectedCoreType"
              :detected-mc-version="detectedMcVersion"
              :mc-version-options="mcVersionOptions"
              :selected-mc-version="selectedMcVersion"
              :mc-version-detection-failed="mcVersionDetectionFailed"
              :disabled="creating"
              @rescan="rescanStartupCandidates"
              @update:selected-startup-id="selectedStartupId = $event"
              @update:custom-startup-command="customStartupCommand = $event"
              @update:selected-core-type="selectedCoreType = $event"
              @update:selected-mc-version="selectedMcVersion = $event"
            />

            <template v-else-if="currentStep === 4">
              <div class="create-config-step">
                <JavaEnvironmentStep
                  :java-list="javaList"
                  :loading="javaLoading"
                  :selected-java="selectedJava"
                  @detect="detectJava"
                  @update:selected-java="selectedJava = $event"
                />

                <ServerStartupConfigStep
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
              </div>
            </template>

            <div v-if="currentStep < totalSteps" class="create-nav-actions">
              <SLButton
                variant="secondary"
                size="lg"
                :disabled="currentStep === 1"
                @click="goPrevStep"
              >
                {{ i18n.t("create.prev_step") }}
              </SLButton>
              <SLButton variant="primary" size="lg" @click="goNextStep">
                {{ i18n.t("create.next_step") }}
              </SLButton>
            </div>

            <div v-else class="create-submit-actions">
              <SLButton variant="secondary" size="lg" @click="goPrevStep">
                {{ i18n.t("create.prev_step") }}
              </SLButton>
              <SLButton variant="secondary" size="lg" @click="router.push('/')">
                {{ i18n.t("create.cancel") }}
              </SLButton>
              <SLButton
                variant="primary"
                size="lg"
                :loading="creating"
                :disabled="!canSubmit || creating"
                @click="handleSubmit"
              >
                {{ i18n.t("create.create") }}
              </SLButton>
            </div>
          </div>

          <StepperSeparator v-if="item.step < stepItems.length" class="create-stepper-separator" />
        </StepperItem>
      </StepperRoot>
    </SLCard>
  </div>
</template>

<style src="@styles/views/CreateServerView.css" scoped></style>
