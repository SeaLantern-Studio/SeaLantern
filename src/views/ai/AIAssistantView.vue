<template>
  <div class="ai-assistant-view">
    <!-- 标题栏 -->
    <div class="ai-header">
      <div class="ai-title">
        <Brain class="ai-icon" />
        <h2>{{ t("ai.assistant.title") }}</h2>
      </div>
      <div class="ai-status" :class="{ enabled: aiEnabled }">
        <span class="status-dot"></span>
        {{ aiEnabled ? t("ai.status.enabled") : t("ai.status.disabled") }}
      </div>
    </div>

    <!-- Tab 切换 -->
    <SLTabBar :tabs="tabs" v-model="activeTab" />

    <!-- 日志分析 Tab -->
    <div v-if="activeTab === 'log'" class="tab-content">
      <div class="section-card">
        <h3>{{ t("ai.log_analysis.title") }}</h3>

        <div class="analysis-options">
          <div class="option-group">
            <label>{{ t("ai.log_analysis.analysis_type") }}</label>
            <select v-model="logAnalysisType">
              <option value="full">{{ t("ai.log_analysis.type_full") }}</option>
              <option value="error">{{ t("ai.log_analysis.type_error") }}</option>
              <option value="performance">{{ t("ai.log_analysis.type_performance") }}</option>
              <option value="security">{{ t("ai.log_analysis.type_security") }}</option>
              <option value="plugin">{{ t("ai.log_analysis.type_plugin") }}</option>
            </select>
          </div>

          <div class="option-group">
            <label>{{ t("ai.log_analysis.max_lines") }}</label>
            <input type="number" v-model="maxLines" min="10" max="1000" />
          </div>
        </div>

        <div class="log-input">
          <textarea
            v-model="logInput"
            :placeholder="t('ai.log_analysis.input_placeholder')"
            rows="10"
          ></textarea>
        </div>

        <button class="analyze-btn" @click="analyzeLogs" :disabled="analyzing || !logInput.trim()">
          <LoadingIcon v-if="analyzing" class="spinning" />
          <SearchIcon v-else />
          {{ analyzing ? t("ai.log_analysis.analyzing") : t("ai.log_analysis.analyze") }}
        </button>

        <!-- 分析结果 -->
        <div v-if="analysisResults.length > 0" class="analysis-results">
          <h4>{{ t("ai.log_analysis.results") }}</h4>
          <div
            v-for="(result, index) in analysisResults"
            :key="index"
            class="result-card"
            :class="result.severity"
          >
            <div class="result-header">
              <span class="severity-badge" :class="result.severity">
                {{ getSeverityText(result.severity) }}
              </span>
              <span class="confidence"
                >{{ t("ai.confidence") }}: {{ (result.confidence * 100).toFixed(0) }}%</span
              >
            </div>
            <h5 class="result-title">{{ result.title }}</h5>
            <p class="result-description">{{ result.description }}</p>

            <div v-if="result.suggestions.length > 0" class="suggestions">
              <h6>{{ t("ai.suggestions") }}</h6>
              <ul>
                <li v-for="(suggestion, idx) in result.suggestions" :key="idx">
                  {{ suggestion }}
                </li>
              </ul>
            </div>

            <div v-if="result.related_logs.length > 0" class="related-logs">
              <h6>{{ t("ai.related_logs") }}</h6>
              <pre v-for="(log, idx) in result.related_logs" :key="idx">{{ log }}</pre>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 命令助手 Tab -->
    <div v-if="activeTab === 'command'" class="tab-content">
      <div class="section-card">
        <h3>{{ t("ai.command.title") }}</h3>

        <div class="command-input">
          <input
            v-model="naturalLanguageInput"
            :placeholder="t('ai.command.input_placeholder')"
            @keyup.enter="generateCommand"
          />
          <button @click="generateCommand" :disabled="generating || !naturalLanguageInput.trim()">
            <LoadingIcon v-if="generating" class="spinning" />
            <SendIcon v-else />
          </button>
        </div>

        <div class="command-options">
          <div class="option-group">
            <label>{{ t("ai.command.server_type") }}</label>
            <select v-model="commandServerType">
              <option value="">{{ t("common.auto") }}</option>
              <option value="vanilla">Vanilla</option>
              <option value="spigot">Spigot</option>
              <option value="paper">Paper</option>
              <option value="fabric">Fabric</option>
              <option value="forge">Forge</option>
            </select>
          </div>
        </div>

        <!-- 命令建议 -->
        <div v-if="commandSuggestion" class="command-result">
          <div class="generated-command">
            <code>{{ commandSuggestion.command }}</code>
            <button class="copy-btn" @click="copyCommand">
              <CopyIcon />
            </button>
          </div>
          <p class="command-description">{{ commandSuggestion.description }}</p>

          <div v-if="commandSuggestion.parameters.length > 0" class="parameters">
            <h6>{{ t("ai.command.parameters") }}</h6>
            <div class="param-list">
              <div
                v-for="param in commandSuggestion.parameters"
                :key="param.name"
                class="param-item"
              >
                <span class="param-name">{{ param.name }}</span>
                <span class="param-type">{{ param.param_type }}</span>
                <span v-if="param.required" class="param-required">{{
                  t("ai.command.required")
                }}</span>
                <span class="param-desc">{{ param.description }}</span>
              </div>
            </div>
          </div>

          <div v-if="commandSuggestion.examples.length > 0" class="examples">
            <h6>{{ t("ai.command.examples") }}</h6>
            <code v-for="(example, idx) in commandSuggestion.examples" :key="idx">{{
              example
            }}</code>
          </div>
        </div>

        <!-- 快捷命令 -->
        <div class="quick-commands">
          <h6>{{ t("ai.command.quick_commands") }}</h6>
          <div class="quick-command-list">
            <button
              v-for="cmd in quickCommands"
              :key="cmd.text"
              @click="
                naturalLanguageInput = cmd.text;
                generateCommand();
              "
            >
              {{ cmd.label }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 配置优化 Tab -->
    <div v-if="activeTab === 'config'" class="tab-content">
      <div class="section-card">
        <h3>{{ t("ai.config.title") }}</h3>

        <div class="config-options">
          <div class="option-group">
            <label>{{ t("ai.config.config_type") }}</label>
            <select v-model="configType">
              <option value="server.properties">server.properties</option>
              <option value="jvm">JVM 参数</option>
              <option value="spigot.yml">spigot.yml</option>
              <option value="paper.yml">paper.yml</option>
            </select>
          </div>
        </div>

        <div v-if="configType === 'jvm'" class="jvm-config">
          <h6>{{ t("ai.config.hardware_info") }}</h6>
          <div class="hardware-form">
            <div class="form-row">
              <div class="form-group">
                <label>{{ t("ai.config.cpu_cores") }}</label>
                <input type="number" v-model="hardwareInfo.cpu_cores" min="1" />
              </div>
              <div class="form-group">
                <label>{{ t("ai.config.total_memory") }} (GB)</label>
                <input type="number" v-model="hardwareInfo.total_memory_gb" min="1" step="0.5" />
              </div>
            </div>
            <div class="form-row">
              <div class="form-group">
                <label>{{ t("ai.config.expected_players") }}</label>
                <input type="number" v-model="expectedPlayers" min="1" />
              </div>
            </div>
          </div>

          <button class="analyze-btn" @click="analyzeJVM" :disabled="analyzingConfig">
            <LoadingIcon v-if="analyzingConfig" class="spinning" />
            <SettingsIcon v-else />
            {{ analyzingConfig ? t("ai.config.analyzing") : t("ai.config.analyze_jvm") }}
          </button>
        </div>

        <div v-else class="config-textarea">
          <textarea
            v-model="configInput"
            :placeholder="t('ai.config.input_placeholder')"
            rows="15"
          ></textarea>
          <button
            class="analyze-btn"
            @click="analyzeConfig"
            :disabled="analyzingConfig || !configInput.trim()"
          >
            <LoadingIcon v-if="analyzingConfig" class="spinning" />
            <SettingsIcon v-else />
            {{ analyzingConfig ? t("ai.config.analyzing") : t("ai.config.analyze") }}
          </button>
        </div>

        <!-- JVM 分析结果 -->
        <div v-if="jvmAnalysis" class="jvm-result">
          <h4>{{ t("ai.config.jvm_result") }}</h4>

          <div class="memory-suggestion">
            <h6>{{ t("ai.config.memory_suggestion") }}</h6>
            <p>{{ jvmAnalysis.memory_suggestion.explanation }}</p>
            <div class="memory-values">
              <span>-Xms{{ jvmAnalysis.memory_suggestion.min_heap_mb }}M</span>
              <span>-Xmx{{ jvmAnalysis.memory_suggestion.max_heap_mb }}M</span>
            </div>
          </div>

          <div class="gc-suggestion">
            <h6>{{ t("ai.config.gc_suggestion") }}</h6>
            <p>{{ jvmAnalysis.gc_suggestion.explanation }}</p>
            <code>{{ jvmAnalysis.gc_suggestion.gc_type }}</code>
          </div>

          <div class="jvm-args">
            <h6>{{ t("ai.config.recommended_args") }}</h6>
            <pre>{{
              jvmAnalysis.suggested_args.join(
                " \
  ",
              )
            }}</pre>
            <button class="copy-btn" @click="copyJVMArgs">
              <CopyIcon />
              {{ t("common.copy") }}
            </button>
          </div>
        </div>

        <!-- 配置建议 -->
        <div v-if="configSuggestions.length > 0" class="config-suggestions">
          <h4>{{ t("ai.config.suggestions") }}</h4>
          <div
            v-for="(suggestion, index) in configSuggestions"
            :key="index"
            class="suggestion-item"
            :class="'priority-' + suggestion.priority"
          >
            <div class="suggestion-header">
              <span class="config-key">{{ suggestion.config_key }}</span>
              <span class="priority">{{ t("ai.config.priority") }}: {{ suggestion.priority }}</span>
            </div>
            <div class="suggestion-values">
              <span class="current"
                >{{ t("ai.config.current") }}: {{ suggestion.current_value }}</span
              >
              <ArrowRightIcon class="arrow" />
              <span class="suggested"
                >{{ t("ai.config.suggested") }}: {{ suggestion.suggested_value }}</span
              >
            </div>
            <p class="suggestion-reason">{{ suggestion.reason }}</p>
            <p class="suggestion-effect">{{ suggestion.expected_effect }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 翻译 Tab -->
    <div v-if="activeTab === 'translate'" class="tab-content">
      <div class="section-card">
        <h3>{{ t("ai.translate.title") }}</h3>

        <div class="translate-options">
          <div class="option-group">
            <label>{{ t("ai.translate.source") }}</label>
            <select v-model="translateOptions.source_language">
              <option value="auto">{{ t("ai.translate.auto_detect") }}</option>
              <option v-for="lang in supportedLanguages" :key="lang.code" :value="lang.code">
                {{ lang.display_name }}
              </option>
            </select>
          </div>
          <button class="swap-btn" @click="swapLanguages">
            <ArrowLeftRightIcon />
          </button>
          <div class="option-group">
            <label>{{ t("ai.translate.target") }}</label>
            <select v-model="translateOptions.target_language">
              <option v-for="lang in supportedLanguages" :key="lang.code" :value="lang.code">
                {{ lang.display_name }}
              </option>
            </select>
          </div>
        </div>

        <div class="translate-input">
          <textarea
            v-model="translateInput"
            :placeholder="t('ai.translate.input_placeholder')"
            rows="5"
          ></textarea>
        </div>

        <button
          class="translate-btn"
          @click="translateText"
          :disabled="translating || !translateInput.trim()"
        >
          <LoadingIcon v-if="translating" class="spinning" />
          <LanguagesIcon v-else />
          {{ translating ? t("ai.translate.translating") : t("ai.translate.translate") }}
        </button>

        <div v-if="translateResult" class="translate-result">
          <h6>{{ t("ai.translate.result") }}</h6>
          <p class="translated-text">{{ translateResult.translated_text }}</p>
          <div class="translate-meta">
            <span>{{ t("ai.translate.detected") }}: {{ translateResult.source_language }}</span>
            <span
              >{{ t("ai.confidence") }}: {{ (translateResult.confidence * 100).toFixed(0) }}%</span
            >
          </div>
          <button class="copy-btn" @click="copyTranslation">
            <CopyIcon />
            {{ t("common.copy") }}
          </button>
        </div>
      </div>
    </div>

    <!-- 内容生成 Tab -->
    <div v-if="activeTab === 'content'" class="tab-content">
      <div class="section-card">
        <h3>{{ t("ai.content.title") }}</h3>

        <div class="content-options">
          <div class="option-group">
            <label>{{ t("ai.content.type") }}</label>
            <select v-model="contentType">
              <option value="announcement">{{ t("ai.content.type_announcement") }}</option>
              <option value="rules">{{ t("ai.content.type_rules") }}</option>
              <option value="event">{{ t("ai.content.type_event") }}</option>
              <option value="welcome">{{ t("ai.content.type_welcome") }}</option>
            </select>
          </div>

          <div class="option-group">
            <label>{{ t("ai.content.style") }}</label>
            <select v-model="contentStyle">
              <option value="">{{ t("common.auto") }}</option>
              <option value="formal">{{ t("ai.content.style_formal") }}</option>
              <option value="casual">{{ t("ai.content.style_casual") }}</option>
              <option value="friendly">{{ t("ai.content.style_friendly") }}</option>
            </select>
          </div>
        </div>

        <div class="content-form">
          <div class="form-group">
            <label>{{ t("ai.content.server_name") }}</label>
            <input
              v-model="contentServerName"
              :placeholder="t('ai.content.server_name_placeholder')"
            />
          </div>

          <div v-if="contentType === 'announcement'" class="form-group">
            <label>{{ t("ai.content.topic") }}</label>
            <input v-model="contentTopic" :placeholder="t('ai.content.topic_placeholder')" />
          </div>

          <div class="form-group">
            <label>{{ t("ai.content.features") }}</label>
            <textarea
              v-model="contentFeatures"
              :placeholder="t('ai.content.features_placeholder')"
              rows="3"
            ></textarea>
          </div>
        </div>

        <button class="generate-btn" @click="generateContent" :disabled="generatingContent">
          <LoadingIcon v-if="generatingContent" class="spinning" />
          <SparklesIcon v-else />
          {{ generatingContent ? t("ai.content.generating") : t("ai.content.generate") }}
        </button>

        <div v-if="generatedContent" class="generated-content">
          <h6>{{ t("ai.content.result") }}</h6>
          <pre>{{ generatedContent.content }}</pre>
          <div class="content-meta">
            <span>{{ t("ai.content.type") }}: {{ generatedContent.content_type }}</span>
            <span
              >{{ t("ai.confidence") }}: {{ (generatedContent.confidence * 100).toFixed(0) }}%</span
            >
          </div>
          <button class="copy-btn" @click="copyContent">
            <CopyIcon />
            {{ t("common.copy") }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useI18n } from "vue-i18n";
import {
  Brain,
  Search as SearchIcon,
  Send as SendIcon,
  Copy as CopyIcon,
  Settings as SettingsIcon,
  Languages as LanguagesIcon,
  Sparkles as SparklesIcon,
  ArrowRight as ArrowRightIcon,
  ArrowLeftRight as ArrowLeftRightIcon,
  Loader2 as LoadingIcon,
} from "lucide-vue-next";

import SLTabBar from "@components/SLTabBar.vue";

import {
  analyzeLogs,
  generateCommand,
  analyzeJVM,
  analyzeConfig,
  translateText,
  generateContent,
  generateAnnouncement,
  getSupportedLanguages,
  checkAIAvailable,
  type AIAnalysisResult,
  type AICommandSuggestion,
  type JVMAnalysisResult,
  type AIConfigSuggestion,
  type AITranslationResult,
  type AIContentGeneration,
  type LanguageInfo,
  type HardwareInfo,
} from "@api/ai";

const { t } = useI18n();

// AI 状态
const aiEnabled = ref(false);

// Tab 管理
const activeTab = ref("log");
const tabs = [
  { id: "log", label: computed(() => t("ai.tabs.log_analysis")) },
  { id: "command", label: computed(() => t("ai.tabs.command")) },
  { id: "config", label: computed(() => t("ai.tabs.config")) },
  { id: "translate", label: computed(() => t("ai.tabs.translate")) },
  { id: "content", label: computed(() => t("ai.tabs.content")) },
];

// ==================== 日志分析 ====================
const logInput = ref("");
const logAnalysisType = ref("full");
const maxLines = ref(100);
const analyzing = ref(false);
const analysisResults = ref<AIAnalysisResult[]>([]);

async function handleAnalyzeLogs() {
  if (!logInput.value.trim()) return;

  analyzing.value = true;
  try {
    const logs = logInput.value.split("\n").filter((l) => l.trim());
    analysisResults.value = await analyzeLogs({
      logs,
      analysis_type: logAnalysisType.value as any,
      max_lines: maxLines.value,
    });
  } catch (error) {
    console.error("Failed to analyze logs:", error);
  } finally {
    analyzing.value = false;
  }
}

function getSeverityText(severity: string): string {
  const map: Record<string, string> = {
    critical: t("ai.severity.critical"),
    error: t("ai.severity.error"),
    warning: t("ai.severity.warning"),
    info: t("ai.severity.info"),
  };
  return map[severity] || severity;
}

// ==================== 命令助手 ====================
const naturalLanguageInput = ref("");
const commandServerType = ref("");
const generating = ref(false);
const commandSuggestion = ref<AICommandSuggestion | null>(null);

const quickCommands = [
  { label: t("ai.quick_commands.gamemode"), text: "设置玩家游戏模式为创造模式" },
  { label: t("ai.quick_commands.tp"), text: "传送到出生点" },
  { label: t("ai.quick_commands.give"), text: "给予玩家64个钻石" },
  { label: t("ai.quick_commands.time"), text: "设置时间为白天" },
  { label: t("ai.quick_commands.weather"), text: "设置天气为晴天" },
];

async function handleGenerateCommand() {
  if (!naturalLanguageInput.value.trim()) return;

  generating.value = true;
  try {
    commandSuggestion.value = await generateCommand({
      natural_language: naturalLanguageInput.value,
      server_type: commandServerType.value || undefined,
    });
  } catch (error) {
    console.error("Failed to generate command:", error);
  } finally {
    generating.value = false;
  }
}

async function copyCommand() {
  if (commandSuggestion.value) {
    await navigator.clipboard.writeText(commandSuggestion.value.command);
  }
}

// ==================== 配置优化 ====================
const configType = ref("server.properties");
const configInput = ref("");
const analyzingConfig = ref(false);
const configSuggestions = ref<AIConfigSuggestion[]>([]);
const jvmAnalysis = ref<JVMAnalysisResult | null>(null);

const hardwareInfo = ref<HardwareInfo>({
  cpu_cores: 4,
  total_memory_gb: 8,
  available_memory_gb: 6,
  is_ssd: true,
  os: "Windows",
});
const expectedPlayers = ref(20);

async function handleAnalyzeJVM() {
  analyzingConfig.value = true;
  try {
    jvmAnalysis.value = await analyzeJVM({
      current_args: [],
      hardware: hardwareInfo.value,
      expected_players: expectedPlayers.value,
    });
  } catch (error) {
    console.error("Failed to analyze JVM:", error);
  } finally {
    analyzingConfig.value = false;
  }
}

async function handleAnalyzeConfig() {
  if (!configInput.value.trim()) return;

  analyzingConfig.value = true;
  try {
    configSuggestions.value = await analyzeConfig({
      config_content: configInput.value,
      config_type: configType.value,
    });
  } catch (error) {
    console.error("Failed to analyze config:", error);
  } finally {
    analyzingConfig.value = false;
  }
}

async function copyJVMArgs() {
  if (jvmAnalysis.value) {
    await navigator.clipboard.writeText(jvmAnalysis.value.suggested_args.join(" "));
  }
}

// ==================== 翻译 ====================
const translateInput = ref("");
const translating = ref(false);
const translateResult = ref<AITranslationResult | null>(null);
const supportedLanguages = ref<LanguageInfo[]>([]);

const translateOptions = ref({
  source_language: "auto",
  target_language: "zh-CN",
});

async function handleTranslate() {
  if (!translateInput.value.trim()) return;

  translating.value = true;
  try {
    translateResult.value = await translateText({
      text: translateInput.value,
      source_language: translateOptions.value.source_language,
      target_language: translateOptions.value.target_language,
    });
  } catch (error) {
    console.error("Failed to translate:", error);
  } finally {
    translating.value = false;
  }
}

function swapLanguages() {
  if (translateOptions.value.source_language !== "auto") {
    const temp = translateOptions.value.source_language;
    translateOptions.value.source_language = translateOptions.value.target_language;
    translateOptions.value.target_language = temp;
  }
}

async function copyTranslation() {
  if (translateResult.value) {
    await navigator.clipboard.writeText(translateResult.value.translated_text);
  }
}

// ==================== 内容生成 ====================
const contentType = ref("announcement");
const contentStyle = ref("");
const contentServerName = ref("");
const contentTopic = ref("");
const contentFeatures = ref("");
const generatingContent = ref(false);
const generatedContent = ref<AIContentGeneration | null>(null);

async function handleGenerateContent() {
  if (!contentServerName.value.trim()) return;

  generatingContent.value = true;
  try {
    generatedContent.value = await generateContent({
      content_type: contentType.value as any,
      server_name: contentServerName.value,
      server_features: contentFeatures.value.split("\n").filter((f) => f.trim()),
      style: (contentStyle.value as any) || undefined,
    });
  } catch (error) {
    console.error("Failed to generate content:", error);
  } finally {
    generatingContent.value = false;
  }
}

async function copyContent() {
  if (generatedContent.value) {
    await navigator.clipboard.writeText(generatedContent.value.content);
  }
}

// ==================== 初始化 ====================
onMounted(async () => {
  aiEnabled.value = await checkAIAvailable();
  supportedLanguages.value = await getSupportedLanguages();
});
</script>

<style scoped>
.ai-assistant-view {
  padding: 20px;
  height: 100%;
  overflow-y: auto;
}

.ai-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.ai-title {
  display: flex;
  align-items: center;
  gap: 10px;
}

.ai-icon {
  width: 24px;
  height: 24px;
  color: var(--sl-primary);
}

.ai-title h2 {
  margin: 0;
  font-size: 18px;
}

.ai-status {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--sl-error);
}

.ai-status.enabled .status-dot {
  background: var(--sl-success);
}

.tab-content {
  margin-top: 20px;
}

.section-card {
  background: var(--sl-card-bg);
  border-radius: 8px;
  padding: 20px;
}

.section-card h3 {
  margin: 0 0 15px 0;
  font-size: 16px;
}

/* 分析选项 */
.analysis-options,
.command-options,
.config-options,
.content-options,
.translate-options {
  display: flex;
  gap: 15px;
  margin-bottom: 15px;
  flex-wrap: wrap;
}

.option-group {
  display: flex;
  flex-direction: column;
  gap: 5px;
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
  min-width: 150px;
}

/* 日志输入 */
.log-input textarea,
.config-textarea textarea,
.translate-input textarea,
.content-form textarea {
  width: 100%;
  padding: 12px;
  border: 1px solid var(--sl-border);
  border-radius: 6px;
  background: var(--sl-input-bg);
  color: var(--sl-text);
  font-family: monospace;
  font-size: 13px;
  resize: vertical;
}

/* 按钮 */
.analyze-btn,
.translate-btn,
.generate-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 12px;
  margin-top: 15px;
  border: none;
  border-radius: 6px;
  background: var(--sl-primary);
  color: white;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.2s;
}

.analyze-btn:hover,
.translate-btn:hover,
.generate-btn:hover {
  background: var(--sl-primary-dark);
}

.analyze-btn:disabled,
.translate-btn:disabled,
.generate-btn:disabled {
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

/* 分析结果 */
.analysis-results {
  margin-top: 20px;
}

.analysis-results h4,
.config-suggestions h4,
.jvm-result h4,
.translate-result h6,
.generated-content h6 {
  margin: 0 0 10px 0;
  font-size: 14px;
  color: var(--sl-text-secondary);
}

.result-card {
  padding: 15px;
  margin-bottom: 10px;
  border-radius: 6px;
  background: var(--sl-bg-secondary);
  border-left: 3px solid var(--sl-border);
}

.result-card.critical {
  border-left-color: #ff4d4f;
}
.result-card.error {
  border-left-color: #ff7a45;
}
.result-card.warning {
  border-left-color: #faad14;
}
.result-card.info {
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
  margin: 0 0 5px 0;
  font-size: 14px;
}

.result-description {
  margin: 0 0 10px 0;
  font-size: 13px;
  color: var(--sl-text-secondary);
}

.suggestions h6,
.related-logs h6 {
  margin: 10px 0 5px 0;
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.suggestions ul {
  margin: 0;
  padding-left: 20px;
  font-size: 13px;
}

.related-logs pre {
  margin: 5px 0;
  padding: 8px;
  background: var(--sl-bg);
  border-radius: 4px;
  font-size: 11px;
  overflow-x: auto;
}

/* 命令助手 */
.command-input {
  display: flex;
  gap: 10px;
}

.command-input input {
  flex: 1;
  padding: 12px;
  border: 1px solid var(--sl-border);
  border-radius: 6px;
  background: var(--sl-input-bg);
  color: var(--sl-text);
  font-size: 14px;
}

.command-input button {
  padding: 12px 16px;
  border: none;
  border-radius: 6px;
  background: var(--sl-primary);
  color: white;
  cursor: pointer;
}

.command-input button:disabled {
  background: var(--sl-disabled);
  cursor: not-allowed;
}

.command-result {
  margin-top: 20px;
  padding: 15px;
  background: var(--sl-bg-secondary);
  border-radius: 6px;
}

.generated-command {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.generated-command code {
  flex: 1;
  padding: 12px;
  background: var(--sl-bg);
  border-radius: 4px;
  font-family: monospace;
  font-size: 14px;
}

.copy-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 8px 12px;
  border: 1px solid var(--sl-border);
  border-radius: 4px;
  background: transparent;
  color: var(--sl-text-secondary);
  font-size: 12px;
  cursor: pointer;
}

.copy-btn:hover {
  background: var(--sl-bg-secondary);
}

.command-description {
  margin: 0 0 15px 0;
  font-size: 13px;
  color: var(--sl-text-secondary);
}

.parameters h6,
.examples h6 {
  margin: 15px 0 8px 0;
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.param-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 0;
  font-size: 13px;
}

.param-name {
  font-weight: 500;
  color: var(--sl-primary);
}

.param-type {
  padding: 2px 6px;
  background: var(--sl-bg);
  border-radius: 3px;
  font-size: 11px;
}

.param-required {
  padding: 2px 6px;
  background: #ff7a4520;
  color: #ff7a45;
  border-radius: 3px;
  font-size: 11px;
}

.param-desc {
  color: var(--sl-text-secondary);
}

.examples code {
  display: block;
  margin: 5px 0;
  padding: 8px;
  background: var(--sl-bg);
  border-radius: 4px;
  font-size: 12px;
}

.quick-commands {
  margin-top: 20px;
}

.quick-commands h6 {
  margin: 0 0 10px 0;
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.quick-command-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.quick-command-list button {
  padding: 6px 12px;
  border: 1px solid var(--sl-border);
  border-radius: 4px;
  background: transparent;
  color: var(--sl-text);
  font-size: 12px;
  cursor: pointer;
}

.quick-command-list button:hover {
  background: var(--sl-bg-secondary);
}

/* 配置优化 */
.hardware-form,
.content-form {
  margin-bottom: 15px;
}

.form-row {
  display: flex;
  gap: 15px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 5px;
  margin-bottom: 10px;
}

.form-group label {
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.form-group input,
.form-group textarea {
  padding: 8px 12px;
  border: 1px solid var(--sl-border);
  border-radius: 4px;
  background: var(--sl-input-bg);
  color: var(--sl-text);
  font-size: 14px;
}

.jvm-result,
.config-suggestions {
  margin-top: 20px;
  padding: 15px;
  background: var(--sl-bg-secondary);
  border-radius: 6px;
}

.memory-suggestion,
.gc-suggestion,
.jvm-args {
  margin-bottom: 15px;
}

.memory-suggestion h6,
.gc-suggestion h6,
.jvm-args h6 {
  margin: 0 0 8px 0;
  font-size: 12px;
}

.memory-suggestion p,
.gc-suggestion p {
  margin: 0 0 8px 0;
  font-size: 13px;
  color: var(--sl-text-secondary);
}

.memory-values {
  display: flex;
  gap: 15px;
  font-family: monospace;
  font-size: 13px;
}

.jvm-args pre {
  margin: 10px 0;
  padding: 12px;
  background: var(--sl-bg);
  border-radius: 4px;
  font-size: 12px;
  overflow-x: auto;
}

.suggestion-item {
  padding: 12px;
  margin-bottom: 10px;
  background: var(--sl-bg);
  border-radius: 6px;
  border-left: 3px solid var(--sl-border);
}

.suggestion-item.priority-5 {
  border-left-color: #ff4d4f;
}
.suggestion-item.priority-4 {
  border-left-color: #ff7a45;
}
.suggestion-item.priority-3 {
  border-left-color: #faad14;
}
.suggestion-item.priority-2 {
  border-left-color: #1890ff;
}
.suggestion-item.priority-1 {
  border-left-color: #52c41a;
}

.suggestion-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.config-key {
  font-weight: 500;
  font-family: monospace;
}

.priority {
  font-size: 11px;
  color: var(--sl-text-secondary);
}

.suggestion-values {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  font-size: 13px;
}

.current {
  color: var(--sl-text-secondary);
  text-decoration: line-through;
}

.arrow {
  width: 14px;
  height: 14px;
  color: var(--sl-text-secondary);
}

.suggested {
  color: var(--sl-success);
  font-weight: 500;
}

.suggestion-reason,
.suggestion-effect {
  margin: 0 0 5px 0;
  font-size: 12px;
  color: var(--sl-text-secondary);
}

/* 翻译 */
.swap-btn {
  padding: 8px;
  border: 1px solid var(--sl-border);
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
  align-self: flex-end;
}

.swap-btn:hover {
  background: var(--sl-bg-secondary);
}

.translate-result,
.generated-content {
  margin-top: 20px;
  padding: 15px;
  background: var(--sl-bg-secondary);
  border-radius: 6px;
}

.translated-text {
  margin: 0 0 10px 0;
  font-size: 14px;
  line-height: 1.6;
}

.translate-meta,
.content-meta {
  display: flex;
  gap: 15px;
  margin-bottom: 10px;
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.generated-content pre {
  margin: 10px 0;
  padding: 12px;
  background: var(--sl-bg);
  border-radius: 4px;
  font-size: 13px;
  white-space: pre-wrap;
  line-height: 1.6;
}
</style>
