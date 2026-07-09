<script setup lang="ts">
import { computed } from "vue";
import SLInput from "@components/common/SLInput.vue";
import SLSelect from "@components/common/SLSelect.vue";
import { i18n } from "@language";
import type { CpuPolicyConfig, CpuPolicyMode } from "@type/server";
import { getCpuPolicyValidationError, normalizeCpuPolicy } from "@utils/serverStartupConfig";

const props = withDefaults(
  defineProps<{
    modelValue: CpuPolicyConfig;
    scope: "config" | "create" | "settings";
    disabled?: boolean;
  }>(),
  {
    disabled: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: CpuPolicyConfig];
}>();

const policy = computed(() => normalizeCpuPolicy(props.modelValue));

const modeOptions = computed(() => [
  {
    value: "off",
    label: i18n.t(`${props.scope}.cpu_policy_mode_off`),
  },
  {
    value: "count",
    label: i18n.t(`${props.scope}.cpu_policy_mode_count`),
  },
  {
    value: "explicit",
    label: i18n.t(`${props.scope}.cpu_policy_mode_explicit`),
  },
]);

const validationMessage = computed(() => {
  const errorKey = getCpuPolicyValidationError(policy.value);
  return errorKey ? i18n.t(`${props.scope}.cpu_policy_invalid_${errorKey}`) : "";
});

function updatePolicy(next: Partial<CpuPolicyConfig>) {
  emit(
    "update:modelValue",
    normalizeCpuPolicy({
      ...policy.value,
      ...next,
    }),
  );
}

function handleModeChange(value: string | number) {
  const nextMode = value as CpuPolicyMode;

  if (nextMode === "off") {
    updatePolicy({
      mode: "off",
      count: null,
      explicit_set: null,
    });
    return;
  }

  if (nextMode === "count") {
    updatePolicy({
      mode: "count",
      count: policy.value.count ?? null,
      explicit_set: null,
    });
    return;
  }

  updatePolicy({
    mode: "explicit",
    count: null,
    explicit_set: policy.value.explicit_set ?? "",
  });
}

function handleCountChange(value: string) {
  const trimmed = value.trim();
  updatePolicy({
    count: trimmed === "" ? null : Number.parseInt(trimmed, 10),
  });
}

function handleExplicitChange(value: string) {
  updatePolicy({
    explicit_set: value,
  });
}
</script>

<template>
  <div class="cpu-policy-editor">
    <SLSelect
      :model-value="policy.mode"
      :options="modeOptions"
      :placeholder="i18n.t(`${scope}.cpu_policy_mode_placeholder`)"
      :disabled="disabled"
      @update:modelValue="handleModeChange"
    />

    <div v-if="policy.mode === 'count'" class="cpu-policy-field">
      <SLInput
        type="number"
        :model-value="policy.count == null ? '' : String(policy.count)"
        :placeholder="i18n.t(`${scope}.cpu_policy_count_placeholder`)"
        :disabled="disabled"
        :min="1"
        :step="1"
        @update:modelValue="handleCountChange"
      />
      <p class="cpu-policy-hint">{{ i18n.t(`${scope}.cpu_policy_count_desc`) }}</p>
    </div>

    <div v-else-if="policy.mode === 'explicit'" class="cpu-policy-field">
      <SLInput
        :model-value="policy.explicit_set ?? ''"
        :placeholder="i18n.t(`${scope}.cpu_policy_explicit_placeholder`)"
        :disabled="disabled"
        @update:modelValue="handleExplicitChange"
      />
      <p class="cpu-policy-hint">{{ i18n.t(`${scope}.cpu_policy_explicit_desc`) }}</p>
    </div>

    <p v-if="validationMessage" class="cpu-policy-error">{{ validationMessage }}</p>
  </div>
</template>

<style scoped>
.cpu-policy-editor {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}

.cpu-policy-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.cpu-policy-hint,
.cpu-policy-error {
  margin: 0;
  font-size: 0.8125rem;
  line-height: 1.5;
}

.cpu-policy-hint {
  color: var(--sl-text-tertiary);
}

.cpu-policy-error {
  color: var(--sl-error);
}
</style>
