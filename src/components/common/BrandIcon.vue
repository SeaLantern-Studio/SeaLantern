<script setup lang="ts">
import { computed } from 'vue';
import * as icons from 'simple-icons';

interface Props {
  name: string;
  size?: number;
}

const props = withDefaults(defineProps<Props>(), {
  size: 24,
});

const iconNameMap: Record<string, string> = {
  qq: 'siQq',
  wechat: 'siWechat',
  weixin: 'siWechat',
};

const darkIcons = new Set(['github', 'gitee']);

const icon = computed(() => {
  const lowerName = props.name.toLowerCase();
  
  if (iconNameMap[lowerName]) {
    return (icons as any)[iconNameMap[lowerName]];
  }
  
  const iconName = 'si' + props.name.charAt(0).toUpperCase() + props.name.slice(1).toLowerCase();
  return (icons as any)[iconName];
});

const pathData = computed(() => icon.value?.path);
const color = computed(() => {
  const lowerName = props.name.toLowerCase();
  if (darkIcons.has(lowerName)) {
    return 'currentColor';
  }
  return icon.value?.hex ? `#${icon.value.hex}` : 'currentColor';
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
