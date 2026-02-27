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
import { computed } from "vue";
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
import { i18n } from "@language";
import { useCreateServerPage } from "@components/views/create/useCreateServerPage";
import { useStarterCoreDownload } from "@components/views/create/useStarterCoreDownload";

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
  rescanStartupCandidates,
  detectJava,
  handleSubmit,
} = useCreateServerPage();

const router = useRouter();
const route = useRoute();

const {
  coreDownloadTargetPath,
  coreDownloadCoreType,
  coreDownloadMcVersion,
  coreDownloadTaskInfo,
  coreTypeSelectOptions,
  mcVersionSelectOptions,
  isCoreDownloading,
  isCoreDownloadCompleted,
  isStarterDownloadControlDisabled,
  starterOptionsUnavailableMessage,
  pickCoreDownloadTargetPath,
  handleCoreDownload,
} = useStarterCoreDownload({
  sourcePath,
  sourceType,
  clearError,
  showError,
});

const stepSequence = computed(() =>
  stepItems.value.map((item) => item.step).sort((left, right) => left - right),
);
const firstStep = computed(() => stepSequence.value[0] ?? 1);
const totalSteps = computed(() => stepSequence.value[stepSequence.value.length - 1] ?? 1);
const validStepSet = computed(() => new Set(stepSequence.value));

const currentStep = computed(() => {
  const rawStep = Number(route.params.step ?? firstStep.value);
  const parsedStep = Number.isFinite(rawStep) ? Math.trunc(rawStep) : firstStep.value;
  return validStepSet.value.has(parsedStep) ? parsedStep : firstStep.value;
});

function goToStep(step: number) {
  if (!validStepSet.value.has(step)) {
    return;
  }

  if (step > currentStep.value) {
    clearError();
    const pendingSteps = stepSequence.value.filter(
      (current) => current >= currentStep.value && current < step,
    );
    for (const s of pendingSteps) {
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
  const currentIndex = stepSequence.value.indexOf(currentStep.value);
  if (currentIndex <= 0) {
    return;
  }
  const previousStep = stepSequence.value[currentIndex - 1];
  void router.push({
    name: "create-server-step",
    params: { step: String(previousStep) },
  });
}

function goNextStep() {
  const currentIndex = stepSequence.value.indexOf(currentStep.value);
  if (currentIndex < 0 || currentIndex >= stepSequence.value.length - 1) {
    return;
  }
  const nextStep = stepSequence.value[currentIndex + 1];
  goToStep(nextStep);
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
                    :disabled="isStarterDownloadControlDisabled"
                    @update:model-value="coreDownloadTargetPath = String($event)"
                  >
                    <template #suffix>
                      <button
                        type="button"
                        class="create-download-path-picker"
                        :disabled="isStarterDownloadControlDisabled"
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
                        :disabled="isStarterDownloadControlDisabled"
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
                        :disabled="isStarterDownloadControlDisabled"
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
                      :disabled="isStarterDownloadControlDisabled"
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
                  <p v-if="starterOptionsUnavailableMessage" class="create-download-unavailable">
                    {{ starterOptionsUnavailableMessage }}
                  </p>
                </div>
              </div>
            </template>

            <RunPathStep
              v-else-if="currentStep === 2"
              :source-type="sourceType"
              :source-path="sourcePath"
              :run-path="runPath"
              :show-overwrite-warning="runPathOverwriteRisk"
              :disabled="creating"
              @pick-path="pickRunPath"
              @update:run-path="updateRunPath"
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

          <StepperSeparator v-if="item.step < totalSteps" class="create-stepper-separator" />
        </StepperItem>
      </StepperRoot>
    </SLCard>
  </div>
</template>

<style src="@styles/views/CreateServerView.css" scoped></style>
