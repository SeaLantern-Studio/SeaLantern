<script setup lang="ts">
import { ref, computed } from "vue";
import { useRegisterComponent } from "../../composables/useRegisterComponent";

interface Props {
  value: number;
  max?: number;
  label?: string;
  showPercent?: boolean;
  variant?: "primary" | "success" | "warning" | "error";
  componentId?: string;
}

const props = withDefaults(defineProps<Props>(), {
  max: 100,
  showPercent: true,
  variant: "primary",
});

const elRef = ref<HTMLElement | null>(null);
const id = props.componentId ?? `sl-progress-${Math.random().toString(36).slice(2, 8)}`;
useRegisterComponent(id, {
  type: "SLProgress",
  get: (prop) => (prop === "value" ? props.value : undefined),
  set: () => {},
  call: () => undefined,
  on: () => () => {},
  el: () => elRef.value,
});

// 计算属性 - 缓存计算结果，避免重复计算
const percentage = computed(() => Math.min((props.value / props.max) * 100, 100));

// 计算属性 - 四舍五入后的百分比值
const roundedPercent = computed(() => Math.round(percentage.value));

// 计算属性 - 进度条宽度
const barWidth = computed(() => `${percentage.value}%`);

// 计算属性 - 是否有头部内容（标签或百分比）
const hasHeader = computed(() => props.label || props.showPercent);

// 计算属性 - 颜色类名
const barClass = computed(() => `sl-progress-bar--${props.variant}`);
</script>

<template>
  <div ref="elRef" class="sl-progress">
    <!-- 头部：标签和百分比 -->
    <div v-if="hasHeader" class="sl-progress-header">
      <span v-if="label" class="sl-progress-label">{{ label }}</span>
      <span v-if="showPercent" class="sl-progress-percent"> {{ roundedPercent }}% </span>
    </div>

    <!-- 进度条轨道 -->
    <div class="sl-progress-track">
      <!-- 进度条本身 -->
      <div
        class="sl-progress-bar"
        :class="barClass"
        :style="{ width: barWidth }"
        role="progressbar"
        :aria-valuenow="value"
        :aria-valuemin="0"
        :aria-valuemax="max"
        :aria-label="label || '进度'"
      />
    </div>
  </div>
</template>

<style scoped>
/* 根容器 */
.sl-progress {
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 100%; /* 确保宽度适应容器 */
  min-width: 0; /* 防止内容溢出 */
}

/* 头部容器 - 标签和百分比 */
.sl-progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  min-height: 1.25rem; /* 保持高度一致 */
}

/* 标签样式 */
.sl-progress-label {
  font-size: 0.8125rem;
  color: var(--sl-text-secondary, #666);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex-shrink: 1; /* 允许缩小 */
}

/* 百分比样式 */
.sl-progress-percent {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-text-primary, #333);
  font-family: var(--sl-font-mono, "Menlo", "Monaco", "Consolas", monospace);
  flex-shrink: 0; /* 防止百分比被挤压 */
  margin-left: 0.5rem; /* 与标签保持间距 */
}

/* 进度条轨道 */
.sl-progress-track {
  height: 6px;
  background: var(--sl-bg-tertiary, #e5e5e5);
  border-radius: var(--sl-radius-full, 9999px);
  overflow: hidden;
  position: relative; /* 为内部元素定位 */
}

/* 进度条本身 */
.sl-progress-bar {
  height: 100%;
  border-radius: var(--sl-radius-full, 9999px);
  transition: width 0.3s ease;
  min-width: 0.5px; /* 确保极小值仍有视觉反馈 */
  position: relative; /* 为渐变或叠加效果准备 */
}

/* 颜色变体 - 为CSS变量提供降级默认值 */
.sl-progress-bar--primary {
  background: linear-gradient(
    90deg,
    var(--sl-primary, #3b82f6) 0%,
    color-mix(in srgb, var(--sl-primary, #3b82f6) 80%, transparent) 100%
  );
}

.sl-progress-bar--success {
  background: linear-gradient(
    90deg,
    var(--sl-success, #10b981) 0%,
    color-mix(in srgb, var(--sl-success, #10b981) 80%, transparent) 100%
  );
}

.sl-progress-bar--warning {
  background: linear-gradient(
    90deg,
    var(--sl-warning, #f59e0b) 0%,
    color-mix(in srgb, var(--sl-warning, #f59e0b) 80%, transparent) 100%
  );
}

.sl-progress-bar--error {
  background: linear-gradient(
    90deg,
    var(--sl-error, #ef4444) 0%,
    color-mix(in srgb, var(--sl-error, #ef4444) 80%, transparent) 100%
  );
}

/* 性能优化：仅在用户需要时使用GPU加速 */
@media (prefers-reduced-motion: no-preference) {
  .sl-progress-bar {
    will-change: width;
    transform: translateZ(0); /* 触发GPU加速 */
  }
}

/* 可访问性：高对比度模式支持 */
@media (prefers-contrast: high) {
  .sl-progress-bar {
    box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.3);
  }
}

/* 打印样式优化 */
@media print {
  .sl-progress-bar {
    -webkit-print-color-adjust: exact;
    print-color-adjust: exact;
  }
}
</style>
