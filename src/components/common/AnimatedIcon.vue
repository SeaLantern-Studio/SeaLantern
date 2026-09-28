<script setup lang="ts">
/**
 * 动画图标组件：加载 assets/icons 下的自绘 SVG 并内联渲染。
 * SVG 内嵌的 :hover 动画只在"内联进文档"时生效，
 * 用 <img> 引入是非交互模式会失效，所以统一走这个组件。
 */
import { ref, watch } from "vue";

// 懒加载全部动画 SVG 源码，按需请求不打进首屏
const sources = import.meta.glob("/src/assets/icons/*.svg", {
  query: "?raw",
  import: "default",
}) as Record<string, () => Promise<string>>;

interface Props {
  name: string; // 图标名，对应 assets/icons/<name>.svg
  size?: number;
}

const props = withDefaults(defineProps<Props>(), {
  size: 20,
});

const svg = ref<string | null>(null);

// 未知图标名的兜底图形，静态 info 样式，保证插件传错名时不至于空白
const FALLBACK_SVG =
  '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>';

watch(
  () => props.name,
  async (name) => {
    svg.value = null;
    const loader = sources[`/src/assets/icons/${name}.svg`];
    if (!loader) {
      svg.value = FALLBACK_SVG;
      return;
    }
    try {
      const loaded = await loader();
      // 异步等待期间 name 可能又变了，过期结果直接丢弃，防止显示串图
      if (props.name === name) {
        svg.value = loaded;
      }
    } catch {
      if (props.name === name) {
        svg.value = FALLBACK_SVG;
      }
    }
  },
  { immediate: true },
);
</script>

<template>
  <span class="anim-icon" :style="{ width: `${size}px`, height: `${size}px` }" v-html="svg"></span>
</template>

<style scoped>
/* 内联 SVG 撑满容器，描边色跟随父级 currentColor 自动适配主题 */
.anim-icon {
  display: inline-flex;
  flex-shrink: 0;
}

.anim-icon :deep(svg) {
  width: 100%;
  height: 100%;
  display: block;
}
</style>
