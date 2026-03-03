/**
 * AI 功能 API
 * 
 * 提供 AI 功能的前端调用接口
 */

import { invoke } from '@tauri-apps/api/core';

// ==================== 类型定义 ====================

/** AI 配置 */
export interface AIConfig {
  enabled: boolean;
  provider: string;
  api_key?: string;
  base_url?: string;
  model: string;
  max_tokens: number;
  temperature: number;
  enable_cache: boolean;
  timeout_seconds: number;
}

/** AI 分析结果 */
export interface AIAnalysisResult {
  analysis_type: string;
  severity: 'critical' | 'error' | 'warning' | 'info';
  title: string;
  description: string;
  suggestions: string[];
  related_logs: string[];
  confidence: number;
}

/** AI 命令建议 */
export interface AICommandSuggestion {
  command: string;
  description: string;
  parameters: CommandParameter[];
  examples: string[];
  related_commands: string[];
}

/** 命令参数 */
export interface CommandParameter {
  name: string;
  param_type: string;
  required: boolean;
  default_value?: string;
  description: string;
}

/** AI 配置建议 */
export interface AIConfigSuggestion {
  config_key: string;
  current_value: string;
  suggested_value: string;
  reason: string;
  expected_effect: string;
  priority: number;
}

/** AI 翻译结果 */
export interface AITranslationResult {
  original_text: string;
  source_language: string;
  translated_text: string;
  target_language: string;
  confidence: number;
}

/** AI 内容生成结果 */
export interface AIContentGeneration {
  content: string;
  content_type: string;
  style?: string;
  generated_at: string;
  confidence: number;
  suggestions: string[];
}

/** 提供商信息 */
export interface ProviderInfo {
  name: string;
  display_name: string;
  models: string[];
}

/** 语言信息 */
export interface LanguageInfo {
  code: string;
  display_name: string;
}

/** JVM 分析结果 */
export interface JVMAnalysisResult {
  current_args: string[];
  suggested_args: string[];
  memory_suggestion: MemorySuggestion;
  gc_suggestion: GCSuggestion;
  explanation: string;
}

/** 内存建议 */
export interface MemorySuggestion {
  min_heap_mb: number;
  max_heap_mb: number;
  new_size_mb?: number;
  explanation: string;
}

/** GC 建议 */
export interface GCSuggestion {
  gc_type: string;
  gc_args: string[];
  explanation: string;
}

/** 日志模式信息 */
export interface LogPatternInfo {
  name: string;
  severity: string;
  description: string;
  solutions: string[];
}

/** 硬件信息 */
export interface HardwareInfo {
  cpu_cores: number;
  total_memory_gb: number;
  available_memory_gb: number;
  is_ssd: boolean;
  os: string;
}

// ==================== 配置管理 ====================

/**
 * 获取 AI 配置
 */
export async function getAIConfig(): Promise<AIConfig> {
  return invoke<AIConfig>('get_ai_config');
}

/**
 * 更新 AI 配置
 */
export async function updateAIConfig(config: AIConfig): Promise<void> {
  return invoke('update_ai_config', { config });
}

/**
 * 检查 AI 是否可用
 */
export async function checkAIAvailable(): Promise<boolean> {
  return invoke<boolean>('check_ai_available');
}

/**
 * 获取支持的提供商列表
 */
export async function getAIProviders(): Promise<ProviderInfo[]> {
  return invoke<ProviderInfo[]>('get_ai_providers');
}

// ==================== 日志分析 ====================

/** 分析日志选项 */
export interface AnalyzeLogsOptions {
  logs: string[];
  analysis_type?: 'error' | 'performance' | 'security' | 'plugin' | 'full';
  include_suggestions?: boolean;
  max_lines?: number;
}

/**
 * 分析日志
 */
export async function analyzeLogs(options: AnalyzeLogsOptions): Promise<AIAnalysisResult[]> {
  return invoke<AIAnalysisResult[]>('analyze_logs', {
    request: {
      logs: options.logs,
      analysis_type: options.analysis_type || 'full',
      include_suggestions: options.include_suggestions ?? true,
      max_lines: options.max_lines,
    },
  });
}

/**
 * 解释日志行
 */
export async function explainLogLine(logLine: string): Promise<string | null> {
  return invoke<string | null>('explain_log_line', { logLine });
}

/**
 * 获取问题解决方案
 */
export async function getSolution(problem: string): Promise<string | null> {
  return invoke<string | null>('get_solution', { problem });
}

/**
 * 获取预定义的日志模式
 */
export async function getLogPatterns(): Promise<LogPatternInfo[]> {
  return invoke<LogPatternInfo[]>('get_log_patterns');
}

// ==================== 命令助手 ====================

/** 生成命令选项 */
export interface GenerateCommandOptions {
  natural_language: string;
  command_type?: string;
  include_explanation?: boolean;
  mc_version?: string;
  server_type?: string;
}

/**
 * 生成命令
 */
export async function generateCommand(options: GenerateCommandOptions): Promise<AICommandSuggestion | null> {
  return invoke<AICommandSuggestion | null>('generate_command', {
    request: {
      natural_language: options.natural_language,
      command_type: options.command_type,
      include_explanation: options.include_explanation ?? true,
      mc_version: options.mc_version,
      server_type: options.server_type,
    },
  });
}

/**
 * 解释命令
 */
export async function explainCommand(command: string): Promise<string | null> {
  return invoke<string | null>('explain_command', { command });
}

/**
 * 获取命令建议
 */
export async function getCommandSuggestions(partialCommand: string): Promise<AICommandSuggestion[]> {
  return invoke<AICommandSuggestion[]>('get_command_suggestions', { partialCommand });
}

// ==================== 配置优化 ====================

/** 分析配置选项 */
export interface AnalyzeConfigOptions {
  config_content: string;
  config_type: string;
  analysis_type?: 'jvm' | 'server' | 'performance' | 'security' | 'full';
  hardware_info?: HardwareInfo;
  expected_players?: number;
  server_type?: string;
  mc_version?: string;
}

/**
 * 分析配置
 */
export async function analyzeConfig(options: AnalyzeConfigOptions): Promise<AIConfigSuggestion[]> {
  return invoke<AIConfigSuggestion[]>('analyze_config', {
    request: {
      config_content: options.config_content,
      config_type: options.config_type,
      analysis_type: options.analysis_type || 'full',
      hardware_info: options.hardware_info,
      expected_players: options.expected_players,
      server_type: options.server_type,
      mc_version: options.mc_version,
    },
  });
}

/** 分析 JVM 选项 */
export interface AnalyzeJVMOptions {
  current_args: string[];
  hardware: HardwareInfo;
  expected_players: number;
}

/**
 * 分析 JVM 参数
 */
export async function analyzeJVM(options: AnalyzeJVMOptions): Promise<JVMAnalysisResult> {
  return invoke<JVMAnalysisResult>('analyze_jvm', { request: options });
}

/**
 * 生成启动脚本
 */
export async function generateStartupScript(jvmArgs: string[], serverJar: string): Promise<string> {
  return invoke<string>('generate_startup_script', { jvmArgs, serverJar });
}

// ==================== 翻译 ====================

/** 翻译选项 */
export interface TranslateOptions {
  text: string;
  source_language: string;
  target_language: string;
  context?: 'chat' | 'announcement' | 'rules' | 'help' | 'item' | 'ui' | 'general';
  preserve_formatting?: boolean;
}

/**
 * 翻译文本
 */
export async function translateText(options: TranslateOptions): Promise<AITranslationResult> {
  return invoke<AITranslationResult>('translate_text', {
    request: {
      text: options.text,
      source_language: options.source_language,
      target_language: options.target_language,
      context: options.context,
      preserve_formatting: options.preserve_formatting ?? true,
    },
  });
}

/**
 * 检测语言
 */
export async function detectLanguage(text: string): Promise<string | null> {
  return invoke<string | null>('detect_language', { text });
}

/**
 * 获取支持的语言列表
 */
export async function getSupportedLanguages(): Promise<LanguageInfo[]> {
  return invoke<LanguageInfo[]>('get_supported_languages');
}

/**
 * 清空翻译缓存
 */
export async function clearTranslationCache(): Promise<void> {
  return invoke('clear_translation_cache');
}

// ==================== 内容生成 ====================

/** 生成内容选项 */
export interface GenerateContentOptions {
  content_type: 'announcement' | 'rules' | 'event' | 'welcome' | 'help' | 'news' | 'advertisement';
  server_name: string;
  server_features: string[];
  target_audience?: string;
  language?: string;
  style?: 'formal' | 'casual' | 'friendly' | 'professional' | 'humorous' | 'dramatic';
  length?: 'short' | 'medium' | 'long' | 'detailed';
  extra_requirements?: string;
}

/**
 * 生成内容
 */
export async function generateContent(options: GenerateContentOptions): Promise<AIContentGeneration> {
  return invoke<AIContentGeneration>('generate_content', { request: options });
}

/**
 * 生成公告
 */
export async function generateAnnouncement(
  serverName: string,
  topic: string,
  details?: string
): Promise<string | null> {
  return invoke<string | null>('generate_announcement', {
    request: {
      server_name: serverName,
      topic,
      details,
    },
  });
}

/**
 * 生成规则
 */
export async function generateRules(serverName: string, serverType: string): Promise<string | null> {
  return invoke<string | null>('generate_rules', { serverName, serverType });
}

/**
 * 生成欢迎消息
 */
export async function generateWelcome(serverName: string, playerName?: string): Promise<string | null> {
  return invoke<string | null>('generate_welcome', { serverName, playerName });
}

// ==================== 工具函数 ====================

/**
 * 格式化严重程度显示
 */
export function formatSeverity(severity: string): { text: string; color: string } {
  switch (severity) {
    case 'critical':
      return { text: '严重', color: '#ff4d4f' };
    case 'error':
      return { text: '错误', color: '#ff7a45' };
    case 'warning':
      return { text: '警告', color: '#faad14' };
    case 'info':
      return { text: '信息', color: '#1890ff' };
    default:
      return { text: severity, color: '#666' };
  }
}

/**
 * 格式化置信度显示
 */
export function formatConfidence(confidence: number): { text: string; level: 'high' | 'medium' | 'low' } {
  if (confidence >= 0.8) {
    return { text: '高', level: 'high' };
  } else if (confidence >= 0.5) {
    return { text: '中', level: 'medium' };
  } else {
    return { text: '低', level: 'low' };
  }
}

/**
 * 验证 API 密钥格式
 */
export function validateAPIKey(provider: string, key: string): boolean {
  if (!key || key.trim().length === 0) {
    return false;
  }

  switch (provider) {
    case 'openai':
      return key.startsWith('sk-');
    case 'anthropic':
      return key.startsWith('sk-ant-');
    default:
      return key.length >= 10;
  }
}

/**
 * 获取默认模型
 */
export function getDefaultModel(provider: string): string {
  switch (provider) {
    case 'openai':
      return 'gpt-4o-mini';
    case 'anthropic':
      return 'claude-3-5-haiku-20241022';
    case 'local':
      return 'llama3.2';
    default:
      return '';
  }
}

/**
 * 获取提供商显示名称
 */
export function getProviderDisplayName(provider: string): string {
  switch (provider) {
    case 'openai':
      return 'OpenAI';
    case 'anthropic':
      return 'Anthropic (Claude)';
    case 'local':
      return '本地模型 (Ollama)';
    default:
      return provider;
  }
}

// 默认导出
export default {
  // 配置管理
  getAIConfig,
  updateAIConfig,
  checkAIAvailable,
  getAIProviders,
  
  // 日志分析
  analyzeLogs,
  explainLogLine,
  getSolution,
  getLogPatterns,
  
  // 命令助手
  generateCommand,
  explainCommand,
  getCommandSuggestions,
  
  // 配置优化
  analyzeConfig,
  analyzeJVM,
  generateStartupScript,
  
  // 翻译
  translateText,
  detectLanguage,
  getSupportedLanguages,
  clearTranslationCache,
  
  // 内容生成
  generateContent,
  generateAnnouncement,
  generateRules,
  generateWelcome,
  
  // 工具函数
  formatSeverity,
  formatConfidence,
  validateAPIKey,
  getDefaultModel,
  getProviderDisplayName,
};
