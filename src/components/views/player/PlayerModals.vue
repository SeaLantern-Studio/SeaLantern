<script setup lang="ts">
import { AlertTriangle } from "lucide-vue-next";
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
  <cmz-modal :visible="visible" :title="title" @close="emit('update:visible', false)">
    <div class="player-modal-form">
      <cmz-input
        :label="i18n.t('players.player_name')"
        :placeholder="i18n.t('players.player_id')"
        :modelValue="playerName"
        @update:modelValue="emit('update:playerName', $event)"
      />
      <cmz-input
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
      <cmz-button variant="outline" @click="emit('update:visible', false)">{{
        i18n.t("players.cancel")
      }}</cmz-button>
      <cmz-button
        :loading="loading"
        :disabled="!serverRunning"
        @click="emit('confirm')"
        >{{ i18n.t("players.confirm") }}</cmz-button>
    </template>
  </cmz-modal>
</template>

<style scoped>
.player-modal-form {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}
</style>
