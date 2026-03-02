<script setup lang="ts">
import { AlertTriangle } from "lucide-vue-next";
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import SLModal from "@components/common/SLModal.vue";
import { i18n } from "@language";

defineProps<{
  visible: boolean;
  title: string;
  showBanReason?: boolean;
  loading?: boolean;
  serverRunning?: boolean;
  playerName?: string;
  banReason?: string;
}>();

const emit = defineEmits<{
  (e: "update:visible", value: boolean): void;
  (e: "update:playerName", value: string): void;
  (e: "update:banReason", value: string): void;
  (e: "confirm"): void;
}>();
</script>

<template>
  <SLModal :visible="visible" :title="title" @close="emit('update:visible', false)">
    <div class="player-modal-form">
      <SLInput
        :label="i18n.t('players.player_name')"
        :placeholder="i18n.t('players.player_id')"
        :modelValue="playerName"
        @update:modelValue="emit('update:playerName', $event)"
      />
      <SLInput
        v-if="showBanReason"
        :label="i18n.t('players.ban_reason')"
        :placeholder="i18n.t('players.ban_reason_placeholder')"
        :modelValue="banReason"
        @update:modelValue="emit('update:banReason', $event)"
      />
      <p v-if="!serverRunning" class="text-error" style="font-size: 0.8125rem">
        <AlertTriangle
          :size="14"
          style="display: inline; vertical-align: middle; margin-right: 4px"
        />{{ i18n.t("players.server_not_running_hint") }}
      </p>
    </div>
    <template #footer>
      <SLButton variant="secondary" @click="emit('update:visible', false)">{{
        i18n.t("players.cancel")
      }}</SLButton>
      <SLButton
        variant="primary"
        :loading="loading"
        :disabled="!serverRunning"
        @click="emit('confirm')"
        >{{ i18n.t("players.confirm") }}</SLButton
      >
    </template>
  </SLModal>
</template>

<style scoped>
.player-modal-form {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}
</style>
