<script setup lang="ts">
import { computed } from "vue";
import { RefreshCw } from "lucide-vue-next";
import SLCard from "@components/common/SLCard.vue";
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import SLSelect from "@components/common/SLSelect.vue";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import type { JavaInfo } from "@api/java";

const props = defineProps<{
  javaList: JavaInfo[];
  selectedJava: string;
  loading: boolean;
}>();

const emit = defineEmits<{
  (e: "update:selectedJava", value: string): void;
  (e: "detect"): void;
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

async function pickJavaFile() {
  try {
    const result = await systemApi.pickJavaFile();
    if (result) {
      emit("update:selectedJava", result);
    }
  } catch (e) {
    console.error("Pick file error:", e);
  }
}
</script>

<template>
  <SLCard :title="i18n.t('create.java_env')" :subtitle="i18n.t('create.java_scan')">
    <div v-if="loading" class="java-loading">
      <div class="spinner"></div>
      <span>{{ i18n.t("create.scanning") }}</span>
    </div>
    <div v-else-if="javaList.length === 0" class="java-empty">
      <p class="text-body">{{ i18n.t("create.no_java") }}</p>
      <SLButton variant="primary" @click="$emit('detect')" style="margin-top: 12px">
        {{ i18n.t("create.scan") }}
      </SLButton>
    </div>
    <div v-else class="java-select-container">
      <div class="java-header">
        <div class="java-found text-caption">
          {{ i18n.t("create.java_found", { count: javaList.length }) }}
        </div>
        <button class="rescan-btn" @click="$emit('detect')" :disabled="loading">
          <RefreshCw :size="14" />
          {{ i18n.t("create.rescan") }}
        </button>
      </div>
      <SLSelect
        :model-value="selectedJava"
        @update:model-value="$emit('update:selectedJava', $event)"
        :options="javaOptions"
        :placeholder="i18n.t('create.select_java')"
        searchable
        maxHeight="240px"
      />
    </div>
    <div class="java-manual">
      <SLInput
        :label="i18n.t('create.java_path')"
        :model-value="selectedJava"
        @update:model-value="$emit('update:selectedJava', $event)"
        :placeholder="i18n.t('create.java_manual')"
      >
        <template #suffix>
          <button class="sl-input-action" @click="pickJavaFile">
            {{ i18n.t("create.browse") }}
          </button>
        </template>
      </SLInput>
    </div>
  </SLCard>
</template>

<style scoped>
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
  font-size: var(--sl-font-size-sm);
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
</style>
