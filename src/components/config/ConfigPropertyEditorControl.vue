<script setup lang="ts">
import SLSwitch from "@components/common/SLSwitch.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLInput from "@components/common/SLInput.vue";

interface Option {
  label: string;
  value: string | number;
}

interface Props {
  propertyKey: string;
  modelValue: string;
  valueType?: string;
  defaultValue?: string;
  numericError?: string;
  gamemodeOptions: Option[];
  difficultyOptions: Option[];
}

const props = withDefaults(defineProps<Props>(), {
  valueType: "string",
  defaultValue: "",
  numericError: "",
});

const emit = defineEmits<{
  "update:modelValue": [value: string | boolean | number];
}>();

function isBooleanControl(valueType: string | undefined, value: string | undefined) {
  return valueType === "boolean" || value === "true" || value === "false";
}
</script>

<template>
  <div class="config-property-editor-control">
    <template v-if="isBooleanControl(valueType, modelValue)">
      <SLSwitch
        :modelValue="modelValue === 'true'"
        @update:modelValue="emit('update:modelValue', $event)"
      />
    </template>
    <template v-else-if="propertyKey === 'gamemode'">
      <SLSelect
        :modelValue="modelValue"
        :options="gamemodeOptions"
        class="config-property-control-input"
        @update:modelValue="emit('update:modelValue', $event)"
      />
    </template>
    <template v-else-if="propertyKey === 'difficulty'">
      <SLSelect
        :modelValue="modelValue"
        :options="difficultyOptions"
        class="config-property-control-input"
        @update:modelValue="emit('update:modelValue', $event)"
      />
    </template>
    <template v-else>
      <SLInput
        :modelValue="modelValue"
        :placeholder="defaultValue"
        :type="valueType === 'number' ? 'number' : 'text'"
        :step="valueType === 'number' ? 1 : undefined"
        class="config-property-control-input"
        @update:modelValue="emit('update:modelValue', $event)"
      />
      <p v-if="numericError" class="entry-desc text-caption">
        {{ numericError }}
      </p>
    </template>
  </div>
</template>

<style scoped>
.config-property-editor-control {
  width: var(--sl-config-control-width);
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--sl-space-xs);
}

.config-property-control-input {
  width: 100%;
}
</style>
