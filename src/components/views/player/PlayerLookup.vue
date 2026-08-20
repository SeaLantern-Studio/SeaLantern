<script setup lang="ts">
import { ref } from "vue";
import { Search, Copy, Check } from "lucide-vue-next";
import { playerApi, type PlayerProfile } from "@api/player";
import { i18n } from "@language";
import { useToast } from "cmzya-modern-ui";
import { useLoading } from "@composables/useAsync";
import PlayerAvatar from "@components/views/player/PlayerAvatar.vue";

const username = ref("");
const result = ref<PlayerProfile | null>(null);
const copied = ref(false);
const { loading, withLoading } = useLoading();
const toast = useToast();

async function handleLookup() {
  const name = username.value.trim();
  if (!name) {
    toast.error(i18n.t("players.lookup_invalid_input"));
    return;
  }

  result.value = null;
  copied.value = false;

  await withLoading(async () => {
    try {
      result.value = await playerApi.lookupPlayer(name);
    } catch (e: unknown) {
      const err = String(e);
      if (err === "not_found") {
        toast.error(i18n.t("players.lookup_not_found"));
      } else if (err === "rate_limited") {
        toast.error(i18n.t("players.lookup_rate_limited"));
      } else if (err === "invalid_input") {
        toast.error(i18n.t("players.lookup_invalid_input"));
      } else {
        toast.error(i18n.t("players.lookup_service_unavailable"));
      }
    }
  });
}

async function copyUuid() {
  if (!result.value) return;
  try {
    await navigator.clipboard.writeText(result.value.uuid);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 2000);
  } catch {
    toast.error(i18n.t("players.lookup_copy_failed"));
  }
}
</script>

<template>
  <div class="player-lookup">
    <div class="lookup-header">
      <span class="lookup-title">{{ i18n.t("players.lookup_title") }}</span>
    </div>
    <div class="lookup-form">
      <cmz-input
        :placeholder="i18n.t('players.lookup_placeholder')"
        v-model="username"
        @keyup.enter="handleLookup"
      />
      <cmz-button :loading="loading" @click="handleLookup">
        <Search :size="16" />
        {{ i18n.t("players.lookup_button") }}
      </cmz-button>
    </div>
    <div v-if="result" class="lookup-result">
      <PlayerAvatar :name="result.name" :size="48" />
      <div class="result-info">
        <div class="result-row">
          <span class="result-label">{{ i18n.t("players.lookup_name_label") }}</span>
          <span class="result-value">{{ result.name }}</span>
        </div>
        <div class="result-row">
          <span class="result-label">UUID</span>
          <span class="result-value result-uuid">{{ result.uuid }}</span>
          <button class="copy-btn" :title="i18n.t('players.lookup_copy_uuid')" @click="copyUuid">
            <Check v-if="copied" :size="14" />
            <Copy v-else :size="14" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.player-lookup {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-md);
  border-radius: var(--sl-radius-md);
  background: var(--sl-bg-secondary);
  border: 1px solid var(--sl-border-light);
}

.lookup-header {
  display: flex;
  align-items: center;
}

.lookup-title {
  font-size: var(--sl-text-sm);
  font-weight: 600;
  color: var(--sl-text-secondary);
}

.lookup-form {
  display: flex;
  gap: var(--sl-space-sm);
  align-items: flex-end;
}

.lookup-form > :first-child {
  flex: 1;
}

.lookup-result {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: var(--sl-space-md);
  padding: var(--sl-space-sm) var(--sl-space-md);
  border-radius: var(--sl-radius-sm);
  background: var(--sl-bg-tertiary);
}

.result-info {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
}

.result-row {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}

.result-label {
  font-size: var(--sl-text-xs);
  color: var(--sl-text-tertiary);
  min-width: 60px;
}

.result-value {
  font-size: var(--sl-text-sm);
  color: var(--sl-text-primary);
  font-family: var(--sl-font-mono, monospace);
}

.result-uuid {
  letter-spacing: 0.5px;
}

.copy-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  border: none;
  border-radius: var(--sl-radius-sm);
  background: transparent;
  color: var(--sl-text-tertiary);
  cursor: pointer;
  transition: all 0.15s ease;
}

.copy-btn:hover {
  background: var(--sl-bg-hover);
  color: var(--sl-text-primary);
}
</style>
