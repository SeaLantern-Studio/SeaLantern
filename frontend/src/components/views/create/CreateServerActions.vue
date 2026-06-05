<script setup lang="ts">
import { useRouter } from "vue-router";
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";

withDefaults(
  defineProps<{
    creating: boolean;
    createDisabled?: boolean;
    importDisabled?: boolean;
  }>(),
  {
    createDisabled: false,
    importDisabled: false,
  },
);

const emit = defineEmits<{
  (e: "create"): void;
  (e: "import"): void;
}>();

const router = useRouter();
</script>

<template>
  <div class="create-actions">
    <SLButton variant="secondary" size="lg" @click="router.push('/')">{{
      i18n.t("create.cancel")
    }}</SLButton>
    <SLButton
      variant="primary"
      size="lg"
      :loading="creating"
      :disabled="createDisabled"
      @click="$emit('create')"
    >
      {{ i18n.t("create.select_and_create") }}
    </SLButton>
    <SLButton
      variant="primary"
      size="lg"
      :loading="creating"
      :disabled="importDisabled"
      @click="$emit('import')"
    >
      {{ i18n.t("create.import_existing") }}
    </SLButton>
  </div>
</template>

<style scoped>
.create-actions {
  display: flex;
  justify-content: center;
  gap: var(--sl-space-md);
}
.create-actions :deep(.sl-button) {
  min-width: 120px;
}
</style>
