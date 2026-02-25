<script setup lang="ts">
import { computed } from "vue";
import { SLTabBar } from "@components/common";
import { i18n } from "@language";

type PlayerTab = "online" | "whitelist" | "banned" | "ops";

const props = defineProps<{
  modelValue: PlayerTab;
  onlineCount: number;
  whitelistCount: number;
  bannedCount: number;
  opsCount: number;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: PlayerTab): void;
}>();

const tabs = computed(() => [
  { key: "online", label: i18n.t("players.online_players"), count: props.onlineCount },
  { key: "whitelist", label: i18n.t("players.whitelist"), count: props.whitelistCount },
  { key: "banned", label: i18n.t("players.banned"), count: props.bannedCount },
  { key: "ops", label: i18n.t("players.ops"), count: props.opsCount },
]);
</script>

<template>
  <SLTabBar
    :modelValue="modelValue"
    :tabs="tabs"
    :level="1"
    @update:modelValue="emit('update:modelValue', $event)"
  />
</template>
