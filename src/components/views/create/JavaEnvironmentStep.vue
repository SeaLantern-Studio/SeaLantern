<script setup lang="ts">
import { computed } from "vue";
import { RefreshCw } from "lucide-vue-next";
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

const javaOptions = computed(() => {
  return props.javaList.map((java) => {
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
      value: java.path,
    };
  });
});

async function pickJavaFile() {
  const selected = await systemApi.pickJavaFile();
  if (selected) {
    emit("update:selectedJava", selected);
  }
}
</script>

<template>
  <div class="java-step">
    <div class="java-step-row">
      <span class="java-step-label">{{ i18n.t("create.java_env") }}</span>
      <div class="java-step-control">
        <div class="java-step-toolbar">
          <button type="button" class="java-step-rescan" @click="$emit('detect')">
            <RefreshCw :size="14" />
            {{ i18n.t("create.rescan") }}
          </button>
        </div>

        <div v-if="loading" class="java-step-loading">{{ i18n.t("create.scanning") }}</div>
        <div v-else-if="javaList.length === 0" class="java-step-empty">
          <span>{{ i18n.t("create.no_java") }}</span>
          <SLButton variant="primary" size="sm" @click="$emit('detect')">
            {{ i18n.t("create.scan") }}
          </SLButton>
        </div>
        <SLSelect
          v-else
          :model-value="selectedJava"
          :options="javaOptions"
          :placeholder="i18n.t('create.select_java')"
          searchable
          maxHeight="240px"
          @update:model-value="$emit('update:selectedJava', String($event))"
        />
      </div>
    </div>

    <div class="java-step-row java-step-row-manual">
      <span class="java-step-label">{{ i18n.t("create.java_path") }}</span>
      <div class="java-step-control">
        <SLInput
          :model-value="selectedJava"
          :placeholder="i18n.t('create.java_manual')"
          @update:model-value="$emit('update:selectedJava', $event)"
        >
          <template #suffix>
            <button class="java-step-pick" @click="pickJavaFile">
              {{ i18n.t("create.browse") }}
            </button>
          </template>
        </SLInput>
      </div>
    </div>
  </div>
</template>

<style src="@styles/components/views/create/JavaEnvironmentStep.css" scoped></style>
