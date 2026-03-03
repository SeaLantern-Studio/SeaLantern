//! AI 服务模块
//! 
//! 提供各种 AI 功能服务，包括日志分析、命令转换、配置优化、翻译和内容生成

pub mod ai_provider;
pub mod ai_console;
pub mod ai_command;
pub mod ai_config;
pub mod ai_translator;
pub mod ai_content;
pub mod ai_anticheat;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 重导出主要类型
pub use ai_provider::{AIProvider, get_provider, get_supported_providers, AIProviderType};
pub use ai_console::{AIConsoleService, LogAnalysisOptions, AnalysisType, LogPattern, get_predefined_patterns};
pub use ai_command::{AICommandService, CommandGenerationOptions, CommandType};
pub use ai_config::{AIConfigService, ConfigAnalysisOptions, ConfigAnalysisType, HardwareInfo, JVMAnalysis};
pub use ai_translator::{AITranslatorService, TranslationOptions, TranslationContext, Language, get_supported_languages};
pub use ai_content::{AIContentService, ContentType, ContentGenerationOptions, ContentStyle, ContentLength};
pub use ai_anticheat::{
    MCAntiCheatService, DetectionType, DetectionCategory, Severity,
    DetectionRecord, DetectionStatus, DetectionRule, PunishmentAction, PunishmentConfig,
    PlayerProfile, PlayerStatus, AntiCheatStatistics,
    PlayerBehaviorData, ModInfo,
};

/// AI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// 是否启用 AI 功能
    pub enabled: bool,
    /// 提供商类型
    pub provider: String,
    /// API 密钥
    pub api_key: Option<String>,
    /// 基础 URL（可选，用于自定义端点）
    pub base_url: Option<String>,
    /// 模型名称
    pub model: String,
    /// 最大 token 数
    pub max_tokens: u32,
    /// 温度参数
    pub temperature: f32,
    /// 是否启用缓存
    pub enable_cache: bool,
    /// 超时时间（秒）
    pub timeout_seconds: u32,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: "openai".to_string(),
            api_key: None,
            base_url: None,
            model: "gpt-4o-mini".to_string(),
            max_tokens: 2048,
            temperature: 0.7,
            enable_cache: true,
            timeout_seconds: 30,
        }
    }
}

/// AI 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMessage {
    /// 角色 (system, user, assistant)
    pub role: String,
    /// 内容
    pub content: String,
    /// 时间戳
    pub timestamp: i64,
}

/// AI 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    /// 是否成功
    pub success: bool,
    /// 响应内容
    pub content: Option<String>,
    /// 错误信息
    pub error: Option<String>,
    /// 使用的 token 数
    pub tokens_used: Option<u32>,
    /// 响应时间（毫秒）
    pub response_time_ms: u64,
}

/// AI 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisResult {
    /// 分析类型
    pub analysis_type: String,
    /// 严重程度 (critical, error, warning, info)
    pub severity: String,
    /// 标题
    pub title: String,
    /// 描述
    pub description: String,
    /// 建议列表
    pub suggestions: Vec<String>,
    /// 相关日志
    pub related_logs: Vec<String>,
    /// 置信度 (0.0 - 1.0)
    pub confidence: f32,
}

/// AI 命令建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICommandSuggestion {
    /// 命令
    pub command: String,
    /// 描述
    pub description: String,
    /// 参数列表
    pub parameters: Vec<CommandParameter>,
    /// 示例
    pub examples: Vec<String>,
    /// 相关命令
    pub related_commands: Vec<String>,
}

/// 命令参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandParameter {
    /// 参数名
    pub name: String,
    /// 参数类型
    pub param_type: String,
    /// 是否必需
    pub required: bool,
    /// 默认值
    pub default_value: Option<String>,
    /// 描述
    pub description: String,
}

/// AI 配置建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfigSuggestion {
    /// 配置键
    pub config_key: String,
    /// 当前值
    pub current_value: String,
    /// 建议值
    pub suggested_value: String,
    /// 原因
    pub reason: String,
    /// 预期效果
    pub expected_effect: String,
    /// 优先级 (1-5, 5最高)
    pub priority: u32,
}

/// AI 翻译结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AITranslationResult {
    /// 原文
    pub original_text: String,
    /// 源语言
    pub source_language: String,
    /// 译文
    pub translated_text: String,
    /// 目标语言
    pub target_language: String,
    /// 置信度
    pub confidence: f32,
}

/// AI 内容生成结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIContentGeneration {
    /// 生成的内容
    pub content: String,
    /// 内容类型
    pub content_type: String,
    /// 风格
    pub style: Option<String>,
    /// 生成时间
    pub generated_at: String,
    /// 置信度
    pub confidence: f32,
    /// 改进建议
    pub suggestions: Vec<String>,
}

/// AI 服务统一入口
pub struct AIService {
    /// 配置
    config: AIConfig,
    /// 控制台服务
    console_service: AIConsoleService,
    /// 命令服务
    command_service: AICommandService,
    /// 配置优化服务
    config_service: AIConfigService,
    /// 翻译服务
    translator_service: AITranslatorService,
    /// 内容生成服务
    content_service: AIContentService,
}

impl AIService {
    /// 创建新的 AI 服务
    pub fn new(config: AIConfig) -> Self {
        Self {
            config,
            console_service: AIConsoleService::new(),
            command_service: AICommandService::new(),
            config_service: AIConfigService::new(),
            translator_service: AITranslatorService::new(),
            content_service: AIContentService::new(),
        }
    }

    /// 获取配置
    pub fn get_config(&self) -> &AIConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: AIConfig) {
        self.config = config;
    }

    /// 分析日志
    pub async fn analyze_logs(
        &self,
        logs: &[String],
        options: &LogAnalysisOptions,
    ) -> Vec<AIAnalysisResult> {
        self.console_service.analyze_logs(logs, options, &self.config).await
    }

    /// 解释日志行
    pub async fn explain_log(&self, log_line: &str) -> Option<String> {
        self.console_service.explain_log_line(log_line, &self.config).await
    }

    /// 生成命令
    pub async fn generate_command(
        &self,
        natural_language: &str,
        options: &CommandGenerationOptions,
    ) -> Option<AICommandSuggestion> {
        self.command_service.generate_command(natural_language, options, &self.config).await
    }

    /// 解释命令
    pub async fn explain_command(&self, command: &str) -> Option<String> {
        self.command_service.explain_command(command, &self.config).await
    }

    /// 获取命令建议
    pub async fn get_command_suggestions(&self, partial: &str) -> Vec<AICommandSuggestion> {
        self.command_service.get_command_suggestions(partial, &self.config).await
    }

    /// 分析配置
    pub async fn analyze_config(
        &self,
        config_content: &str,
        config_type: &str,
        options: &ConfigAnalysisOptions,
    ) -> Vec<AIConfigSuggestion> {
        self.config_service.analyze_config(config_content, config_type, options, &self.config).await
    }

    /// 分析 JVM 参数
    pub async fn analyze_jvm(
        &self,
        current_args: &[String],
        hardware: &HardwareInfo,
        expected_players: u32,
    ) -> JVMAnalysis {
        self.config_service.analyze_jvm(current_args, hardware, expected_players, &self.config).await
    }

    /// 翻译文本
    pub async fn translate(
        &mut self,
        text: &str,
        options: &TranslationOptions,
    ) -> AITranslationResult {
        self.translator_service.translate(text, options, &self.config).await
    }

    /// 检测语言
    pub async fn detect_language(&self, text: &str) -> Option<Language> {
        self.translator_service.detect_language(text, &self.config).await
    }

    /// 生成内容
    pub async fn generate_content(
        &self,
        options: &ContentGenerationOptions,
    ) -> AIContentGeneration {
        self.content_service.generate_content(options, &self.config).await
    }

    /// 生成服务器公告
    pub async fn generate_announcement(
        &self,
        server_name: &str,
        topic: &str,
        details: Option<&str>,
    ) -> Option<String> {
        self.content_service.generate_announcement(server_name, topic, details, &self.config).await
    }

    /// 生成服务器规则
    pub async fn generate_rules(
        &self,
        server_name: &str,
        server_type: &str,
    ) -> Option<String> {
        self.content_service.generate_rules(server_name, server_type, &self.config).await
    }

    /// 生成欢迎消息
    pub async fn generate_welcome(
        &self,
        server_name: &str,
        player_name: Option<&str>,
    ) -> Option<String> {
        self.content_service.generate_welcome(server_name, player_name, &self.config).await
    }

    /// 获取问题解决方案
    pub async fn get_solution(&self, problem: &str) -> Option<String> {
        self.console_service.get_solution(problem, &self.config).await
    }
    /// 清空翻译缓存
    pub fn clear_translation_cache(&mut self) {
        self.translator_service.clear_cache();
    }

    /// 检查 AI 是否可用
    pub fn is_available(&self) -> bool {
        self.config.enabled && self.config.api_key.is_some()
    }
}

/// 创建默认 AI 服务
pub fn create_default_service() -> AIService {
    AIService::new(AIConfig::default())
}

/// 创建带配置的 AI 服务
pub fn create_service_with_config(config: AIConfig) -> AIService {
    AIService::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_config_default() {
        let config = AIConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-4o-mini");
    }

    #[test]
    fn test_ai_service_creation() {
        let service = create_default_service();
        assert!(!service.is_available());
    }

    #[test]
    fn test_ai_message() {
        let message = AIMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
            timestamp: 1234567890,
        };
        assert_eq!(message.role, "user");
    }

    #[test]
    fn test_ai_response() {
        let response = AIResponse {
            success: true,
            content: Some("Response".to_string()),
            error: None,
            tokens_used: Some(100),
            response_time_ms: 500,
        };
        assert!(response.success);
    }

    #[test]
    fn test_ai_analysis_result() {
        let result = AIAnalysisResult {
            analysis_type: "test".to_string(),
            severity: "warning".to_string(),
            title: "Test Issue".to_string(),
            description: "Test description".to_string(),
            suggestions: vec!["Fix it".to_string()],
            related_logs: vec!["log line".to_string()],
            confidence: 0.9,
        };
        assert_eq!(result.severity, "warning");
    }
}
