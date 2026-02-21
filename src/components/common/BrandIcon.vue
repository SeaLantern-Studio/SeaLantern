<script setup lang="ts">
import { computed } from "vue";
import type { SimpleIcon } from "simple-icons";
import {
  siGithub,
  siGitee,
  siQq,
  siWechat,
  siSteam,
  siBilibili,
  siX,
  siYoutube,
  siTwitch,
  siDiscord,
  siTelegram,
  siReddit,
  siSpotify,
} from "simple-icons";

interface Props {
  name: string;
  size?: number;
}

const props = withDefaults(defineProps<Props>(), {
  size: 24,
});

const iconMap: Record<string, SimpleIcon> = {
  github: siGithub,
  gitee: siGitee,
  qq: siQq,
  wechat: siWechat,
  weixin: siWechat,
  steam: siSteam,
  bilibili: siBilibili,
  x: siX,
  youtube: siYoutube,
  twitch: siTwitch,
  discord: siDiscord,
  telegram: siTelegram,
  reddit: siReddit,
  spotify: siSpotify,
};

const darkIcons = new Set(["github", "gitee"]);

const icon = computed(() => {
  const lowerName = props.name.toLowerCase();
  return iconMap[lowerName] ?? null;
});

const pathData = computed(() => icon.value?.path);
const color = computed(() => {
  const lowerName = props.name.toLowerCase();
  if (darkIcons.has(lowerName)) {
    return "currentColor";
  }
  return icon.value?.hex ? `#${icon.value.hex}` : "currentColor";
});
</script>

<template>
  <svg
    v-if="pathData"
    viewBox="0 0 24 24"
    :width="size"
    :height="size"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <path :d="pathData" :fill="color" />
  </svg>
</template>
