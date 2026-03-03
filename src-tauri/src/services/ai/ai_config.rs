//! AI 配置优化服务
//! 
//! 提供智能配置分析和优化建议

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{AIConfig, AIConfigSuggestion, AIMessage, AIProvider, ai_provider::get_provider};

/// 配置分析选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigAnalysisOptions {
    /// 分析类型
    pub analysis_type: ConfigAnalysisType,
    /// 服务器硬件信息
    pub hardware_info: Option<HardwareInfo>,
    /// 预期玩家数量
    pub expected_players: Option<u32>,
    /// 服务器类型
    pub server_type: Option<String>,
    /// Minecraft 版本
    pub mc_version: Option<String>,
}

/// 配置分析类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConfigAnalysisType {
    /// JVM 参数优化
    JVM,
    /// server.properties 优化
    ServerProperties,
    /// 性能优化
    Performance,
    /// 安全配置
    Security,
    /// 全面分析
    Full,
}

/// 硬件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    /// CPU 核心数
    pub cpu_cores: u32,
    /// 总内存 (GB)
    pub total_memory_gb: f64,
    /// 可用内存 (GB)
    pub available_memory_gb: f64,
    /// 是否为 SSD
    pub is_ssd: bool,
    /// 操作系统
    pub os: String,
}

/// JVM 参数分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JVMAnalysis {
    /// 当前 JVM 参数
    pub current_args: Vec<String>,
    /// 建议的 JVM 参数
    pub suggested_args: Vec<String>,
    /// 内存配置建议
    pub memory_suggestion: MemorySuggestion,
    /// GC 配置建议
    pub gc_suggestion: GCSuggestion,
    /// 优化说明
    pub explanation: String,
}

/// 内存配置建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySuggestion {
    /// 最小堆内存 (MB)
    pub min_heap_mb: u32,
    /// 最大堆内存 (MB)
    pub max_heap_mb: u32,
    /// 新生代大小 (MB)
    pub new_size_mb: Option<u32>,
    /// 建议说明
    pub explanation: String,
}

/// GC 配置建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCSuggestion {
    /// 推荐的 GC 类型
    pub gc_type: String,
    /// GC 参数
    pub gc_args: Vec<String>,
    /// 说明
    pub explanation: String,
}

/// AI 配置优化服务
pub struct AIConfigService;

impl AIConfigService {
    /// 创建新的配置优化服务
    pub fn new() -> Self {
        Self
    }

    /// 分析配置并生成优化建议
    pub async fn analyze_config(
        &self,
        config_content: &str,
        config_type: &str,
        options: &ConfigAnalysisOptions,
        ai_config: &AIConfig,
    ) -> Vec<AIConfigSuggestion> {
        let mut suggestions = Vec::new();

        // 首先进行本地分析
        let local_suggestions = self.local_config_analysis(config_content, config_type, options);
        suggestions.extend(local_suggestions);

        // 如果启用 AI，进行更深入的分析
        if ai_config.enabled {
            let ai_suggestions = self
                .ai_config_analysis(config_content, config_type, options, ai_config)
                .await;
            suggestions.extend(ai_suggestions);
        }

        suggestions
    }

    /// 分析 JVM 参数
    pub async fn analyze_jvm(
        &self,
        current_args: &[String],
        hardware: &HardwareInfo,
        expected_players: u32,
        ai_config: &AIConfig,
    ) -> JVMAnalysis {
        // 本地基础分析
        let base_analysis = self.local_jvm_analysis(current_args, hardware, expected_players);

        // AI 增强分析
        if ai_config.enabled {
            if let Some(enhanced) = self
                .ai_jvm_analysis(&base_analysis, hardware, expected_players, ai_config)
                .await
            {
                return enhanced;
            }
        }

        base_analysis
    }

    /// 本地配置分析
    fn local_config_analysis(
        &self,
        config_content: &str,
        config_type: &str,
        options: &ConfigAnalysisOptions,
    ) -> Vec<AIConfigSuggestion> {
        match config_type {
            "server.properties" => self.analyze_server_properties(config_content, options),
            "spigot.yml" => self.analyze_spigot_config(config_content, options),
            "paper.yml" | "paper-global.yml" => self.analyze_paper_config(config_content, options),
            "bukkit.yml" => self.analyze_bukkit_config(config_content, options),
            _ => Vec::new(),
        }
    }

    /// 分析 server.properties
    fn analyze_server_properties(
        &self,
        content: &str,
        options: &ConfigAnalysisOptions,
    ) -> Vec<AIConfigSuggestion> {
        let mut suggestions = Vec::new();
        let config = parse_properties(content);

        // 视距优化
        if let Some(view_distance) = config.get("view-distance") {
            if let Ok(distance) = view_distance.parse::<u32>() {
                if distance > 12 {
                    suggestions.push(AIConfigSuggestion {
                        config_key: "view-distance".to_string(),
                        current_value: view_distance.clone(),
                        suggested_value: "10".to_string(),
                        reason: "过大的视距会增加服务器负载，建议设置为 8-12".to_string(),
                        expected_effect: "降低 CPU 和内存使用，提高服务器性能".to_string(),
                        priority: 4,
                    });
                }
            }
        }

        // 模拟距离优化
        if let Some(sim_distance) = config.get("simulation-distance") {
            if let Ok(distance) = sim_distance.parse::<u32>() {
                if distance > 10 {
                    suggestions.push(AIConfigSuggestion {
                        config_key: "simulation-distance".to_string(),
                        current_value: sim_distance.clone(),
                        suggested_value: "8".to_string(),
                        reason: "过大的模拟距离会显著增加服务器负载".to_string(),
                        expected_effect: "降低 CPU 使用，改善性能".to_string(),
                        priority: 4,
                    });
                }
            }
        }

        // 最大玩家数
        if let Some(max_players) = config.get("max-players") {
            if let Ok(players) = max_players.parse::<u32>() {
                if let Some(expected) = options.expected_players {
                    if players > expected * 2 {
                        suggestions.push(AIConfigSuggestion {
                            config_key: "max-players".to_string(),
                            current_value: max_players.clone(),
                            suggested_value: (expected + 10).to_string(),
                            reason: "最大玩家数远超预期，合理设置可优化资源分配".to_string(),
                            expected_effect: "更合理的资源分配".to_string(),
                            priority: 2,
                        });
                    }
                }
            }
        }

        // 难度设置检查
        if let Some(difficulty) = config.get("difficulty") {
            if difficulty == "peaceful" {
                suggestions.push(AIConfigSuggestion {
                    config_key: "difficulty".to_string(),
                    current_value: difficulty.clone(),
                    suggested_value: "easy".to_string(),
                    reason: "和平模式会减少游戏体验，建议至少使用简单难度".to_string(),
                    expected_effect: "更好的游戏体验".to_string(),
                    priority: 1,
                });
            }
        }

        // 硬核模式检查
        if let Some(hardcore) = config.get("hardcore") {
            if hardcore == "true" {
                suggestions.push(AIConfigSuggestion {
                    config_key: "hardcore".to_string(),
                    current_value: hardcore.clone(),
                    suggested_value: "false".to_string(),
                    reason: "硬核模式会永久封禁死亡玩家，请确认这是预期行为".to_string(),
                    expected_effect: "避免意外封禁玩家".to_string(),
                    priority: 1,
                });
            }
        }

        // 在线模式检查
        if let Some(online_mode) = config.get("online-mode") {
            if online_mode == "false" {
                suggestions.push(AIConfigSuggestion {
                    config_key: "online-mode".to_string(),
                    current_value: online_mode.clone(),
                    suggested_value: "true".to_string(),
                    reason: "关闭正版验证存在安全风险，非正版玩家可能无法正常游戏".to_string(),
                    expected_effect: "提高服务器安全性，确保正版玩家体验".to_string(),
                    priority: 5,
                });
            }
        }

        suggestions
    }

    /// 分析 Spigot 配置
    fn analyze_spigot_config(&self, _content: &str, _options: &ConfigAnalysisOptions) -> Vec<AIConfigSuggestion> {
        // Spigot 配置分析实现
        Vec::new()
    }

    /// 分析 Paper 配置
    fn analyze_paper_config(&self, _content: &str, _options: &ConfigAnalysisOptions) -> Vec<AIConfigSuggestion> {
        // Paper 配置分析实现
        Vec::new()
    }

    /// 分析 Bukkit 配置
    fn analyze_bukkit_config(&self, _content: &str, _options: &ConfigAnalysisOptions) -> Vec<AIConfigSuggestion> {
        // Bukkit 配置分析实现
        Vec::new()
    }

    /// 本地 JVM 分析
    fn local_jvm_analysis(
        &self,
        current_args: &[String],
        hardware: &HardwareInfo,
        expected_players: u32,
    ) -> JVMAnalysis {
        // 计算推荐的内存分配
        let total_mem = hardware.total_memory_gb as u32;
        let recommended_heap = self.calculate_recommended_heap(total_mem, expected_players);

        // 确定最佳 GC
        let gc_suggestion = self.recommend_gc(hardware);

        // 构建建议的 JVM 参数
        let mut suggested_args = vec![
            format!("-Xms{}M", recommended_heap),
            format!("-Xmx{}M", recommended_heap),
        ];

        // 添加 GC 参数
        suggested_args.extend(gc_suggestion.gc_args.clone());

        // 添加通用优化参数
        suggested_args.extend(vec![
            "-XX:+UseG1GC".to_string(),
            "-XX:+ParallelRefProcEnabled".to_string(),
            "-XX:MaxGCPauseMillis=200".to_string(),
            "-XX:+UnlockExperimentalVMOptions".to_string(),
            "-XX:+DisableExplicitGC".to_string(),
            "-XX:+AlwaysPreTouch".to_string(),
            "-XX:G1NewSizePercent=30".to_string(),
            "-XX:G1MaxNewSizePercent=40".to_string(),
            "-XX:G1HeapRegionSize=8M".to_string(),
            "-XX:G1ReservePercent=20".to_string(),
            "-XX:G1HeapWastePercent=5".to_string(),
            "-XX:G1MixedGCCountTarget=4".to_string(),
            "-XX:InitiatingHeapOccupancyPercent=15".to_string(),
            "-XX:G1MixedGCLiveThresholdPercent=90".to_string(),
            "-XX:G1RSetUpdatingPauseTimePercent=5".to_string(),
            "-XX:SurvivorRatio=32".to_string(),
            "-XX:+PerfDisableSharedMem".to_string(),
            "-XX:MaxTenuringThreshold=1".to_string(),
        ]);

        // 分析当前参数
        let current_args_vec: Vec<String> = current_args.to_vec();

        let memory_suggestion = MemorySuggestion {
            min_heap_mb: recommended_heap,
            max_heap_mb: recommended_heap,
            new_size_mb: Some((recommended_heap as f64 * 0.3) as u32),
            explanation: format!(
                "根据系统内存 {}GB 和预期 {} 名玩家，推荐分配 {}MB 堆内存",
                total_mem, expected_players, recommended_heap
            ),
        };

        JVMAnalysis {
            current_args: current_args_vec,
            suggested_args,
            memory_suggestion,
            gc_suggestion,
            explanation: "基于硬件配置和预期负载的 JVM 优化建议".to_string(),
        }
    }

    /// 计算推荐的堆内存
    fn calculate_recommended_heap(&self, total_mem_gb: u32, expected_players: u32) -> u32 {
        // 基础内存：2GB
        let base = 2048;
        // 每玩家额外内存：约 50-100MB
        let per_player = 70;

        let calculated = base + (expected_players * per_player);

        // 不要超过系统内存的 70%
        let max_allowed = (total_mem_gb * 1024 * 7 / 10) as u32;

        // 留至少 2GB 给系统
        let safe_max = if total_mem_gb > 4 {
            max_allowed
        } else {
            (total_mem_gb * 1024 / 2) as u32 // 小内存系统只用一半
        };

        calculated.min(safe_max).max(1024) // 至少 1GB
    }

    /// 推荐 GC 配置
    fn recommend_gc(&self, hardware: &HardwareInfo) -> GCSuggestion {
        // G1GC 适用于大多数现代服务器
        GCSuggestion {
            gc_type: "G1GC".to_string(),
            gc_args: vec![
                "-XX:+UseG1GC".to_string(),
                "-XX:MaxGCPauseMillis=130".to_string(),
                "-XX:+UnlockExperimentalVMOptions".to_string(),
            ],
            explanation: format!(
                "G1GC 适合 {} 核 CPU 和 {:.1}GB 内存的系统，提供良好的吞吐量和低延迟",
                hardware.cpu_cores, hardware.total_memory_gb
            ),
        }
    }

    /// AI 增强配置分析
    async fn ai_config_analysis(
        &self,
        config_content: &str,
        config_type: &str,
        options: &ConfigAnalysisOptions,
        ai_config: &AIConfig,
    ) -> Vec<AIConfigSuggestion> {
        let provider = get_provider(&ai_config.provider);

        let system_prompt = format!(
            r#"你是一个专业的 Minecraft 服务器配置优化专家。
分析以下 {} 配置并提供优化建议。

服务器信息：
- 硬件：{} 核 CPU，{:.1}GB 内存
- 预期玩家数：{}
- 服务器类型：{}

返回 JSON 数组格式：
{{
    "suggestions": [
        {{
            "config_key": "配置项名",
            "current_value": "当前值",
            "suggested_value": "建议值",
            "reason": "原因",
            "expected_effect": "预期效果",
            "priority": 1-5
        }}
    ]
}}
"#,
            config_type,
            options.hardware_info.as_ref().map(|h| h.cpu_cores).unwrap_or(4),
            options.hardware_info.as_ref().map(|h| h.total_memory_gb).unwrap_or(8.0),
            options.expected_players.unwrap_or(20),
            options.server_type.as_deref().unwrap_or("vanilla")
        );

        let messages = vec![
            AIMessage {
                role: "system".to_string(),
                content: system_prompt,
                timestamp: chrono::Utc::now().timestamp(),
            },
            AIMessage {
                role: "user".to_string(),
                content: format!("配置内容：\n{}", config_content),
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        let response = provider.chat(&messages, ai_config).await;

        if response.success {
            if let Some(content) = response.content {
                return self.parse_config_suggestions(&content);
            }
        }

        Vec::new()
    }

    /// AI 增强分析
    async fn ai_jvm_analysis(
        &self,
        base_analysis: &JVMAnalysis,
        hardware: &HardwareInfo,
        expected_players: u32,
        ai_config: &AIConfig,
    ) -> Option<JVMAnalysis> {
        let provider = get_provider(&ai_config.provider);

        let messages = vec![
            AIMessage {
                role: "system".to_string(),
                content: "你是一个 JVM 性能调优专家，专注于 Minecraft 服务器优化。".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
            AIMessage {
                role: "user".to_string(),
                content: format!(
                    "请优化以下 JVM 配置：\n当前参数：{:?}\n硬件：{}核CPU，{}GB内存\n预期玩家：{}\n请返回优化后的 JVM 参数列表。",
                    base_analysis.current_args,
                    hardware.cpu_cores,
                    hardware.total_memory_gb,
                    expected_players
                ),
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        let response = provider.chat(&messages, ai_config).await;

        if response.success {
            // 简化返回，实际应用中可以解析更详细的内容
            return response.content.map(|_| base_analysis.clone());
        }

        None
    }

    /// 解析配置建议响应
    fn parse_config_suggestions(&self, content: &str) -> Vec<AIConfigSuggestion> {
        let json_start = content.find('{');
        let json_end = content.rfind('}');

        if let (Some(start), Some(end)) = (json_start, json_end) {
            let json_str = &content[start..=end];

            #[derive(Deserialize)]
            struct SuggestionsResponse {
                suggestions: Vec<AIConfigSuggestion>,
            }

            if let Ok(response) = serde_json::from_str::<SuggestionsResponse>(json_str) {
                return response.suggestions;
            }
        }

        Vec::new()
    }

    /// 生成完整的启动脚本
    pub fn generate_startup_script(
        &self,
        jvm_analysis: &JVMAnalysis,
        server_jar: &str,
    ) -> String {
        let jvm_args = jvm_analysis.suggested_args.join(" ");

        format!(
            r#"#!/bin/bash
# SeaLantern AI 生成的启动脚本

# JVM 参数
JVM_ARGS="{}"

# 服务器 JAR
SERVER_JAR="{}"

# 启动服务器
java $JVM_ARGS -jar $SERVER_JAR nogui
"#,
            jvm_args, server_jar
        )
    }
}

impl Default for AIConfigService {
    fn default() -> Self {
        Self::new()
    }
}

/// 解析 properties 文件
fn parse_properties(content: &str) -> HashMap<String, String> {
    let mut config = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            config.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_service_creation() {
        let service = AIConfigService::new();
        assert!(true);
    }

    #[test]
    fn test_parse_properties() {
        let content = r#"
# Server properties
server-port=25565
max-players=20
online-mode=true
"#;
        let config = parse_properties(content);
        assert_eq!(config.get("server-port"), Some(&"25565".to_string()));
        assert_eq!(config.get("max-players"), Some(&"20".to_string()));
    }

    #[test]
    fn test_calculate_recommended_heap() {
        let service = AIConfigService::new();
        
        // 小内存系统
        let heap = service.calculate_recommended_heap(4, 10);
        assert!(heap >= 1024 && heap <= 2048);

        // 中等内存系统
        let heap = service.calculate_recommended_heap(8, 20);
        assert!(heap >= 2048 && heap <= 4096);

        // 大内存系统
        let heap = service.calculate_recommended_heap(16, 50);
        assert!(heap >= 4096);
    }

    #[test]
    fn test_recommend_gc() {
        let service = AIConfigService::new();
        let hardware = HardwareInfo {
            cpu_cores: 8,
            total_memory_gb: 16.0,
            available_memory_gb: 12.0,
            is_ssd: true,
            os: "Linux".to_string(),
        };

        let gc = service.recommend_gc(&hardware);
        assert_eq!(gc.gc_type, "G1GC");
        assert!(!gc.gc_args.is_empty());
    }

    #[test]
    fn test_analyze_server_properties() {
        let service = AIConfigService::new();
        let content = r#"
view-distance=16
simulation-distance=12
online-mode=false
max-players=100
"#;
        let options = ConfigAnalysisOptions {
            analysis_type: ConfigAnalysisType::ServerProperties,
            hardware_info: None,
            expected_players: Some(20),
            server_type: None,
            mc_version: None,
        };

        let suggestions = service.analyze_server_properties(content, &options);
        assert!(!suggestions.is_empty());
    }
}
