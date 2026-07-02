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
import { computed, ref, watch } from "vue";
import { useRouter } from "vue-router";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import JavaEnvironmentStep from "@next-src/components/servers/create/JavaEnvironmentStep.vue";
import ServerStartupConfigStep from "@next-src/components/servers/create/ServerStartupConfigStep.vue";
import StartupSelectionStep from "@next-src/components/servers/create/StartupSelectionStep.vue";
import { javaApi, type JavaInfo } from "@api/java";
import { serverApi } from "@api/server";
import { systemApi } from "@api/system";
import type { StartupCandidate, StartupMode } from "@next-src/pages/servers/create/startupTypes";
import {
  detectVersionCandidatesFromText,
  startupModeRequiresJava,
} from "@next-src/pages/servers/create/startupUtils";
import type { CpuPolicyConfig, JvmPresetConfig } from "@type/server";
import { useLoading } from "@composables/useAsync";
import { useMessage } from "@composables/useMessage";
import { i18n } from "@language";
import { useServerStore } from "@stores/serverStore";
import {
  resolveCreatedServerRoute,
  resolveServerCreationCancelRoute,
} from "@utils/serverCreationNavigation";
import {
  createDefaultCpuPolicy,
  createDefaultJvmPreset,
  getCpuPolicyValidationError,
  normalizeCpuPolicy,
  normalizeJvmPreset,
  serializeJvmArgsText,
} from "@utils/serverStartupConfig";

function parseNumber(value: string, fallbackValue: number): number {
  const parsed = Number.parseInt(value, 10);
  return Number.isNaN(parsed) ? fallbackValue : parsed;
}

function getLastPathSegment(path: string): string {
  const segments = path.split(/[\\/]/);
  for (let index = segments.length - 1; index >= 0; index -= 1) {
    const segment = segments[index]?.trim();
    if (segment) {
      return segment;
    }
  }
  return "";
}

function buildFallbackCandidates(folderPath: string): StartupCandidate[] {
  const normalized = folderPath.replace(/\\/g, "/").replace(/\/$/, "");
  return [
    {
      id: "fallback-bat",
      mode: "bat",
      label: i18n.t("add_existing.startup_mode_bat_label"),
      detail: `${normalized}/start.bat`,
      path: `${normalized}/start.bat`,
      recommended: 1,
    },
    {
      id: "fallback-sh",
      mode: "sh",
      label: i18n.t("add_existing.startup_mode_sh_label"),
      detail: `${normalized}/start.sh`,
      path: `${normalized}/start.sh`,
      recommended: 1,
    },
    {
      id: "fallback-ps1",
      mode: "ps1",
      label: i18n.t("add_existing.startup_mode_ps1_label"),
      detail: `${normalized}/start.ps1`,
      path: `${normalized}/start.ps1`,
      recommended: 2,
    },
    {
      id: "fallback-jar",
      mode: "jar",
      label: i18n.t("add_existing.startup_mode_jar_label"),
      detail: `${normalized}/server.jar`,
      path: `${normalized}/server.jar`,
      recommended: 3,
    },
  ];
}

function prioritizeDetectedOption(values: string[], detected: string): string[] {
  const unique = [...new Set(values.filter((value) => value.trim().length > 0))];
  if (!detected.trim()) {
    return unique;
  }

  return [detected, ...unique.filter((value) => value !== detected)];
}

function resolveDetectedMcVersion(
  path: string,
  discoveredOptions: string[],
  apiDetectedVersion: string,
  candidates: StartupCandidate[],
): { detected: string; failed: boolean } {
  if (apiDetectedVersion.trim()) {
    return { detected: apiDetectedVersion.trim(), failed: false };
  }

  const fallbackInputs = [path, ...candidates.map((candidate) => candidate.path)];
  for (const input of fallbackInputs) {
    const matches = detectVersionCandidatesFromText(input, discoveredOptions);
    if (matches.length > 0) {
      return { detected: matches[0], failed: false };
    }
  }

  return { detected: "", failed: discoveredOptions.length > 0 };
}

function isExistingServerStartupMode(
  mode: StartupMode,
): mode is "jar" | "bat" | "sh" | "ps1" | "custom" {
  return mode !== "starter";
}

const router = useRouter();
const serverStore = useServerStore();
const { error: errorMsg, showError, clearError } = useMessage();
const { loading: javaLoading, start: startJavaLoading, stop: stopJavaLoading } = useLoading();
const { loading: creating, start: startCreating, stop: stopCreating } = useLoading();

const serverPath = ref("");
const serverName = ref("My Server");
const selectedJava = ref("");
const javaList = ref<JavaInfo[]>([]);
const maxMemory = ref("2048");
const minMemory = ref("512");
const port = ref("25565");
const onlineMode = ref(true);
const jvmArgsText = ref("");
const jvmPreset = ref<JvmPresetConfig>(createDefaultJvmPreset());
const cpuPolicy = ref<CpuPolicyConfig>(createDefaultCpuPolicy());

const startupCandidates = ref<StartupCandidate[]>([]);
const selectedStartupId = ref("");
const startupDetecting = ref(false);
const detectedCoreTypeKey = ref("");
const coreTypeOptions = ref<string[]>([]);
const selectedCoreType = ref("");
const detectedMcVersion = ref("");
const mcVersionOptions = ref<string[]>([]);
const selectedMcVersion = ref("");
const mcVersionDetectionFailed = ref(false);

const selectedStartup = computed(() => {
  const startup = startupCandidates.value.find((item) => item.id === selectedStartupId.value);
  return startup ?? null;
});
const selectedStartupTarget = computed(() => {
  const startup = selectedStartup.value;
  if (!startup) {
    return "";
  }

  if (startup.mode === "custom") {
    return startup.path || startup.detail;
  }

  return startup.path;
});

const requiresJava = computed(() => startupModeRequiresJava(selectedStartup.value?.mode));

const stepItems = computed(() => [
  {
    step: 1,
    title: i18n.t("add_existing.step_folder_title"),
    description: i18n.t("add_existing.step_folder_desc"),
    completed: serverPath.value.trim().length > 0,
  },
  {
    step: 2,
    title: i18n.t("add_existing.step_startup_title"),
    description: i18n.t("add_existing.step_startup_desc"),
    completed: selectedStartup.value !== null,
  },
  {
    step: 3,
    title: i18n.t("add_existing.step_config_title"),
    description: i18n.t("add_existing.step_config_desc"),
    completed:
      (!requiresJava.value || selectedJava.value.trim().length > 0) &&
      serverName.value.trim().length > 0,
  },
  {
    step: 4,
    title: i18n.t("add_existing.step_action_title"),
    description: i18n.t("add_existing.step_action_desc"),
    completed: false,
  },
]);

const activeStep = computed(() => {
  if (serverPath.value.trim().length === 0) {
    return 1;
  }
  if (!selectedStartup.value) {
    return 2;
  }
  if (
    (requiresJava.value && selectedJava.value.trim().length === 0) ||
    serverName.value.trim().length === 0
  ) {
    return 3;
  }
  return 4;
});

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

const canSubmit = computed(
  () =>
    serverPath.value.trim().length > 0 &&
    selectedStartup.value !== null &&
    (!requiresJava.value || selectedJava.value.trim().length > 0) &&
    serverName.value.trim().length > 0 &&
    !startupDetecting.value,
);

async function detectJava() {
  startJavaLoading();
  try {
    javaList.value = await javaApi.detect();
    if (javaList.value.length > 0) {
      const preferredJava = javaList.value.find(
        (java) => java.is_64bit && java.major_version >= 17,
      );
      selectedJava.value = preferredJava ? preferredJava.path : javaList.value[0].path;
    }
  } catch (error) {
    showError(String(error));
  } finally {
    stopJavaLoading();
  }
}

async function pickServerFolder() {
  const selected = await systemApi.pickFolder();
  if (!selected) {
    return;
  }

  serverPath.value = selected;
  const folderName = getLastPathSegment(selected);
  if (folderName) {
    serverName.value = folderName;
  }
}

async function refreshStartupCandidates() {
  const trimmedPath = serverPath.value.trim();
  if (!trimmedPath) {
    startupCandidates.value = [];
    selectedStartupId.value = "";
    detectedCoreTypeKey.value = "";
    coreTypeOptions.value = [];
    selectedCoreType.value = "";
    detectedMcVersion.value = "";
    mcVersionOptions.value = [];
    selectedMcVersion.value = "";
    mcVersionDetectionFailed.value = false;
    return;
  }

  startupDetecting.value = true;
  try {
    const scanResult = await serverApi.scanStartupCandidates(trimmedPath, "folder");
    const filtered = scanResult.candidates.filter((candidate) => candidate.mode !== "starter");
    startupCandidates.value = filtered.length > 0 ? filtered : buildFallbackCandidates(trimmedPath);
    selectedStartupId.value = startupCandidates.value[0]?.id ?? "";

    detectedCoreTypeKey.value = scanResult.detectedCoreTypeKey ?? "";
    coreTypeOptions.value = prioritizeDetectedOption(
      scanResult.coreTypeOptions,
      detectedCoreTypeKey.value,
    );
    if (!coreTypeOptions.value.includes(selectedCoreType.value)) {
      selectedCoreType.value = detectedCoreTypeKey.value;
    }

    const detectedMcVersionResult = resolveDetectedMcVersion(
      trimmedPath,
      scanResult.mcVersionOptions,
      scanResult.detectedMcVersion ?? "",
      startupCandidates.value,
    );
    detectedMcVersion.value = detectedMcVersionResult.detected;
    mcVersionOptions.value = prioritizeDetectedOption(
      scanResult.mcVersionOptions,
      detectedMcVersion.value,
    );
    mcVersionDetectionFailed.value = detectedMcVersionResult.failed;
    if (!mcVersionOptions.value.includes(selectedMcVersion.value)) {
      selectedMcVersion.value = detectedMcVersion.value;
    }
  } catch (error) {
    console.warn("Failed to scan startup candidates for existing server:", error);
    startupCandidates.value = buildFallbackCandidates(trimmedPath);
    selectedStartupId.value = startupCandidates.value[0]?.id ?? "";
    detectedCoreTypeKey.value = "";
    coreTypeOptions.value = [];
    selectedCoreType.value = "";
    detectedMcVersion.value = "";
    mcVersionOptions.value = [];
    selectedMcVersion.value = "";
    mcVersionDetectionFailed.value = false;
  } finally {
    startupDetecting.value = false;
  }
}

function validateBeforeSubmit(): boolean {
  clearError();

  if (!serverPath.value.trim()) {
    showError(i18n.t("add_existing.select_server_folder"));
    return false;
  }
  if (!selectedStartup.value) {
    showError(i18n.t("create.startup_required"));
    return false;
  }
  if (requiresJava.value && !selectedJava.value.trim()) {
    showError(i18n.t("common.select_java_path"));
    return false;
  }
  if (!serverName.value.trim()) {
    showError(i18n.t("common.enter_server_name"));
    return false;
  }

  const cpuPolicyError = getCpuPolicyValidationError(cpuPolicy.value);
  if (cpuPolicyError) {
    showError(i18n.t(`create.cpu_policy_invalid_${cpuPolicyError}`));
    return false;
  }

  return true;
}

async function handleSubmit() {
  if (!validateBeforeSubmit()) {
    return;
  }

  startCreating();
  try {
    const startup = selectedStartup.value;
    if (!startup) {
      return;
    }
    if (!isExistingServerStartupMode(startup.mode)) {
      showError(i18n.t("create.startup_required"));
      return;
    }

    const addedServer = await serverApi.addExistingServer({
      name: serverName.value.trim(),
      serverPath: serverPath.value.trim(),
      javaPath: requiresJava.value ? selectedJava.value.trim() : "",
      maxMemory: parseNumber(maxMemory.value, 2048),
      minMemory: parseNumber(minMemory.value, 512),
      port: parseNumber(port.value, 25565),
      startupMode: startup.mode,
      executablePath:
        startup.mode === "custom" ? undefined : startup.path.trim() ? startup.path : undefined,
      customCommand: startup.mode === "custom" ? startup.path.trim() || undefined : undefined,
      coreType: selectedCoreType.value.trim() || detectedCoreTypeKey.value.trim() || undefined,
      mcVersion: selectedMcVersion.value.trim() || detectedMcVersion.value.trim() || undefined,
      jvmArgs: serializeJvmArgsText(jvmArgsText.value),
      cpuPolicy: normalizeCpuPolicy(cpuPolicy.value),
      jvmPreset: normalizeJvmPreset(jvmPreset.value),
    });

    await serverStore.refreshList();
    serverStore.setCurrentServer(addedServer.id);
    await router.push(resolveCreatedServerRoute(router.currentRoute.value, addedServer.id));
  } catch (error) {
    showError(String(error));
  } finally {
    stopCreating();
  }
}

async function handleCancel() {
  await router.push(resolveServerCreationCancelRoute(router.currentRoute.value));
}

watch(
  () => serverPath.value,
  () => {
    void refreshStartupCandidates();
  },
);
</script>

<template>
  <div class="create-view animate-fade-in-up">
    <div v-if="errorMsg" class="create-error-banner">
      <span>{{ errorMsg }}</span>
      <button class="create-error-close" @click="clearError">x</button>
    </div>

    <SLCard
      class="create-stepper-card"
      :title="i18n.t('add_existing.title')"
      :subtitle="i18n.t('add_existing.page_subtitle')"
    >
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
              <StepperDescription class="create-stepper-description">
                {{ item.description }}
              </StepperDescription>
            </div>
          </StepperTrigger>

          <div v-if="item.step === currentStep" class="create-step-panel">
            <template v-if="item.step === 1">
              <div class="existing-folder-card">
                <div class="existing-folder-copy">
                  <p class="existing-folder-title">{{ i18n.t("add_existing.server_folder") }}</p>
                  <p class="existing-folder-desc">
                    {{ i18n.t("add_existing.server_folder_hint") }}
                  </p>
                </div>
                <div class="existing-folder-input-row" :class="{ 'is-empty': !serverPath }">
                  <div class="existing-folder-path">
                    {{ serverPath || i18n.t("add_existing.server_folder_placeholder") }}
                  </div>
                  <SLButton variant="primary" size="lg" @click="pickServerFolder">
                    {{ i18n.t("add_existing.browse") }}
                  </SLButton>
                </div>

                <div class="create-step-actions create-step-actions--end">
                  <SLButton variant="secondary" size="lg" @click="handleCancel">
                    {{ i18n.t("create.cancel") }}
                  </SLButton>
                  <SLButton variant="primary" size="lg" :disabled="!canGoNext" @click="goNext">
                    {{ i18n.t("common.next_step") }}
                  </SLButton>
                </div>
              </div>
            </template>

            <template v-else-if="item.step === 2">
              <StartupSelectionStep
                :loading="startupDetecting"
                :candidates="startupCandidates"
                :selected-startup-id="selectedStartupId"
                :custom-startup-command="''"
                :custom-command-has-redirect="false"
                :starter-selected="false"
                :core-detecting="startupDetecting"
                :detected-core-type-key="detectedCoreTypeKey"
                :core-type-options="coreTypeOptions"
                :selected-core-type="selectedCoreType"
                :detected-mc-version="detectedMcVersion"
                :mc-version-options="mcVersionOptions"
                :selected-mc-version="selectedMcVersion"
                :mc-version-detection-failed="mcVersionDetectionFailed"
                :show-advanced-details="true"
                :disabled="creating"
                @rescan="refreshStartupCandidates"
                @update:selected-startup-id="selectedStartupId = $event"
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

            <template v-else-if="item.step === 3">
              <div class="create-config-step">
                <JavaEnvironmentStep
                  :java-list="javaList"
                  :loading="javaLoading"
                  :required="requiresJava"
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
                  :startup-target="selectedStartupTarget"
                  :show-online-mode="false"
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
                  {{ i18n.t("add_existing.add_server") }}
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

<style scoped>
.existing-folder-card {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}

.existing-folder-copy {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
}

.existing-folder-title {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}

.existing-folder-desc {
  margin: 0;
  color: var(--sl-text-secondary);
  line-height: 1.6;
}

.existing-folder-path {
  flex: 1;
  min-height: 56px;
  display: flex;
  align-items: center;
  padding: var(--sl-space-md);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  background: var(--sl-bg-secondary);
  color: var(--sl-text-primary);
  font-family: var(--sl-font-mono);
  word-break: break-all;
}

.existing-folder-input-row {
  display: flex;
  align-items: stretch;
  gap: var(--sl-space-md);
}

.existing-folder-input-row.is-empty .existing-folder-path {
  color: var(--sl-text-tertiary);
}

@media (max-width: 900px) {
  .existing-folder-input-row {
    flex-direction: column;
  }
}
</style>
