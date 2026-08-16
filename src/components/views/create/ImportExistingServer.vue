<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ArrowLeft, FolderSymlink, FolderSearch, Loader2 } from "lucide-vue-next";
import { useToast } from "cmzya-modern-ui";
import { systemApi } from "@api/system";
import { rpcInvoke } from "@api/rpc";
import { useServerStore } from "@stores/serverStore";
import { i18n } from "@language";

const emit = defineEmits<{
  (e: "back"): void;
  (e: "imported"): void;
}>();

const toast = useToast();
const serverStore = useServerStore();

/* ── 检查报告类型（仅取前端需要的字段）── */
interface Detected<T> {
  value: T | null;
  confidence: number;
}
interface LaunchTarget {
  kind: "jar" | "script" | "main_class" | "argument_files";
  path?: string;
  class_name?: string;
  paths?: string[];
}
interface LaunchProfile {
  id: string;
  platform: string;
  working_directory: string | null;
  target: LaunchTarget;
  jvm_arguments: string[];
  program_arguments: string[];
  required_java_major: number | null;
}
interface Attributed<T> {
  value: T;
  confidence: number;
}
interface ServerProduct {
  key: string;
  display_name: string;
}
interface ServerInspectionReport {
  identity: {
    category: Detected<string>;
    implementation: Detected<ServerProduct>;
    version: Detected<string>;
  };
  minecraft: { version: Detected<string> } | null;
  java: { required_major: Detected<number>; runtime_component: Detected<string> };
  launches: Attributed<LaunchProfile>[];
}

/* ── 状态 ── */
const selectedDir = ref("");
const scanning = ref(false);
const importing = ref(false);
const report = ref<ServerInspectionReport | null>(null);
const scanError = ref("");

const name = ref("");
const port = ref<number | null>(25565);
const maxMemory = ref<number | null>(4096);
const minMemory = ref<number | null>(1024);
const javaExecutable = ref("");
const jvmArgumentsText = ref("");
const selectedLaunchId = ref("");

const launchOptions = computed(() =>
  (report.value?.launches ?? []).map((attributed) => ({
    value: attributed.value.id,
    label: formatLaunchTarget(attributed.value.target),
  })),
);

const recognized = computed(() => report.value !== null);

const hasLaunchOptions = computed(() => launchOptions.value.length > 0);

const canSubmit = computed(
  () =>
    !!selectedDir.value &&
    !scanning.value &&
    !importing.value &&
    recognized.value &&
    hasLaunchOptions.value,
);

const recognizedImpl = computed(
  () => report.value?.identity.implementation.value?.display_name || "",
);
const recognizedVersion = computed(() => report.value?.minecraft?.version.value || "");
const recognizedJava = computed(() => {
  const major = report.value?.java.required_major.value;
  return major ? `Java ${major}` : "";
});

function folderName(path: string): string {
  const segments = path.split(/[\\/]/).filter(Boolean);
  return segments.length > 0 ? segments[segments.length - 1] : path;
}

function formatLaunchTarget(target: LaunchTarget): string {
  switch (target.kind) {
    case "jar":
      return `JAR · ${target.path ?? ""}`;
    case "script":
      return `脚本 · ${target.path ?? ""}`;
    case "main_class":
      return `主类 · ${target.class_name ?? ""}`;
    case "argument_files":
      return `参数文件 · ${(target.paths ?? []).join(", ")}`;
    default:
      return target.kind;
  }
}

async function pickDir() {
  const selected = await systemApi.pickFolder();
  if (selected) {
    selectedDir.value = selected;
  }
}

watch(selectedDir, async (dir) => {
  if (!dir) {
    report.value = null;
    scanError.value = "";
    return;
  }
  await inspect(dir);
});

async function inspect(dir: string) {
  scanning.value = true;
  scanError.value = "";
  report.value = null;
  try {
    const result = await rpcInvoke<ServerInspectionReport>("provisioning.inspect", {
      path: dir,
    });
    report.value = result;
    name.value = folderName(dir);
    const firstLaunch = result.launches[0]?.value;
    selectedLaunchId.value = firstLaunch?.id ?? "";
  } catch (error) {
    scanError.value = friendlyMessage(error);
  } finally {
    scanning.value = false;
  }
}

async function submit() {
  if (!canSubmit.value) return;
  importing.value = true;
  try {
    const request: Record<string, unknown> = {
      source_directory: selectedDir.value,
    };
    if (name.value.trim()) request.name = name.value.trim();
    if (port.value != null) request.port = port.value;
    if (maxMemory.value != null) request.max_memory_mib = maxMemory.value;
    if (minMemory.value != null) request.min_memory_mib = minMemory.value;
    if (javaExecutable.value.trim()) request.java_executable = javaExecutable.value.trim();
    const jvm = jvmArgumentsText.value
      .split(/[\s,]+/)
      .map((item) => item.trim())
      .filter(Boolean);
    if (jvm.length) request.jvm_arguments = jvm;
    if (selectedLaunchId.value) request.selected_launch_profile_id = selectedLaunchId.value;

    await rpcInvoke("provisioning.importExisting", { request });
    await serverStore.refreshList();
    toast.success(i18n.t("create.import_success"));
    emit("imported");
  } catch (error) {
    toast.error(friendlyMessage(error));
  } finally {
    importing.value = false;
  }
}

/** Tauri 传输下错误可能被序列化为 {code,message} 的 JSON 字符串，尽量取出友好消息。 */
function friendlyMessage(error: unknown): string {
  const raw = error instanceof Error ? error.message : String(error);
  try {
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed.message === "string") return parsed.message;
  } catch {
    /* 非 JSON，按原文返回 */
  }
  return raw;
}
</script>

<template>
  <div class="import-existing-view">
    <cmz-card class="import-existing-card" :title="i18n.t('create.import_title')">
      <div class="import-back-row">
        <button type="button" class="import-back-link" :disabled="importing" @click="emit('back')">
          <ArrowLeft :size="16" />
          <span>{{ i18n.t("create.import_back") }}</span>
        </button>
      </div>

      <p class="import-desc">{{ i18n.t("create.import_desc") }}</p>

      <button class="import-dir-button" type="button" :disabled="importing" @click="pickDir">
        <FolderSymlink :size="20" stroke-width="2" />
        <span>{{ i18n.t("create.import_choose") }}</span>
      </button>

      <div v-if="selectedDir" class="import-dir-path">
        <FolderSearch :size="16" />
        <span>{{ selectedDir }}</span>
      </div>

      <!-- 识别中 -->
      <div v-if="scanning" class="import-status">
        <Loader2 :size="18" class="spin" />
        <span>{{ i18n.t("create.import_scanning") }}</span>
      </div>

      <!-- 识别失败 -->
      <div v-else-if="scanError" class="import-status import-status-error">
        <span>{{ scanError }}</span>
      </div>

      <!-- 识别结果 -->
      <template v-else-if="recognized">
        <div class="import-recognized">
          <div class="import-recognized-title">
            {{ i18n.t("create.import_recognized_title") }}
          </div>
          <div class="import-recognized-grid">
            <div class="recognized-item">
              <span class="recognized-label">{{ i18n.t("create.import_impl_label") }}</span>
              <span class="recognized-value">{{ recognizedImpl || "—" }}</span>
            </div>
            <div class="recognized-item">
              <span class="recognized-label">{{ i18n.t("create.import_version_label") }}</span>
              <span class="recognized-value">{{ recognizedVersion || "—" }}</span>
            </div>
            <div class="recognized-item">
              <span class="recognized-label">{{ i18n.t("create.import_java_label") }}</span>
              <span class="recognized-value">{{ recognizedJava || "—" }}</span>
            </div>
          </div>

          <div v-if="launchOptions.length" class="import-field">
            <label>{{ i18n.t("create.import_launch_label") }}</label>
            <cmz-select
              :model-value="selectedLaunchId"
              :options="launchOptions"
              @update:modelValue="selectedLaunchId = $event"
            />
          </div>
          <p v-else class="import-unrecognized">
            {{ i18n.t("create.import_unrecognized") }}
          </p>
        </div>

        <!-- 可编辑字段 -->
        <div class="import-form">
          <div class="import-field">
            <label>{{ i18n.t("create.import_name_label") }}</label>
            <cmz-input v-model="name" :placeholder="i18n.t('create.import_name_label')" />
          </div>
          <div class="import-field-row">
            <div class="import-field">
              <label>{{ i18n.t("create.import_port_label") }}</label>
              <cmz-input v-model.number="port" type="number" />
            </div>
            <div class="import-field">
              <label>{{ i18n.t("create.import_max_memory_label") }}</label>
              <cmz-input v-model.number="maxMemory" type="number" />
            </div>
            <div class="import-field">
              <label>{{ i18n.t("create.import_min_memory_label") }}</label>
              <cmz-input v-model.number="minMemory" type="number" />
            </div>
          </div>
          <div class="import-field">
            <label>{{ i18n.t("create.import_java_exec_label") }}</label>
            <cmz-input v-model="javaExecutable" placeholder="java" />
          </div>
          <div class="import-field">
            <label>{{ i18n.t("create.import_jvm_label") }}</label>
            <cmz-input v-model="jvmArgumentsText" :placeholder="i18n.t('create.import_jvm_hint')" />
          </div>
        </div>

        <div class="import-actions">
          <cmz-button variant="outline" :disabled="importing" @click="emit('back')">
            <ArrowLeft :size="16" />
            <span>{{ i18n.t("create.import_back") }}</span>
          </cmz-button>
          <cmz-button size="lg" :loading="importing" :disabled="!canSubmit" @click="submit">
            {{ i18n.t("create.import_confirm") }}
          </cmz-button>
        </div>
      </template>
    </cmz-card>
  </div>
</template>

<style scoped>
.import-existing-view {
  display: flex;
  justify-content: center;
  padding: 8px 0 24px;
}
.import-existing-card {
  width: 100%;
  max-width: 720px;
}
.import-back-row {
  margin-bottom: 12px;
}
.import-back-link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border: none;
  background: transparent;
  color: var(--primary, #4f8cff);
  cursor: pointer;
  font-size: 13px;
  border-radius: 6px;
}
.import-back-link:hover:not(:disabled) {
  background: var(--surface-2, rgba(128, 128, 128, 0.08));
}
.import-back-link:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.import-desc {
  margin: 0 0 16px;
  font-size: 13px;
  opacity: 0.7;
  line-height: 1.6;
}
.import-dir-button {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border-radius: 10px;
  border: 1px solid var(--border, rgba(128, 128, 128, 0.3));
  background: var(--surface, #1e1e22);
  color: inherit;
  cursor: pointer;
  font-size: 14px;
}
.import-dir-button:hover {
  border-color: var(--primary, #4f8cff);
}
.import-dir-path {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 10px;
  font-size: 12px;
  opacity: 0.6;
  word-break: break-all;
}
.import-status {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 16px;
  font-size: 13px;
  opacity: 0.8;
}
.import-status-error {
  color: #ff6b6b;
  opacity: 1;
}
.spin {
  animation: import-spin 1s linear infinite;
}
@keyframes import-spin {
  to {
    transform: rotate(360deg);
  }
}
.import-recognized {
  margin-top: 18px;
  padding: 14px;
  border-radius: 10px;
  border: 1px solid var(--border, rgba(128, 128, 128, 0.25));
  background: var(--surface-2, rgba(128, 128, 128, 0.06));
}
.import-recognized-title {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 10px;
  opacity: 0.85;
}
.import-recognized-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}
.recognized-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.recognized-label {
  font-size: 11px;
  opacity: 0.55;
}
.recognized-value {
  font-size: 14px;
  font-weight: 500;
  word-break: break-all;
}
.import-unrecognized {
  margin-top: 12px;
  font-size: 12px;
  color: #ffb454;
}
.import-form {
  margin-top: 18px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.import-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.import-field label {
  font-size: 12px;
  opacity: 0.7;
}
.import-field-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}
.import-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 22px;
}
</style>
