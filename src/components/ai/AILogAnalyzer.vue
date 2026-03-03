<template>
  <div class="ai-log-analyzer">
    <!-- 日志输入区域 -->
    <div class="log-input-section">
      <div class="section-header">
        <h4>{{ title }}</h4>
        <div class="actions">
          <button class="action-btn" @click="clearLogs" :disabled="!logs.length">
            <TrashIcon />
            清空
          </button>
          <button class="action-btn" @click="pasteFromClipboard">
            <ClipboardIcon />
            粘贴
          </button>
        </div>
      </div>

      <textarea
        v-model="logInput"
        :placeholder="placeholder"
        rows="8"
        class="log-textarea"
      ></textarea>

      <div class="log-stats" v-if="logs.length">
        <span>{{ logs.length }} 行日志</span>
        <span>{{ estimatedSize }}</span>
      </div>
    </div>

    <!-- 分析选项 -->
    <div class="analysis-options" v-if="showOptions">
      <div class="option-group">
        <label>分析类型</label>
        <select v-model="selectedType">
          <option v-for="type in analysisTypes" :key="type.value" :value="type.value">
            {{ type.label }}
          </option>
        </select>
      </div>

      <div class="option-group">
        <label>最大行数</label>
        <input type="number" v-model="maxLines" min="10" max="1000" />
      </div>
    </div>

    <!-- 分析按钮 -->
    <button class="analyze-btn" :disabled="analyzing || !logInput.trim()" @click="handleAnalyze">
      <LoadingIcon v-if="analyzing" class="spinning" />
      <SearchIcon v-else />
      {{ analyzing ? "分析中..." : "开始分析" }}
    </button>

    <!-- 分析结果 -->
    <div v-if="results.length > 0" class="analysis-results">
      <div class="results-header">
        <h4>分析结果 ({{ results.length }})</h4>
        <div class="severity-filters">
          <button
            v-for="filter in severityFilters"
            :key="filter.value"
            :class="['filter-btn', filter.value, { active: activeFilter === filter.value }]"
            @click="activeFilter = filter.value"
          >
            {{ filter.label }}
            <span class="count">{{ getSeverityCount(filter.value) }}</span>
          </button>
        </div>
      </div>

      <div class="results-list">
        <div
          v-for="(result, index) in filteredResults"
          :key="index"
          class="result-item"
          :class="result.severity"
        >
          <div class="result-header">
            <span class="severity-badge" :class="result.severity">
              {{ getSeverityLabel(result.severity) }}
            </span>
            <span class="confidence"> 置信度: {{ (result.confidence * 100).toFixed(0) }}% </span>
          </div>

          <h5 class="result-title">{{ result.title }}</h5>
          <p class="result-description">{{ result.description }}</p>

          <div v-if="result.suggestions.length > 0" class="suggestions">
            <h6>建议措施</h6>
            <ul>
              <li v-for="(suggestion, idx) in result.suggestions" :key="idx">
                {{ suggestion }}
              </li>
            </ul>
          </div>

          <div v-if="result.related_logs.length > 0 && showRelatedLogs" class="related-logs">
            <h6>相关日志</h6>
            <pre v-for="(log, idx) in result.related_logs.slice(0, 3)" :key="idx">{{ log }}</pre>
            <button
              v-if="result.related_logs.length > 3"
              class="show-more-btn"
              @click="toggleExpand(index)"
            >
              {{
                expandedItems.includes(index) ? "收起" : `展开全部 (${result.related_logs.length})`
              }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="!analyzing && hasAnalyzed" class="empty-state">
      <CheckCircleIcon class="success-icon" />
      <p>分析完成，未发现问题</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import {
  Search as SearchIcon,
  Trash2 as TrashIcon,
  ClipboardPaste as ClipboardIcon,
  Loader2 as LoadingIcon,
  CheckCircle as CheckCircleIcon,
} from "lucide-vue-next";

export interface AnalysisResult {
  analysis_type: string;
  severity: "critical" | "error" | "warning" | "info";
  title: string;
  description: string;
  suggestions: string[];
  related_logs: string[];
  confidence: number;
}

const props = withDefaults(
  defineProps<{
    title?: string;
    placeholder?: string;
    showOptions?: boolean;
    showRelatedLogs?: boolean;
    analysisTypes?: { value: string; label: string }[];
  }>(),
  {
    title: "日志分析",
    placeholder: "粘贴服务器日志内容...",
    showOptions: true,
    showRelatedLogs: true,
    analysisTypes: () => [
      { value: "full", label: "全面分析" },
      { value: "error", label: "错误检测" },
      { value: "performance", label: "性能分析" },
      { value: "security", label: "安全检查" },
      { value: "plugin", label: "插件冲突" },
    ],
  },
);

const emit = defineEmits<{
  (e: "analyze", logs: string[], options: { type: string; maxLines: number }): void;
  (e: "clear"): void;
}>();

const logInput = ref("");
const selectedType = ref("full");
const maxLines = ref(100);
const analyzing = ref(false);
const hasAnalyzed = ref(false);
const results = ref<AnalysisResult[]>([]);
const activeFilter = ref("all");
const expandedItems = ref<number[]>([]);

const defaultSeverityFilters = [
  { value: "all", label: "全部" },
  { value: "critical", label: "严重" },
  { value: "error", label: "错误" },
  { value: "warning", label: "警告" },
  { value: "info", label: "信息" },
];

const severityFilters = defaultSeverityFilters;

// 计算属性
const logs = computed(() => {
  return logInput.value.split("\n").filter((l) => l.trim());
});

const estimatedSize = computed(() => {
  const bytes = new Blob([logInput.value]).size;
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
});

const filteredResults = computed(() => {
  if (activeFilter.value === "all") return results.value;
  return results.value.filter((r) => r.severity === activeFilter.value);
});

// 方法
function getSeverityCount(severity: string): number {
  if (severity === "all") return results.value.length;
  return results.value.filter((r) => r.severity === severity).length;
}

function getSeverityLabel(severity: string): string {
  const labels: Record<string, string> = {
    critical: "严重",
    error: "错误",
    warning: "警告",
    info: "信息",
  };
  return labels[severity] || severity;
}

async function handleAnalyze() {
  if (!logInput.value.trim() || analyzing.value) return;

  analyzing.value = true;
  hasAnalyzed.value = false;

  try {
    emit("analyze", logs.value, {
      type: selectedType.value,
      maxLines: maxLines.value,
    });
    // 实际分析在外部处理，这里模拟结果
    hasAnalyzed.value = true;
  } finally {
    analyzing.value = false;
  }
}

function clearLogs() {
  logInput.value = "";
  results.value = [];
  hasAnalyzed.value = false;
  emit("clear");
}

async function pasteFromClipboard() {
  try {
    const text = await navigator.clipboard.readText();
    logInput.value = text;
  } catch (err) {
    console.error("Failed to paste:", err);
  }
}

function toggleExpand(index: number) {
  const idx = expandedItems.value.indexOf(index);
  if (idx > -1) {
    expandedItems.value.splice(idx, 1);
  } else {
    expandedItems.value.push(index);
  }
}

// 暴露方法供父组件使用
defineExpose({
  setResults: (newResults: AnalysisResult[]) => {
    results.value = newResults;
    hasAnalyzed.value = true;
  },
  getLogs: () => logs.value,
  clear: clearLogs,
});
</script>

<style scoped>
.ai-log-analyzer {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.log-input-section {
  background: var(--sl-card-bg);
  border-radius: 8px;
  padding: 16px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.section-header h4 {
  margin: 0;
  font-size: 14px;
}

.actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 10px;
  border: 1px solid var(--sl-border);
  border-radius: 4px;
  background: transparent;
  color: var(--sl-text-secondary);
  font-size: 12px;
  cursor: pointer;
}

.action-btn:hover:not(:disabled) {
  background: var(--sl-bg-secondary);
  color: var(--sl-text);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.log-textarea {
  width: 100%;
  padding: 12px;
  border: 1px solid var(--sl-border);
  border-radius: 6px;
  background: var(--sl-input-bg);
  color: var(--sl-text);
  font-family: monospace;
  font-size: 12px;
  resize: vertical;
  min-height: 150px;
}

.log-textarea:focus {
  outline: none;
  border-color: var(--sl-primary);
}

.log-stats {
  display: flex;
  gap: 16px;
  margin-top: 8px;
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.analysis-options {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.option-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.option-group label {
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.option-group select,
.option-group input {
  padding: 8px 12px;
  border: 1px solid var(--sl-border);
  border-radius: 4px;
  background: var(--sl-input-bg);
  color: var(--sl-text);
  font-size: 14px;
  min-width: 140px;
}

.analyze-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 12px;
  border: none;
  border-radius: 6px;
  background: var(--sl-primary);
  color: white;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.2s;
}

.analyze-btn:hover:not(:disabled) {
  background: var(--sl-primary-dark);
}

.analyze-btn:disabled {
  background: var(--sl-disabled);
  cursor: not-allowed;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.analysis-results {
  background: var(--sl-card-bg);
  border-radius: 8px;
  padding: 16px;
}

.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  flex-wrap: wrap;
  gap: 12px;
}

.results-header h4 {
  margin: 0;
  font-size: 14px;
}

.severity-filters {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.filter-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border: 1px solid var(--sl-border);
  border-radius: 12px;
  background: transparent;
  color: var(--sl-text-secondary);
  font-size: 12px;
  cursor: pointer;
}

.filter-btn .count {
  background: var(--sl-bg-secondary);
  padding: 2px 6px;
  border-radius: 8px;
  font-size: 10px;
}

.filter-btn.active,
.filter-btn:hover {
  background: var(--sl-bg-secondary);
  color: var(--sl-text);
}

.filter-btn.critical {
  border-color: #ff4d4f;
}
.filter-btn.error {
  border-color: #ff7a45;
}
.filter-btn.warning {
  border-color: #faad14;
}
.filter-btn.info {
  border-color: #1890ff;
}

.results-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.result-item {
  padding: 12px;
  border-radius: 6px;
  background: var(--sl-bg-secondary);
  border-left: 3px solid var(--sl-border);
}

.result-item.critical {
  border-left-color: #ff4d4f;
}
.result-item.error {
  border-left-color: #ff7a45;
}
.result-item.warning {
  border-left-color: #faad14;
}
.result-item.info {
  border-left-color: #1890ff;
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
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

.confidence {
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.result-title {
  margin: 0 0 4px 0;
  font-size: 14px;
}

.result-description {
  margin: 0 0 8px 0;
  font-size: 13px;
  color: var(--sl-text-secondary);
}

.suggestions h6,
.related-logs h6 {
  margin: 12px 0 6px 0;
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.suggestions ul {
  margin: 0;
  padding-left: 18px;
  font-size: 13px;
}

.suggestions li {
  margin-bottom: 4px;
}

.related-logs pre {
  margin: 4px 0;
  padding: 8px;
  background: var(--sl-bg);
  border-radius: 4px;
  font-size: 11px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

.show-more-btn {
  padding: 4px 8px;
  border: none;
  background: transparent;
  color: var(--sl-primary);
  font-size: 12px;
  cursor: pointer;
}

.show-more-btn:hover {
  text-decoration: underline;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px;
  background: var(--sl-card-bg);
  border-radius: 8px;
}

.success-icon {
  width: 48px;
  height: 48px;
  color: var(--sl-success);
  margin-bottom: 12px;
}

.empty-state p {
  margin: 0;
  color: var(--sl-text-secondary);
}
</style>
