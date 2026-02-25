<script setup lang="ts">
import { computed } from "vue";
import SLInput from "@components/common/SLInput.vue";
import type { SourceType } from "@components/views/create/SourceIntakeField.vue";
import { getPathName, normalizePathForCompare } from "@components/views/create/startupUtils";
import { i18n } from "@language";

const props = withDefaults(
  defineProps<{
    sourceType: SourceType;
    sourcePath: string;
    runPath: string;
    useSoftwareDataDir: boolean;
    showOverwriteWarning?: boolean;
    disabled?: boolean;
  }>(),
  {
    showOverwriteWarning: false,
    disabled: false,
  },
);

const emit = defineEmits<{
  (e: "update:runPath", value: string): void;
  (e: "pickPath"): void;
  (e: "toggleUseSoftwareDataDir"): void;
}>();

const softwareDataDirText = "已使用软件数据目录";

const inputValue = computed(() => {
  if (props.useSoftwareDataDir) {
    return softwareDataDirText;
  }
  return props.runPath;
});

const inputDisabled = computed(() => props.disabled || props.useSoftwareDataDir);

const effectivePath = computed(() => {
  if (props.sourceType !== "folder") {
    return props.runPath;
  }

  const source = props.sourcePath.trim();
  const target = props.runPath.trim();
  if (!target || normalizePathForCompare(source) === normalizePathForCompare(target)) {
    return source;
  }

  return target;
});
</script>

<template>
  <div class="run-path-step">
    <p class="run-path-hint">
      {{
        sourceType === "archive"
          ? i18n.t("create.path_required_archive")
          : i18n.t("create.path_optional_folder")
      }}
    </p>

    <div class="run-path-row">
      <span class="run-path-label">{{ i18n.t("create.path_label") }}</span>
      <SLInput
        class="run-path-input"
        :model-value="inputValue"
        :disabled="inputDisabled"
        :placeholder="
          sourceType === 'archive'
            ? i18n.t('create.path_archive_placeholder')
            : i18n.t('create.path_folder_placeholder')
        "
        @update:model-value="emit('update:runPath', $event)"
      >
        <template #suffix>
          <button type="button" class="run-path-picker" :disabled="inputDisabled" @click="emit('pickPath')">
            {{ i18n.t("create.browse") }}
          </button>
        </template>
      </SLInput>
    </div>

    <div class="run-path-toggle-row">
      <p v-if="showOverwriteWarning" class="run-path-overwrite-warning">
        {{ i18n.t("create.path_overwrite_warning") }}
      </p>
      <button
        type="button"
        class="run-path-data-dir-toggle"
        :class="{ 'is-active': useSoftwareDataDir }"
        :disabled="disabled"
        @click="emit('toggleUseSoftwareDataDir')"
      >
        使用软件数据目录
      </button>
    </div>

    <p v-if="sourceType === 'folder'" class="run-path-effective">
      {{ i18n.t("create.path_effective_label") }} {{ getPathName(effectivePath) || "-" }}
    </p>
  </div>
</template>

<style src="@styles/components/views/create/RunPathStep.css" scoped></style>
