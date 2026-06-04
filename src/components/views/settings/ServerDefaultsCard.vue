<script setup lang="ts">
import { computed } from "vue";
import { RefreshCw } from "lucide-vue-next";
import SLCard from "@components/common/SLCard.vue";
import SLInput from "@components/common/SLInput.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLTextarea from "@components/common/SLTextarea.vue";
import CpuPolicyEditor from "@components/startup/CpuPolicyEditor.vue";
import JavaDownloader from "@components/JavaDownloader.vue";
import type { JavaInfo } from "@api/java";
import { i18n } from "@language";
import type { CpuPolicyConfig, JvmPresetConfig, JvmPresetId } from "@type/server";
import { MVP_JVM_PRESET_IDS } from "@utils/serverStartupConfig";

const props = defineProps<{
  maxMemory: string;
  minMemory: string;
  port: string;
  defaultJavaPath: string;
  defaultJvmArgsText: string;
  defaultJvmPreset: JvmPresetConfig;
  defaultCpuPolicy: CpuPolicyConfig;
  defaultRunPath: string;
  javaList: JavaInfo[];
  javaLoading: boolean;
}>();

const emit = defineEmits<{
  (e: "update:maxMemory", value: string): void;
  (e: "update:minMemory", value: string): void;
  (e: "update:port", value: string): void;
  (e: "update:defaultJavaPath", value: string): void;
  (e: "update:defaultJvmArgsText", value: string): void;
  (e: "update:defaultJvmPreset", value: JvmPresetConfig): void;
  (e: "update:defaultCpuPolicy", value: CpuPolicyConfig): void;
  (e: "update:defaultRunPath", value: string): void;
  (e: "change"): void;
  (e: "detectJava"): void;
  (e: "javaInstalled", path: string): void;
  (e: "browseJavaPath"): void;
  (e: "browseRunPath"): void;
}>();

function getJavaLabel(java: JavaInfo): { label: string; subLabel: string } {
  const version = java.major_version;
  const arch = java.is_64bit ? i18n.t("common.java_64bit") : i18n.t("common.java_32bit");

  let vendor = java.vendor;
  if (vendor.includes("Oracle") || vendor.includes("Sun")) {
    vendor = "Oracle";
  } else if (vendor.includes("Temurin") || vendor.includes("Adopt")) {
    vendor = "Eclipse Temurin";
  } else if (vendor.includes("Amazon")) {
    vendor = "Amazon Corretto";
  } else if (vendor.includes("Microsoft")) {
    vendor = "Microsoft";
  } else if (vendor.includes("Zulu") || vendor.includes("Azul")) {
    vendor = "Azul Zulu";
  } else if (vendor.includes("Liberica") || vendor.includes("BellSoft")) {
    vendor = "Liberica";
  }

  return {
    label: `Java ${version} ${vendor} ${arch}`,
    subLabel: java.path,
  };
}

const javaOptions = computed(() => {
  return props.javaList.map((java) => {
    const labelInfo = getJavaLabel(java);
    return {
      label: labelInfo.label,
      subLabel: labelInfo.subLabel,
      value: java.path,
    };
  });
});

const jvmPresetOptions = computed(() =>
  MVP_JVM_PRESET_IDS.map((preset) => ({
    value: preset,
    label: i18n.t(`common.jvm_preset_${preset}_label`),
    subLabel: i18n.t(`common.jvm_preset_${preset}_desc`),
  })),
);
</script>

<template>
  <SLCard
    :title="i18n.t('settings.server_defaults')"
    :subtitle="i18n.t('settings.server_defaults_desc')"
  >
    <div class="sl-settings-group">
      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.default_memory") }} (MB)</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.max_memory_desc") }}</span>
        </div>
        <div class="sl-input-sm">
          <SLInput
            :model-value="maxMemory"
            type="number"
            @update:model-value="
              (v) => {
                emit('update:maxMemory', v);
                emit('change');
              }
            "
          />
        </div>
      </div>

      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.min_memory") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.min_memory_desc") }}</span>
        </div>
        <div class="sl-input-sm">
          <SLInput
            :model-value="minMemory"
            type="number"
            @update:model-value="
              (v) => {
                emit('update:minMemory', v);
                emit('change');
              }
            "
          />
        </div>
      </div>

      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.default_port") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.port_desc") }}</span>
        </div>
        <div class="sl-input-sm">
          <SLInput
            :model-value="port"
            type="number"
            @update:model-value="
              (v) => {
                emit('update:port', v);
                emit('change');
              }
            "
          />
        </div>
      </div>

      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.default_java") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.default_java_desc") }}</span>
        </div>
        <div class="java-setting-panel sl-input-lg">
          <div v-if="javaLoading" class="java-loading">
            <div class="spinner"></div>
            <span>{{ i18n.t("create.scanning") }}</span>
          </div>

          <div v-else-if="javaList.length > 0" class="java-select-container">
            <div class="java-header">
              <div class="java-found">
                {{ i18n.t("create.java_found", { count: javaList.length }) }}
              </div>
              <button class="rescan-btn" type="button" @click="emit('detectJava')">
                <RefreshCw :size="14" />
                {{ i18n.t("create.rescan") }}
              </button>
            </div>

            <SLSelect
              :model-value="defaultJavaPath"
              :options="javaOptions"
              :placeholder="i18n.t('create.select_java')"
              searchable
              maxHeight="240px"
              @update:model-value="
                (value) => {
                  emit('update:defaultJavaPath', String(value));
                  emit('change');
                }
              "
            />
          </div>

          <div v-else class="java-empty">
            <p>{{ i18n.t("create.no_java") }}</p>
            <button class="scan-btn" type="button" @click="emit('detectJava')">
              {{ i18n.t("create.scan") }}
            </button>
          </div>

          <SLInput
            :model-value="defaultJavaPath"
            :placeholder="i18n.t('create.java_manual')"
            @update:model-value="
              (v) => {
                emit('update:defaultJavaPath', v);
                emit('change');
              }
            "
          >
            <template #suffix>
              <button type="button" class="sl-input-action" @click="emit('browseJavaPath')">
                {{ i18n.t("settings.browse") }}
              </button>
            </template>
          </SLInput>
        </div>
      </div>

      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.default_run_path") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.default_run_path_desc") }}</span>
        </div>
        <div class="sl-input-lg">
          <SLInput
            :model-value="defaultRunPath"
            :placeholder="i18n.t('settings.default_run_path_desc')"
            @update:model-value="
              (v) => {
                emit('update:defaultRunPath', v);
                emit('change');
              }
            "
          >
            <template #suffix>
              <button type="button" class="sl-input-action" @click="emit('browseRunPath')">
                {{ i18n.t("settings.browse") }}
              </button>
            </template>
          </SLInput>
        </div>
      </div>

      <div class="sl-setting-row full-width">
        <JavaDownloader
          @installed="
            (path) => {
              emit('javaInstalled', path);
              emit('change');
            }
          "
        />
      </div>

      <div class="sl-setting-row full-width">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.default_jvm_preset_label") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.default_jvm_preset_desc") }}</span>
        </div>
        <SLSelect
          :model-value="defaultJvmPreset.preset"
          :options="jvmPresetOptions"
          :placeholder="i18n.t('settings.default_jvm_preset_placeholder')"
          @update:model-value="
            (value) => {
              emit('update:defaultJvmPreset', { preset: value as JvmPresetId });
              emit('change');
            }
          "
        />
      </div>

      <div class="sl-setting-row full-width startup-notice-row">
        <p class="sl-startup-notice">
          {{ i18n.t("settings.startup_notice_not_main_thread_boost") }}
        </p>
      </div>

      <div class="sl-setting-row full-width">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.default_cpu_policy_label") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.default_cpu_policy_desc") }}</span>
        </div>
        <CpuPolicyEditor
          :model-value="defaultCpuPolicy"
          scope="settings"
          @update:modelValue="
            (value) => {
              emit('update:defaultCpuPolicy', value);
              emit('change');
            }
          "
        />
      </div>

      <div class="sl-setting-row full-width">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.default_jvm_args_label") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.default_jvm_args_desc") }}</span>
        </div>
        <SLTextarea
          :model-value="defaultJvmArgsText"
          :placeholder="i18n.t('settings.default_jvm_args_placeholder')"
          :rows="3"
          @update:model-value="
            (v) => {
              emit('update:defaultJvmArgsText', v);
              emit('change');
            }
          "
        />
      </div>
    </div>
  </SLCard>
</template>

<style scoped>
.java-setting-panel {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}

.java-loading {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  min-height: 40px;
  color: var(--sl-text-tertiary);
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--sl-border);
  border-top-color: var(--sl-primary);
  border-radius: 50%;
  animation: sl-spin 0.8s linear infinite;
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
  gap: var(--sl-space-sm);
}

.java-found {
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-sm);
}

.java-empty {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-sm);
  padding: 10px 12px;
  border: 1px dashed var(--sl-border);
  border-radius: var(--sl-radius-md);
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-sm);
}

.java-empty p {
  margin: 0;
}

.scan-btn,
.rescan-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: none;
  border-radius: var(--sl-radius-sm);
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
  font-size: var(--sl-font-size-sm);
  cursor: pointer;
  transition:
    background-color var(--sl-transition-fast),
    color var(--sl-transition-fast);
}

.scan-btn:hover,
.rescan-btn:hover {
  background: color-mix(in srgb, var(--sl-primary) 16%, var(--sl-primary-bg));
}

.sl-setting-row.full-width {
  flex-direction: column;
  align-items: stretch;
}

.sl-setting-row.full-width :deep(.sl-textarea) {
  margin-top: var(--sl-space-sm);
  font-family: var(--sl-font-mono);
  font-size: 0.8125rem;
  line-height: 1.6;
}

.startup-notice-row {
  padding-top: 0;
}

.sl-startup-notice {
  margin: 0;
  color: var(--sl-warning, #b45309);
  font-size: var(--sl-font-size-sm);
  line-height: 1.5;
}

@keyframes sl-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
