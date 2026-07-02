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
import { FileUp } from "@lucide/vue";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import JavaEnvironmentStep from "@components/views/create/JavaEnvironmentStep.vue";
import RunPathStep from "@components/views/create/RunPathStep.vue";
import ServerStartupConfigStep from "@components/views/create/ServerStartupConfigStep.vue";
import SourceIntakeField from "@components/views/create/SourceIntakeField.vue";
import StartupSelectionStep from "@components/views/create/StartupSelectionStep.vue";
import { useCreateServerWindowDrop } from "@components/views/create/useCreateServerWindowDrop";
import { startupModeRequiresJava } from "@components/views/create/startupUtils";
import { i18n } from "@language";
import { useCreateServerPage } from "@components/views/create/useCreateServerPage";
import { computed, ref, watch } from "vue";

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
  selectedStartup,
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
  jvmArgsText,
  jvmPreset,
  cpuPolicy,
  javaList,
  activeStep,
  stepItems,
  canSubmit,
  pickRunPath,
  updateRunPath,
  rescanStartupCandidates,
  detectJava,
  handleCancel,
  handleSubmit,
} = useCreateServerPage();

const { isDragging } = useCreateServerWindowDrop();

const lastStep = computed(() => stepItems.value.length);
const currentStep = ref(activeStep.value);

watch(
  activeStep,
  (nextStep) => {
    if (currentStep.value > nextStep) {
      currentStep.value = nextStep;
      return;
    }

    if (currentStep.value < 1) {
      currentStep.value = 1;
    }
  },
  { immediate: true },
);

const canGoPrevious = computed(() => currentStep.value > 1);
const canGoNext = computed(
  () => currentStep.value < lastStep.value && currentStep.value < activeStep.value,
);

function goPrevious() {
  if (!canGoPrevious.value) {
    return;
  }
  currentStep.value -= 1;
}

function goNext() {
  if (!canGoNext.value) {
    return;
  }
  currentStep.value += 1;
}
</script>

<template>
  <div class="create-view animate-fade-in-up">
    <!-- 拖放提示遮罩 -->
    <div v-if="isDragging" class="create-drop-overlay">
      <div class="drop-hint">
        <FileUp :size="48" />
        <p>{{ i18n.t("create.drop_hint") }}</p>
      </div>
    </div>

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
          <StepperTrigger class="create-stepper-trigger">
            <StepperIndicator class="create-stepper-indicator">{{ item.step }}</StepperIndicator>
            <div class="create-stepper-copy">
              <StepperTitle class="create-stepper-title">{{ item.title }}</StepperTitle>
              <StepperDescription class="create-stepper-description">{{
                item.description
              }}</StepperDescription>
            </div>
          </StepperTrigger>

          <div v-if="item.step === currentStep" class="create-step-panel">
            <template v-if="item.step === 1">
              <SourceIntakeField
                v-model:source-path="sourcePath"
                v-model:source-type="sourceType"
                @error="showError"
              />

              <div class="create-step-actions create-step-actions--end">
                <SLButton variant="secondary" size="lg" @click="handleCancel">
                  {{ i18n.t("create.cancel") }}
                </SLButton>
                <SLButton variant="primary" size="lg" :disabled="!canGoNext" @click="goNext">
                  {{ i18n.t("common.next_step") }}
                </SLButton>
              </div>
            </template>

            <template v-else-if="item.step === 2">
              <RunPathStep
                :source-type="sourceType"
                :source-path="sourcePath"
                :run-path="runPath"
                :show-overwrite-warning="runPathOverwriteRisk"
                :disabled="creating"
                @pick-path="pickRunPath"
                @update:run-path="updateRunPath"
              />

              <div class="create-step-actions">
                <SLButton variant="secondary" size="lg" @click="goPrevious">
                  {{ i18n.t("common.previous_step") }}
                </SLButton>
                <SLButton variant="primary" size="lg" :disabled="!canGoNext" @click="goNext">
                  {{ i18n.t("common.next_step") }}
                </SLButton>
              </div>
            </template>

            <template v-else-if="item.step === 3">
              <StartupSelectionStep
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

              <div class="create-step-actions">
                <SLButton variant="secondary" size="lg" @click="goPrevious">
                  {{ i18n.t("common.previous_step") }}
                </SLButton>
                <SLButton variant="primary" size="lg" :disabled="!canGoNext" @click="goNext">
                  {{ i18n.t("common.next_step") }}
                </SLButton>
              </div>
            </template>

            <template v-else-if="item.step === 4">
              <div class="create-config-step">
                <JavaEnvironmentStep
                  :java-list="javaList"
                  :loading="javaLoading"
                  :required="startupModeRequiresJava(selectedStartup?.mode)"
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
                  :jvm-args-text="jvmArgsText"
                  :jvm-preset="jvmPreset.preset"
                  :cpu-policy="cpuPolicy"
                  :startup-mode="selectedStartup?.mode"
                  :startup-target="selectedStartup?.path"
                  :custom-command-preview="customStartupCommand"
                  :disabled="creating"
                  @update:server-name="serverName = $event"
                  @update:max-memory="maxMemory = $event"
                  @update:min-memory="minMemory = $event"
                  @update:port="port = $event"
                  @update:online-mode="onlineMode = $event"
                  @update:jvm-args-text="jvmArgsText = $event"
                  @update:jvm-preset="jvmPreset = { preset: $event }"
                  @update:cpu-policy="cpuPolicy = $event"
                />

                <div class="create-step-actions">
                  <SLButton variant="secondary" size="lg" @click="goPrevious">
                    {{ i18n.t("common.previous_step") }}
                  </SLButton>
                  <SLButton variant="primary" size="lg" :disabled="!canGoNext" @click="goNext">
                    {{ i18n.t("common.next_step") }}
                  </SLButton>
                </div>
              </div>
            </template>

            <template v-else>
              <div class="create-submit-actions">
                <SLButton variant="secondary" size="lg" @click="goPrevious">
                  {{ i18n.t("common.previous_step") }}
                </SLButton>
                <SLButton variant="secondary" size="lg" @click="handleCancel">
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
              <div v-if="errorMsg" class="create-submit-error">
                {{ errorMsg }}
              </div>
            </template>
          </div>

          <StepperSeparator v-if="item.step < stepItems.length" class="create-stepper-separator" />
        </StepperItem>
      </StepperRoot>
    </SLCard>
  </div>
</template>

<style src="@styles/views/CreateServerView.css" scoped></style>
