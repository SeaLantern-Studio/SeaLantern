//! AI 功能 Tauri 命令
//! 
//! 提供 AI 功能的前端调用接口

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

use crate::services::ai::{
    AIConfig, AIAnalysisResult, AICommandSuggestion, AIConfigSuggestion, 
    AITranslationResult, AIContentGeneration, AIService,
    LogAnalysisOptions, AnalysisType,
    CommandGenerationOptions, CommandType,
    ConfigAnalysisOptions, ConfigAnalysisType, HardwareInfo,
    TranslationOptions, TranslationContext, Language,
    ContentGenerationOptions, ContentType, ContentStyle, ContentLength,
};

/// AI 服务状态
pub struct AIState {
    pub service: Mutex<AIService>,
}

impl AIState {
    pub fn new() -> Self {
        Self {
            service: Mutex::new(AIService::new(AIConfig::default())),
        }
    }

    pub fn with_config(config: AIConfig) -> Self {
        Self {
            service: Mutex::new(AIService::new(config)),
        }
    }
}

impl Default for AIState {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== 配置管理 ====================

/// 获取 AI 配置
#[tauri::command]
pub fn get_ai_config(state: State<AIState>) -> AIConfig {
    let service = state.service.lock().unwrap();
    service.get_config().clone()
}

/// 更新 AI 配置
#[tauri::command]
pub fn update_ai_config(state: State<AIState>, config: AIConfig) -> Result<(), String> {
    let mut service = state.service.lock().unwrap();
    service.update_config(config);
    Ok(())
}

/// 检查 AI 是否可用
#[tauri::command]
pub fn check_ai_available(state: State<AIState>) -> bool {
    let service = state.service.lock().unwrap();
    service.is_available()
}

/// 获取支持的提供商列表
#[tauri::command]
pub fn get_ai_providers() -> Vec<ProviderInfo> {
    use crate::services::ai::get_supported_providers;
    
    let providers = get_supported_providers();
    providers
        .into_iter()
        .map(|(name, models)| ProviderInfo {
            name: name.clone(),
            display_name: get_provider_display_name(&name),
            models,
        })
        .collect()
}

/// 提供商信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub display_name: String,
    pub models: Vec<String>,
}

fn get_provider_display_name(name: &str) -> String {
    match name {
        "openai" => "OpenAI",
        "anthropic" => "Anthropic (Claude)",
        "local" => "本地模型 (Ollama)",
        _ => name,
    }.to_string()
}

// ==================== 日志分析 ====================

/// 分析日志请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeLogsRequest {
    /// 日志内容
    pub logs: Vec<String>,
    /// 分析类型
    pub analysis_type: String,
    /// 是否包含建议
    pub include_suggestions: bool,
    /// 最大分析行数
    pub max_lines: Option<usize>,
}

/// 分析日志
#[tauri::command]
pub async fn analyze_logs(
    state: State<'_, AIState>,
    request: AnalyzeLogsRequest,
) -> Result<Vec<AIAnalysisResult>, String> {
    let analysis_type = match request.analysis_type.as_str() {
        "error" => AnalysisType::ErrorDetection,
        "performance" => AnalysisType::PerformanceAnalysis,
        "security" => AnalysisType::SecurityCheck,
        "plugin" => AnalysisType::PluginConflict,
        _ => AnalysisType::FullAnalysis,
    };

    let options = LogAnalysisOptions {
        analysis_type,
        time_range: None,
        include_suggestions: request.include_suggestions,
        max_lines: request.max_lines,
    };

    let service = state.service.lock().unwrap();
    let results = service.analyze_logs(&request.logs, &options).await;
    Ok(results)
}

/// 解释日志行
#[tauri::command]
pub async fn explain_log_line(
    state: State<'_, AIState>,
    log_line: String,
) -> Result<Option<String>, String> {
    let service = state.service.lock().unwrap();
    let result = service.explain_log(&log_line).await;
    Ok(result)
}

/// 获取问题解决方案
#[tauri::command]
pub async fn get_solution(
    state: State<'_, AIState>,
    problem: String,
) -> Result<Option<String>, String> {
    let service = state.service.lock().unwrap();
    let result = service.get_solution(&problem).await;
    Ok(result)
}

// ==================== 命令助手 ====================

/// 生成命令请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateCommandRequest {
    /// 自然语言描述
    pub natural_language: String,
    /// 命令类型
    pub command_type: Option<String>,
    /// 是否包含解释
    pub include_explanation: bool,
    /// Minecraft 版本
    pub mc_version: Option<String>,
    /// 服务器类型
    pub server_type: Option<String>,
}

/// 生成命令
#[tauri::command]
pub async fn generate_command(
    state: State<'_, AIState>,
    request: GenerateCommandRequest,
) -> Result<Option<AICommandSuggestion>, String> {
    let command_type = request.command_type.and_then(|t| match t.as_str() {
        "gamemode" => Some(CommandType::GameMode),
        "teleport" => Some(CommandType::Teleport),
        "give" => Some(CommandType::Give),
        "time" => Some(CommandType::Time),
        "weather" => Some(CommandType::Weather),
        "player" => Some(CommandType::PlayerManagement),
        "world" => Some(CommandType::WorldManagement),
        _ => None,
    });

    let options = CommandGenerationOptions {
        command_type,
        selector_type: None,
        include_explanation: request.include_explanation,
        include_examples: true,
        mc_version: request.mc_version,
        server_type: request.server_type,
    };

    let service = state.service.lock().unwrap();
    let result = service.generate_command(&request.natural_language, &options).await;
    Ok(result)
}

/// 解释命令
#[tauri::command]
pub async fn explain_command(
    state: State<'_, AIState>,
    command: String,
) -> Result<Option<String>, String> {
    let service = state.service.lock().unwrap();
    let result = service.explain_command(&command).await;
    Ok(result)
}

/// 获取命令建议
#[tauri::command]
pub async fn get_command_suggestions(
    state: State<'_, AIState>,
    partial_command: String,
) -> Result<Vec<AICommandSuggestion>, String> {
    let service = state.service.lock().unwrap();
    let results = service.get_command_suggestions(&partial_command).await;
    Ok(results)
}

// ==================== 配置优化 ====================

/// 分析配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeConfigRequest {
    /// 配置内容
    pub config_content: String,
    /// 配置类型
    pub config_type: String,
    /// 分析类型
    pub analysis_type: String,
    /// 硬件信息
    pub hardware_info: Option<HardwareInfoRequest>,
    /// 预期玩家数
    pub expected_players: Option<u32>,
    /// 服务器类型
    pub server_type: Option<String>,
    /// MC 版本
    pub mc_version: Option<String>,
}

/// 硬件信息请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfoRequest {
    pub cpu_cores: u32,
    pub total_memory_gb: f64,
    pub available_memory_gb: f64,
    pub is_ssd: bool,
    pub os: String,
}

/// 分析配置
#[tauri::command]
pub async fn analyze_config(
    state: State<'_, AIState>,
    request: AnalyzeConfigRequest,
) -> Result<Vec<AIConfigSuggestion>, String> {
    let analysis_type = match request.analysis_type.as_str() {
        "jvm" => ConfigAnalysisType::JVM,
        "server" => ConfigAnalysisType::ServerProperties,
        "performance" => ConfigAnalysisType::Performance,
        "security" => ConfigAnalysisType::Security,
        _ => ConfigAnalysisType::Full,
    };

    let hardware_info = request.hardware_info.map(|h| HardwareInfo {
        cpu_cores: h.cpu_cores,
        total_memory_gb: h.total_memory_gb,
        available_memory_gb: h.available_memory_gb,
        is_ssd: h.is_ssd,
        os: h.os,
    });

    let options = ConfigAnalysisOptions {
        analysis_type,
        hardware_info,
        expected_players: request.expected_players,
        server_type: request.server_type,
        mc_version: request.mc_version,
    };

    let service = state.service.lock().unwrap();
    let results = service.analyze_config(&request.config_content, &request.config_type, &options).await;
    Ok(results)
}

/// 分析 JVM 参数请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeJVMRequest {
    /// 当前 JVM 参数
    pub current_args: Vec<String>,
    /// 硬件信息
    pub hardware: HardwareInfoRequest,
    /// 预期玩家数
    pub expected_players: u32,
}

/// 分析 JVM 参数
#[tauri::command]
pub async fn analyze_jvm(
    state: State<'_, AIState>,
    request: AnalyzeJVMRequest,
) -> Result<JVMAnalysisResult, String> {
    let hardware = HardwareInfo {
        cpu_cores: request.hardware.cpu_cores,
        total_memory_gb: request.hardware.total_memory_gb,
        available_memory_gb: request.hardware.available_memory_gb,
        is_ssd: request.hardware.is_ssd,
        os: request.hardware.os,
    };

    let service = state.service.lock().unwrap();
    let analysis = service.analyze_jvm(&request.current_args, &hardware, request.expected_players).await;

    Ok(JVMAnalysisResult {
        current_args: analysis.current_args,
        suggested_args: analysis.suggested_args,
        memory_suggestion: MemorySuggestionResult {
            min_heap_mb: analysis.memory_suggestion.min_heap_mb,
            max_heap_mb: analysis.memory_suggestion.max_heap_mb,
            new_size_mb: analysis.memory_suggestion.new_size_mb,
            explanation: analysis.memory_suggestion.explanation,
        },
        gc_suggestion: GCSuggestionResult {
            gc_type: analysis.gc_suggestion.gc_type,
            gc_args: analysis.gc_suggestion.gc_args,
            explanation: analysis.gc_suggestion.explanation,
        },
        explanation: analysis.explanation,
    })
}

/// JVM 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JVMAnalysisResult {
    pub current_args: Vec<String>,
    pub suggested_args: Vec<String>,
    pub memory_suggestion: MemorySuggestionResult,
    pub gc_suggestion: GCSuggestionResult,
    pub explanation: String,
}

/// 内存建议结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySuggestionResult {
    pub min_heap_mb: u32,
    pub max_heap_mb: u32,
    pub new_size_mb: Option<u32>,
    pub explanation: String,
}

/// GC 建议结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCSuggestionResult {
    pub gc_type: String,
    pub gc_args: Vec<String>,
    pub explanation: String,
}

/// 生成启动脚本
#[tauri::command]
pub fn generate_startup_script(
    jvm_args: Vec<String>,
    server_jar: String,
) -> String {
    let jvm_args_str = jvm_args.join(" ");
    format!(
        r#"#!/bin/bash
# SeaLantern AI 生成的启动脚本

JVM_ARGS="{}"
SERVER_JAR="{}"

java $JVM_ARGS -jar $SERVER_JAR nogui
"#,
        jvm_args_str, server_jar
    )
}

// ==================== 翻译 ====================

/// 翻译请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateRequest {
    /// 待翻译文本
    pub text: String,
    /// 源语言
    pub source_language: String,
    /// 目标语言
    pub target_language: String,
    /// 翻译上下文
    pub context: Option<String>,
    /// 保留格式代码
    pub preserve_formatting: bool,
}

/// 翻译文本
#[tauri::command]
pub async fn translate_text(
    state: State<'_, AIState>,
    request: TranslateRequest,
) -> Result<AITranslationResult, String> {
    let source_language = Language::from_code(&request.source_language)
        .unwrap_or(Language::Auto);
    
    let target_language = Language::from_code(&request.target_language)
        .unwrap_or(Language::English);

    let context = request.context.and_then(|c| match c.as_str() {
        "chat" => Some(TranslationContext::Chat),
        "announcement" => Some(TranslationContext::Announcement),
        "rules" => Some(TranslationContext::Rules),
        "help" => Some(TranslationContext::Help),
        "item" => Some(TranslationContext::ItemDescription),
        "ui" => Some(TranslationContext::UIText),
        _ => Some(TranslationContext::General),
    });

    let options = TranslationOptions {
        source_language,
        target_language,
        context,
        preserve_formatting: request.preserve_formatting,
    };

    let mut service = state.service.lock().unwrap();
    let result = service.translate(&request.text, &options).await;
    Ok(result)
}

/// 检测语言
#[tauri::command]
pub async fn detect_language(
    state: State<'_, AIState>,
    text: String,
) -> Result<Option<String>, String> {
    let service = state.service.lock().unwrap();
    let result = service.detect_language(&text).await;
    Ok(result.map(|l| l.code().to_string()))
}

/// 获取支持的语言列表
#[tauri::command]
pub fn get_supported_languages() -> Vec<LanguageInfo> {
    use crate::services::ai::get_supported_languages;
    
    get_supported_languages()
        .into_iter()
        .map(|l| LanguageInfo {
            code: l.code().to_string(),
            display_name: l.display_name().to_string(),
        })
        .collect()
}

/// 语言信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub code: String,
    pub display_name: String,
}

// ==================== 内容生成 ====================

/// 生成内容请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateContentRequest {
    /// 内容类型
    pub content_type: String,
    /// 服务器名称
    pub server_name: String,
    /// 服务器特色
    pub server_features: Vec<String>,
    /// 目标受众
    pub target_audience: Option<String>,
    /// 语言
    pub language: Option<String>,
    /// 风格
    pub style: Option<String>,
    /// 长度
    pub length: Option<String>,
    /// 额外要求
    pub extra_requirements: Option<String>,
}

/// 生成内容
#[tauri::command]
pub async fn generate_content(
    state: State<'_, AIState>,
    request: GenerateContentRequest,
) -> Result<AIContentGeneration, String> {
    let content_type = match request.content_type.as_str() {
        "announcement" => ContentType::Announcement,
        "rules" => ContentType::ServerRules,
        "event" => ContentType::Event,
        "welcome" => ContentType::Welcome,
        "help" => ContentType::Help,
        "news" => ContentType::News,
        "advertisement" => ContentType::Advertisement,
        _ => ContentType::Announcement,
    };

    let style = request.style.and_then(|s| match s.as_str() {
        "formal" => Some(ContentStyle::Formal),
        "casual" => Some(ContentStyle::Casual),
        "friendly" => Some(ContentStyle::Friendly),
        "professional" => Some(ContentStyle::Professional),
        "humorous" => Some(ContentStyle::Humorous),
        "dramatic" => Some(ContentStyle::Dramatic),
        _ => None,
    });

    let length = request.length.and_then(|l| match l.as_str() {
        "short" => Some(ContentLength::Short),
        "medium" => Some(ContentLength::Medium),
        "long" => Some(ContentLength::Long),
        "detailed" => Some(ContentLength::Detailed),
        _ => None,
    });

    let options = ContentGenerationOptions {
        content_type,
        server_name: request.server_name,
        server_features: request.server_features,
        target_audience: request.target_audience,
        language: request.language,
        style,
        length,
        extra_requirements: request.extra_requirements,
    };

    let service = state.service.lock().unwrap();
    let result = service.generate_content(&options).await;
    Ok(result)
}

/// 生成公告请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateAnnouncementRequest {
    pub server_name: String,
    pub topic: String,
    pub details: Option<String>,
}

/// 生成公告
#[tauri::command]
pub async fn generate_announcement(
    state: State<'_, AIState>,
    request: GenerateAnnouncementRequest,
) -> Result<Option<String>, String> {
    let service = state.service.lock().unwrap();
    let result = service
        .generate_announcement(&request.server_name, &request.topic, request.details.as_deref())
        .await;
    Ok(result)
}

/// 生成规则
#[tauri::command]
pub async fn generate_rules(
    state: State<'_, AIState>,
    server_name: String,
    server_type: String,
) -> Result<Option<String>, String> {
    let service = state.service.lock().unwrap();
    let result = service.generate_rules(&server_name, &server_type).await;
    Ok(result)
}

/// 生成欢迎消息
#[tauri::command]
pub async fn generate_welcome(
    state: State<'_, AIState>,
    server_name: String,
    player_name: Option<String>,
) -> Result<Option<String>, String> {
    let service = state.service.lock().unwrap();
    let result = service.generate_welcome(&server_name, player_name.as_deref()).await;
    Ok(result)
}

// ==================== 工具函数 ====================

/// 清空翻译缓存
#[tauri::command]
pub fn clear_translation_cache(state: State<AIState>) -> Result<(), String> {
    let mut service = state.service.lock().unwrap();
    service.clear_translation_cache();
    Ok(())
}

/// 获取预定义的日志模式
#[tauri::command]
pub fn get_log_patterns() -> Vec<LogPatternInfo> {
    use crate::services::ai::get_predefined_patterns;
    
    get_predefined_patterns()
        .into_iter()
        .map(|p| LogPatternInfo {
            name: p.name,
            severity: p.severity,
            description: p.description,
            solutions: p.solutions,
        })
        .collect()
}

/// 日志模式信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPatternInfo {
    pub name: String,
    pub severity: String,
    pub description: String,
    pub solutions: Vec<String>,
}

// ==================== MCP 反作弊 ====================

use crate::services::ai::{
    MCAntiCheatService, DetectionType, DetectionCategory, Severity,
    DetectionRecord, DetectionStatus, DetectionRule, PunishmentAction, PunishmentConfig,
    PlayerProfile, PlayerStatus, AntiCheatStatistics, PlayerBehaviorData, ModInfo,
};

/// 反作弊服务状态
pub struct AntiCheatState {
    pub service: Mutex<MCAntiCheatService>,
}

impl AntiCheatState {
    pub fn new() -> Self {
        Self {
            service: Mutex::new(MCAntiCheatService::new()),
        }
    }
}

impl Default for AntiCheatState {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== 检测相关命令 ====================

/// 分析玩家行为
#[tauri::command]
pub async fn analyze_player_behavior(
    anticheat_state: State<'_, AntiCheatState>,
    ai_state: State<'_, AIState>,
    request: AnalyzePlayerBehaviorRequest,
) -> Result<Option<DetectionRecord>, String> {
    let ai_config = {
        let service = ai_state.service.lock().unwrap();
        service.get_config().clone()
    };

    let behavior_data = PlayerBehaviorData {
        server_id: request.server_id,
        move_speed: request.move_speed,
        is_flying: request.is_flying,
        has_flight_permission: request.has_flight_permission,
        clicks_per_second: request.clicks_per_second,
        attack_reach: request.attack_reach,
        blocks_broken_normal: request.blocks_broken_normal,
        blocks_broken_rare: request.blocks_broken_rare,
        targets_attacked: request.targets_attacked,
        no_fall_count: request.no_fall_count,
        no_knockback_count: request.no_knockback_count,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    let service = anticheat_state.service.lock().unwrap();
    let result = service.analyze_player_behavior(
        &request.player_name,
        &request.player_uuid,
        &behavior_data,
        &ai_config,
    ).await;

    Ok(result)
}

/// 玩家行为分析请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzePlayerBehaviorRequest {
    pub server_id: String,
    pub player_name: String,
    pub player_uuid: String,
    pub move_speed: f64,
    pub is_flying: bool,
    pub has_flight_permission: bool,
    pub clicks_per_second: u32,
    pub attack_reach: f64,
    pub blocks_broken_normal: u32,
    pub blocks_broken_rare: u32,
    pub targets_attacked: u32,
    pub no_fall_count: u32,
    pub no_knockback_count: u32,
}

/// 分析客户端模组
#[tauri::command]
pub async fn analyze_client_mods(
    anticheat_state: State<'_, AntiCheatState>,
    ai_state: State<'_, AIState>,
    request: AnalyzeClientModsRequest,
) -> Result<Vec<DetectionRecord>, String> {
    let ai_config = {
        let service = ai_state.service.lock().unwrap();
        service.get_config().clone()
    };

    let mods: Vec<ModInfo> = request.mods.into_iter().map(|m| ModInfo {
        mod_id: m.mod_id,
        name: m.name,
        version: m.version,
        server_id: request.server_id.clone(),
    }).collect();

    let service = anticheat_state.service.lock().unwrap();
    let results = service.analyze_client_mods(
        &request.player_name,
        &request.player_uuid,
        &mods,
        &ai_config,
    ).await;

    Ok(results)
}

/// 客户端模组分析请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeClientModsRequest {
    pub server_id: String,
    pub player_name: String,
    pub player_uuid: String,
    pub mods: Vec<ModInfoRequest>,
}

/// 模组信息请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfoRequest {
    pub mod_id: String,
    pub name: String,
    pub version: String,
}

/// 提交检测记录
#[tauri::command]
pub fn submit_detection(
    anticheat_state: State<AntiCheatState>,
    detection: DetectionRecord,
) -> Result<(), String> {
    let mut service = anticheat_state.service.lock().unwrap();
    service.add_detection(detection);
    Ok(())
}

/// 获取检测记录列表
#[tauri::command]
pub fn get_detections(
    anticheat_state: State<AntiCheatState>,
    filter: Option<DetectionFilter>,
) -> Result<Vec<DetectionRecord>, String> {
    let service = anticheat_state.service.lock().unwrap();
    let detections = service.get_detections();
    
    let filtered = if let Some(f) = filter {
        detections.into_iter().filter(|d| {
            let mut match_type = true;
            let mut match_severity = true;
            let mut match_status = true;
            let mut match_player = true;

            if let Some(ref t) = f.detection_type {
                match_type = format!("{:?}", d.detection_type).to_lowercase() == t.to_lowercase();
            }
            if let Some(ref s) = f.severity {
                match_severity = format!("{:?}", d.severity).to_lowercase() == s.to_lowercase();
            }
            if let Some(ref s) = f.status {
                match_status = format!("{:?}", d.status).to_lowercase() == s.to_lowercase();
            }
            if let Some(ref p) = f.player_name {
                match_player = d.player_name.to_lowercase().contains(&p.to_lowercase());
            }

            match_type && match_severity && match_status && match_player
        }).cloned().collect()
    } else {
        detections.into_iter().cloned().collect()
    };

    Ok(filtered)
}

/// 检测记录过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionFilter {
    pub detection_type: Option<String>,
    pub severity: Option<String>,
    pub status: Option<String>,
    pub player_name: Option<String>,
}

/// 确定处罚动作
#[tauri::command]
pub fn determine_punishment(
    anticheat_state: State<AntiCheatState>,
    detection_id: String,
) -> Result<PunishmentAction, String> {
    let service = anticheat_state.service.lock().unwrap();
    
    if let Some(detection) = service.get_detections().iter().find(|d| d.id == detection_id) {
        let action = service.determine_punishment(detection);
        return Ok(action);
    }

    Err("Detection not found".to_string())
}

/// 执行处罚
#[tauri::command]
pub fn execute_punishment(
    anticheat_state: State<AntiCheatState>,
    detection_id: String,
    action: PunishmentAction,
) -> Result<bool, String> {
    let mut service = anticheat_state.service.lock().unwrap();
    Ok(service.execute_punishment(&detection_id, action))
}

// ==================== 玩家档案 ====================

/// 获取玩家档案
#[tauri::command]
pub fn get_player_profile(
    anticheat_state: State<AntiCheatState>,
    player_uuid: String,
) -> Result<Option<PlayerProfile>, String> {
    let service = anticheat_state.service.lock().unwrap();
    Ok(service.get_player_profile(&player_uuid).cloned())
}

// ==================== 规则管理 ====================

/// 获取检测规则列表
#[tauri::command]
pub fn get_detection_rules(
    anticheat_state: State<AntiCheatState>,
) -> Result<Vec<DetectionRule>, String> {
    let service = anticheat_state.service.lock().unwrap();
    Ok(service.get_rules().into_iter().cloned().collect())
}

/// 保存检测规则
#[tauri::command]
pub fn save_detection_rule(
    anticheat_state: State<AntiCheatState>,
    rule: DetectionRule,
) -> Result<(), String> {
    let mut service = anticheat_state.service.lock().unwrap();
    service.upsert_rule(rule);
    Ok(())
}

/// 删除检测规则
#[tauri::command]
pub fn delete_detection_rule(
    anticheat_state: State<AntiCheatState>,
    rule_id: String,
) -> Result<bool, String> {
    let mut service = anticheat_state.service.lock().unwrap();
    Ok(service.delete_rule(&rule_id))
}

// ==================== 处罚配置 ====================

/// 获取处罚配置
#[tauri::command]
pub fn get_punishment_config(
    anticheat_state: State<AntiCheatState>,
) -> Result<PunishmentConfig, String> {
    let service = anticheat_state.service.lock().unwrap();
    Ok(service.get_punishment_config().clone())
}

/// 更新处罚配置
#[tauri::command]
pub fn update_punishment_config(
    anticheat_state: State<AntiCheatState>,
    config: PunishmentConfig,
) -> Result<(), String> {
    let mut service = anticheat_state.service.lock().unwrap();
    service.update_punishment_config(config);
    Ok(())
}

// ==================== 统计数据 ====================

/// 获取统计数据
#[tauri::command]
pub fn get_anticheat_statistics(
    anticheat_state: State<AntiCheatState>,
) -> Result<AntiCheatStatistics, String> {
    let service = anticheat_state.service.lock().unwrap();
    Ok(service.get_statistics().clone())
}

/// 重置每日统计
#[tauri::command]
pub fn reset_daily_statistics(
    anticheat_state: State<AntiCheatState>,
) -> Result<(), String> {
    let mut service = anticheat_state.service.lock().unwrap();
    service.reset_daily_stats();
    Ok(())
}

// ==================== 检测类型信息 ====================

/// 获取所有检测类型
#[tauri::command]
pub fn get_detection_types() -> Vec<DetectionTypeInfo> {
    vec![
        DetectionTypeInfo { id: "speed_hack".to_string(), name: "加速外挂".to_string(), category: "hack".to_string() },
        DetectionTypeInfo { id: "fly_hack".to_string(), name: "飞行外挂".to_string(), category: "hack".to_string() },
        DetectionTypeInfo { id: "kill_aura".to_string(), name: "杀戮光环".to_string(), category: "hack".to_string() },
        DetectionTypeInfo { id: "xray".to_string(), name: "透视外挂".to_string(), category: "hack".to_string() },
        DetectionTypeInfo { id: "auto_clicker".to_string(), name: "自动点击".to_string(), category: "hack".to_string() },
        DetectionTypeInfo { id: "reach_hack".to_string(), name: "攻击距离扩展".to_string(), category: "hack".to_string() },
        DetectionTypeInfo { id: "no_fall".to_string(), name: "无摔落伤害".to_string(), category: "hack".to_string() },
        DetectionTypeInfo { id: "no_knockback".to_string(), name: "无击退".to_string(), category: "hack".to_string() },
        DetectionTypeInfo { id: "scaffold".to_string(), name: "自动搭路".to_string(), category: "hack".to_string() },
        DetectionTypeInfo { id: "baritone".to_string(), name: "自动行走".to_string(), category: "hack".to_string() },
        DetectionTypeInfo { id: "illegal_mod".to_string(), name: "违规模组".to_string(), category: "mod".to_string() },
        DetectionTypeInfo { id: "dupe_exploit".to_string(), name: "复制漏洞利用".to_string(), category: "exploit".to_string() },
        DetectionTypeInfo { id: "abnormal_behavior".to_string(), name: "异常行为".to_string(), category: "behavior".to_string() },
    ]
}

/// 检测类型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionTypeInfo {
    pub id: String,
    pub name: String,
    pub category: String,
}
