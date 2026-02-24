<script setup lang="ts">
import { computed } from "vue";
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import type { SourceType } from "@components/views/create/SourceIntakeField.vue";
import { getPathName, normalizePathForCompare } from "@components/views/create/startupUtils";
import { i18n } from "@language";

const props = withDefaults(
  defineProps<{
    sourceType: SourceType;
    sourcePath: string;
    runPath: string;
    disabled?: boolean;
  }>(),
  {
    disabled: false,
  },
);

const emit = defineEmits<{
  (e: "update:runPath", value: string): void;
  (e: "pickPath"): void;
}>();

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

    <SLInput
      :label="i18n.t('create.path_label')"
      :model-value="runPath"
      :disabled="disabled"
      :placeholder="
        sourceType === 'archive'
          ? i18n.t('create.path_archive_placeholder')
          : i18n.t('create.path_folder_placeholder')
      "
      @update:model-value="emit('update:runPath', $event)"
    >
      <template #suffix>
        <button type="button" class="run-path-picker" :disabled="disabled" @click="emit('pickPath')">
          {{ i18n.t("create.browse") }}
        </button>
      </template>
    </SLInput>

    <div v-if="sourceType === 'folder'" class="run-path-actions">
      <SLButton variant="secondary" size="sm" :disabled="disabled" @click="emit('update:runPath', '')">
        {{ i18n.t("create.path_use_in_place") }}
      </SLButton>
      <SLButton
        variant="secondary"
        size="sm"
        :disabled="disabled"
        @click="emit('update:runPath', sourcePath)"
      >
        {{ i18n.t("create.path_use_source") }}
      </SLButton>
    </div>

    <p v-if="sourceType === 'folder'" class="run-path-effective">
      {{ i18n.t("create.path_effective_label") }} {{ getPathName(effectivePath) || "-" }}
    </p>
  </div>
</template>

<style src="@styles/components/views/create/RunPathStep.css" scoped></style>
