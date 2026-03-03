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
}>();

const inputDisabled = computed(() => props.disabled);

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
      {{ i18n.t("create.path_required_hint") }}
    </p>

    <div class="run-path-row">
      <span class="run-path-label">{{ i18n.t("create.path_label") }}</span>
      <SLInput
        class="run-path-input"
        :model-value="runPath"
        :disabled="inputDisabled"
        :placeholder="i18n.t('create.path_custom_placeholder')"
        @update:model-value="emit('update:runPath', $event)"
      >
        <template #suffix>
          <button
            type="button"
            class="sl-input-action"
            :disabled="inputDisabled"
            @click="emit('pickPath')"
          >
            {{ i18n.t("create.browse") }}
          </button>
        </template>
      </SLInput>
    </div>

    <p v-if="showOverwriteWarning" class="run-path-overwrite-warning">
      {{ i18n.t("create.path_overwrite_warning") }}
    </p>

    <p v-if="sourceType === 'folder'" class="run-path-effective">
      {{ i18n.t("create.path_effective_label") }} {{ getPathName(effectivePath) || "-" }}
    </p>
  </div>
</template>

<style src="@styles/components/views/create/RunPathStep.css" scoped></style>
