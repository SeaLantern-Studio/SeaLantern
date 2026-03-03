<template>
  <div class="ai-suggestion-card" :class="[severity, { compact }]">
    <!-- 严重程度指示器 -->
    <div class="severity-indicator" :class="severity"></div>

    <!-- 卡片内容 -->
    <div class="card-content">
      <!-- 标题区域 -->
      <div class="card-header">
        <div class="header-left">
          <component :is="severityIcon" class="severity-icon" v-if="showIcon" />
          <h5 class="card-title">{{ title }}</h5>
        </div>
        <div class="header-right">
          <span class="severity-badge" :class="severity" v-if="showSeverity">
            {{ severityLabel }}
          </span>
          <span class="confidence-badge" v-if="showConfidence">
            {{ (confidence * 100).toFixed(0) }}%
          </span>
        </div>
      </div>

      <!-- 描述 -->
      <p class="card-description" v-if="description">{{ description }}</p>

      <!-- 建议列表 -->
      <div class="suggestions-section" v-if="suggestions.length > 0">
        <h6 class="section-title">
          <LightbulbIcon class="section-icon" />
          建议措施
        </h6>
        <ul class="suggestions-list">
          <li v-for="(suggestion, index) in displaySuggestions" :key="index">
            <CheckIcon class="check-icon" />
            <span>{{ suggestion }}</span>
          </li>
        </ul>
        <button
          v-if="suggestions.length > maxSuggestions"
          class="show-more-btn"
          @click="expanded = !expanded"
        >
          {{ expanded ? "收起" : `展开全部 (${suggestions.length})` }}
        </button>
      </div>

      <!-- 操作按钮 -->
      <div class="card-actions" v-if="actions.length > 0">
        <button
          v-for="action in actions"
          :key="action.id"
          :class="['action-btn', action.type || 'default']"
          @click="handleAction(action)"
        >
          <component :is="action.icon" v-if="action.icon" class="btn-icon" />
          {{ action.label }}
        </button>
      </div>

      <!-- 详情展开区 -->
      <div class="details-section" v-if="details && showDetails">
        <button class="details-toggle" @click="detailsExpanded = !detailsExpanded">
          <ChevronDownIcon :class="{ rotated: detailsExpanded }" />
          {{ detailsExpanded ? "收起详情" : "查看详情" }}
        </button>
        <div class="details-content" v-if="detailsExpanded">
          <pre>{{ details }}</pre>
        </div>
      </div>
    </div>

    <!-- 关闭按钮 -->
    <button class="close-btn" v-if="dismissible" @click="emit('dismiss')">
      <XIcon />
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import {
  AlertTriangle as WarningIcon,
  AlertCircle as ErrorIcon,
  XCircle as CriticalIcon,
  Info as InfoIcon,
  Lightbulb as LightbulbIcon,
  Check as CheckIcon,
  ChevronDown as ChevronDownIcon,
  X as XIcon,
} from "lucide-vue-next";

export interface CardAction {
  id: string;
  label: string;
  type?: "primary" | "secondary" | "danger";
  icon?: any;
  handler?: () => void;
}

const props = withDefaults(
  defineProps<{
    title: string;
    description?: string;
    severity?: "critical" | "error" | "warning" | "info";
    confidence?: number;
    suggestions?: string[];
    actions?: CardAction[];
    details?: string;
    showIcon?: boolean;
    showSeverity?: boolean;
    showConfidence?: boolean;
    showDetails?: boolean;
    compact?: boolean;
    dismissible?: boolean;
    maxSuggestions?: number;
  }>(),
  {
    severity: "info",
    confidence: 0.8,
    suggestions: () => [],
    actions: () => [],
    showIcon: true,
    showSeverity: true,
    showConfidence: true,
    showDetails: true,
    compact: false,
    dismissible: false,
    maxSuggestions: 3,
  },
);

const emit = defineEmits<{
  (e: "action", action: CardAction): void;
  (e: "dismiss"): void;
}>();

const expanded = ref(false);
const detailsExpanded = ref(false);

// 计算属性
const severityIcon = computed(() => {
  switch (props.severity) {
    case "critical":
      return CriticalIcon;
    case "error":
      return ErrorIcon;
    case "warning":
      return WarningIcon;
    default:
      return InfoIcon;
  }
});

const severityLabel = computed(() => {
  const labels: Record<string, string> = {
    critical: "严重",
    error: "错误",
    warning: "警告",
    info: "信息",
  };
  return labels[props.severity] || props.severity;
});

const displaySuggestions = computed(() => {
  if (expanded.value || props.suggestions.length <= props.maxSuggestions) {
    return props.suggestions;
  }
  return props.suggestions.slice(0, props.maxSuggestions);
});

// 方法
function handleAction(action: CardAction) {
  if (action.handler) {
    action.handler();
  }
  emit("action", action);
}
</script>

<style scoped>
.ai-suggestion-card {
  position: relative;
  display: flex;
  background: var(--sl-card-bg);
  border-radius: 8px;
  overflow: hidden;
  transition: box-shadow 0.2s;
}

.ai-suggestion-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.ai-suggestion-card.compact {
  padding: 0;
}

.severity-indicator {
  width: 4px;
  flex-shrink: 0;
}

.severity-indicator.critical {
  background: #ff4d4f;
}
.severity-indicator.error {
  background: #ff7a45;
}
.severity-indicator.warning {
  background: #faad14;
}
.severity-indicator.info {
  background: #1890ff;
}

.card-content {
  flex: 1;
  padding: 16px;
}

.compact .card-content {
  padding: 12px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 8px;
  gap: 12px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.severity-icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
}

.severity-icon.critical,
.ai-suggestion-card.critical .severity-icon {
  color: #ff4d4f;
}
.severity-icon.error,
.ai-suggestion-card.error .severity-icon {
  color: #ff7a45;
}
.severity-icon.warning,
.ai-suggestion-card.warning .severity-icon {
  color: #faad14;
}
.severity-icon.info,
.ai-suggestion-card.info .severity-icon {
  color: #1890ff;
}

.card-title {
  margin: 0;
  font-size: 14px;
  font-weight: 500;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.severity-badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
}

.severity-badge.critical {
  background: #ff4d4f20;
  color: #ff4d4f;
}
.severity-badge.error {
  background: #ff7a4520;
  color: #ff7a45;
}
.severity-badge.warning {
  background: #faad1420;
  color: #faad14;
}
.severity-badge.info {
  background: #1890ff20;
  color: #1890ff;
}

.confidence-badge {
  padding: 2px 6px;
  background: var(--sl-bg-secondary);
  border-radius: 4px;
  font-size: 11px;
  color: var(--sl-text-secondary);
}

.card-description {
  margin: 0 0 12px 0;
  font-size: 13px;
  color: var(--sl-text-secondary);
  line-height: 1.5;
}

.compact .card-description {
  margin-bottom: 8px;
  font-size: 12px;
}

.suggestions-section {
  margin-bottom: 12px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 0 0 8px 0;
  font-size: 12px;
  font-weight: 500;
  color: var(--sl-text-secondary);
}

.section-icon {
  width: 14px;
  height: 14px;
}

.suggestions-list {
  margin: 0;
  padding: 0;
  list-style: none;
}

.suggestions-list li {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 4px 0;
  font-size: 13px;
}

.compact .suggestions-list li {
  font-size: 12px;
}

.check-icon {
  width: 14px;
  height: 14px;
  color: var(--sl-success);
  flex-shrink: 0;
  margin-top: 2px;
}

.show-more-btn {
  padding: 4px 0;
  border: none;
  background: transparent;
  color: var(--sl-primary);
  font-size: 12px;
  cursor: pointer;
}

.show-more-btn:hover {
  text-decoration: underline;
}

.card-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border: 1px solid var(--sl-border);
  border-radius: 6px;
  background: transparent;
  color: var(--sl-text);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  background: var(--sl-bg-secondary);
}

.action-btn.primary {
  background: var(--sl-primary);
  border-color: var(--sl-primary);
  color: white;
}

.action-btn.primary:hover {
  background: var(--sl-primary-dark);
}

.action-btn.danger {
  background: #ff4d4f;
  border-color: #ff4d4f;
  color: white;
}

.action-btn.danger:hover {
  background: #ff7875;
}

.btn-icon {
  width: 14px;
  height: 14px;
}

.details-section {
  margin-top: 12px;
  border-top: 1px solid var(--sl-border);
  padding-top: 12px;
}

.details-toggle {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--sl-text-secondary);
  font-size: 12px;
  cursor: pointer;
}

.details-toggle:hover {
  color: var(--sl-text);
}

.details-toggle svg {
  width: 14px;
  height: 14px;
  transition: transform 0.2s;
}

.details-toggle svg.rotated {
  transform: rotate(180deg);
}

.details-content {
  margin-top: 10px;
}

.details-content pre {
  margin: 0;
  padding: 12px;
  background: var(--sl-bg-secondary);
  border-radius: 4px;
  font-size: 12px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

.close-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  padding: 4px;
  border: none;
  background: transparent;
  color: var(--sl-text-secondary);
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.2s;
}

.ai-suggestion-card:hover .close-btn {
  opacity: 1;
}

.close-btn:hover {
  color: var(--sl-text);
}

.close-btn svg {
  width: 14px;
  height: 14px;
}
</style>
