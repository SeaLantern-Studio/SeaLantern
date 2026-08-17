<script setup lang="ts">
import { ref } from "vue";
import { i18n } from "@language";
import MotdEditorBody from "./MotdEditorBody.vue";

interface Props {
  /** 当前 server.properties 中的 motd 草稿值（换行为字面 \n 的存储格式） */
  modelValue: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  close: [];
  apply: [value: string];
}>();

const bodyRef = ref<InstanceType<typeof MotdEditorBody> | null>(null);
</script>

<template>
  <cmz-modal
    :visible="true"
    :title="i18n.t('config.motd.title')"
    width="1000px"
    @close="emit('close')"
  >
    <MotdEditorBody ref="bodyRef" :modelValue="modelValue" @apply="emit('apply', $event)" />

    <template #footer>
      <cmz-button variant="outline" @click="emit('close')">
        {{ i18n.t("common.cancel") }}
      </cmz-button>
      <cmz-button @click="bodyRef?.requestApply()">
        {{ i18n.t("config.motd.apply") }}
      </cmz-button>
    </template>
  </cmz-modal>
</template>
