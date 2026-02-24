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
import { useRouter } from "vue-router";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import CopyConflictDialog from "@components/views/create/CopyConflictDialog.vue";
import JavaEnvironmentStep from "@components/views/create/JavaEnvironmentStep.vue";
import RunPathStep from "@components/views/create/RunPathStep.vue";
import ServerStartupConfigStep from "@components/views/create/ServerStartupConfigStep.vue";
import SourceIntakeField from "@components/views/create/SourceIntakeField.vue";
import StartupSelectionStep from "@components/views/create/StartupSelectionStep.vue";
import { i18n } from "@language";
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
  coreDetecting,
  startupDetecting,
  startupCandidates,
  selectedStartupId,
  customStartupCommand,
  customCommandHasRedirect,
  copyConflictDialogOpen,
  copyConflictItems,
  serverName,
  maxMemory,
  minMemory,
  port,
  selectedJava,
  onlineMode,
  javaList,
  activeStep,
  stepItems,
  canSubmit,
  showSourceCoreInfo,
  sourceCoreInfoText,
  pickRunPath,
  rescanStartupCandidates,
  detectJava,
  handleSubmit,
  confirmCopyConflict,
  cancelCopyConflict,
} = useCreateServerPage();

const router = useRouter();
</script>

<template>
  <div class="create-view animate-fade-in-up">
    <div v-if="errorMsg" class="create-error-banner">
      <span>{{ errorMsg }}</span>
      <button class="create-error-close" @click="clearError">x</button>
    </div>

    <SLCard class="create-stepper-card" :title="i18n.t('create.title')">
      <StepperRoot orientation="vertical" :model-value="activeStep" :linear="false" class="create-stepper">
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
              <StepperDescription class="create-stepper-description">{{ item.description }}</StepperDescription>
            </div>
          </StepperTrigger>

          <div class="create-step-panel">
            <template v-if="item.step === 1">
              <SourceIntakeField v-model:source-path="sourcePath" v-model:source-type="sourceType" @error="showError" />
              <p v-if="showSourceCoreInfo" class="create-source-core-info">
                {{ coreDetecting ? i18n.t("create.source_detecting_core") : sourceCoreInfoText }}
              </p>
            </template>

            <RunPathStep
              v-else-if="item.step === 2"
              :source-type="sourceType"
              :source-path="sourcePath"
              :run-path="runPath"
              :disabled="creating"
              @pick-path="pickRunPath"
              @update:run-path="runPath = $event"
            />

            <StartupSelectionStep
              v-else-if="item.step === 3"
              :loading="startupDetecting"
              :candidates="startupCandidates"
              :selected-startup-id="selectedStartupId"
              :custom-startup-command="customStartupCommand"
              :custom-command-has-redirect="customCommandHasRedirect"
              :disabled="creating"
              @rescan="rescanStartupCandidates"
              @update:selected-startup-id="selectedStartupId = $event"
              @update:custom-startup-command="customStartupCommand = $event"
            />

            <template v-else>
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

                <div class="create-submit-actions">
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
                    {{ i18n.t("create.create_config_only") }}
                  </SLButton>
                </div>
              </div>
            </template>
          </div>

          <StepperSeparator v-if="item.step < stepItems.length" class="create-stepper-separator" />
        </StepperItem>
      </StepperRoot>
    </SLCard>

    <CopyConflictDialog
      :open="copyConflictDialogOpen"
      :items="copyConflictItems"
      @update:open="copyConflictDialogOpen = $event"
      @confirm="confirmCopyConflict"
      @cancel="cancelCopyConflict"
    />
  </div>
</template>

<style src="@styles/views/CreateServerView.css" scoped></style>
