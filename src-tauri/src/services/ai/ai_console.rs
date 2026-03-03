//! AI 控制台助手服务
//! 
//! 提供智能日志分析功能，帮助识别服务器问题和提供解决方案

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{AIAnalysisResult, AIConfig, AIMessage, AIResponse, AIProvider, ai_provider::get_provider};

/// 日志分析选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAnalysisOptions {
    /// 分析类型
    pub analysis_type: AnalysisType,
    /// 时间范围 (秒)
    pub time_range: Option<u64>,
    /// 是否包含详细建议
    pub include_suggestions: bool,
    /// 最大分析行数
    pub max_lines: Option<usize>,
}

/// 分析类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisType {
    /// 错误检测
    ErrorDetection,
    /// 性能分析
    PerformanceAnalysis,
    /// 安全检查
    SecurityCheck,
    /// 插件冲突检测
    PluginConflict,
    /// 全面分析
    FullAnalysis,
}

/// 日志模式定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPattern {
    /// 模式名称
    pub name: String,
    /// 正则表达式
    pub pattern: String,
    /// 严重程度
    pub severity: String,
    /// 问题描述
    pub description: String,
    /// 解决方案
    pub solutions: Vec<String>,
}

/// 预定义的日志模式
pub fn get_predefined_patterns() -> Vec<LogPattern> {
    vec![
        // 内存问题
        LogPattern {
            name: "out_of_memory".to_string(),
            pattern: r"(?i)(out of memory|java\.lang\.OutOfMemoryError|heap space)".to_string(),
            severity: "critical".to_string(),
            description: "服务器内存不足".to_string(),
            solutions: vec![
                "增加 JVM 堆内存分配 (-Xmx 参数)".to_string(),
                "检查是否存在内存泄漏的插件".to_string(),
                "减少视距渲染距离".to_string(),
                "优化实体数量上限".to_string(),
            ],
        },
        // 崩溃错误
        LogPattern {
            name: "server_crash".to_string(),
            pattern: r"(?i)(crash|fatal|exception|stopped)".to_string(),
            severity: "critical".to_string(),
            description: "服务器崩溃或严重错误".to_string(),
            solutions: vec![
                "查看完整崩溃报告确定原因".to_string(),
                "检查最近添加的插件/mod".to_string(),
                "验证服务器版本兼容性".to_string(),
                "检查硬件资源是否充足".to_string(),
            ],
        },
        // 插件错误
        LogPattern {
            name: "plugin_error".to_string(),
            pattern: r"(?i)(plugin.*error|plugin.*exception|plugin.*failed)".to_string(),
            severity: "warning".to_string(),
            description: "插件运行错误".to_string(),
            solutions: vec![
                "检查插件是否为最新版本".to_string(),
                "验证插件与服务器版本兼容性".to_string(),
                "查看插件配置文件是否正确".to_string(),
                "尝试禁用问题插件进行排查".to_string(),
            ],
        },
        // 连接问题
        LogPattern {
            name: "connection_issue".to_string(),
            pattern: r"(?i)(connection.*lost|disconnect|timeout|refused)".to_string(),
            severity: "warning".to_string(),
            description: "网络连接问题".to_string(),
            solutions: vec![
                "检查服务器网络连接".to_string(),
                "验证防火墙设置".to_string(),
                "检查端口是否正确开放".to_string(),
                "查看服务器负载情况".to_string(),
            ],
        },
        // 性能警告
        LogPattern {
            name: "performance_warning".to_string(),
            pattern: r"(?i)(lag|slow|tick.*took|can't keep up)".to_string(),
            severity: "warning".to_string(),
            description: "服务器性能问题".to_string(),
            solutions: vec![
                "减少实体数量".to_string(),
                "优化红石机械".to_string(),
                "增加服务器内存".to_string(),
                "使用性能优化插件".to_string(),
                "检查硬件资源使用".to_string(),
            ],
        },
        // 线程问题
        LogPattern {
            name: "thread_issue".to_string(),
            pattern: r"(?i)(thread.*deadlock|thread.*blocked|concurrent)".to_string(),
            severity: "error".to_string(),
            description: "线程死锁或阻塞".to_string(),
            solutions: vec![
                "检查是否存在死锁的插件".to_string(),
                "重启服务器清除线程状态".to_string(),
                "减少高并发操作".to_string(),
            ],
        },
        // 权限问题
        LogPattern {
            name: "permission_denied".to_string(),
            pattern: r"(?i)(permission denied|access denied|unauthorized)".to_string(),
            severity: "warning".to_string(),
            description: "权限被拒绝".to_string(),
            solutions: vec![
                "检查文件/文件夹权限".to_string(),
                "确认运行用户权限".to_string(),
                "检查安全软件拦截".to_string(),
            ],
        },
        // 磁盘空间
        LogPattern {
            name: "disk_space".to_string(),
            pattern: r"(?i)(disk.*full|no space|disk.*space)".to_string(),
            severity: "critical".to_string(),
            description: "磁盘空间不足".to_string(),
            solutions: vec![
                "清理磁盘空间".to_string(),
                "删除不需要的备份文件".to_string(),
                "清理日志文件".to_string(),
                "移动服务器到更大磁盘".to_string(),
            ],
        },
    ]
}

/// AI 控制台助手服务
pub struct AIConsoleService {
    patterns: Vec<LogPattern>,
}

impl AIConsoleService {
    /// 创建新的控制台助手服务
    pub fn new() -> Self {
        Self {
            patterns: get_predefined_patterns(),
        }
    }

    /// 分析日志内容
    pub async fn analyze_logs(
        &self,
        logs: &[String],
        options: &LogAnalysisOptions,
        config: &AIConfig,
    ) -> Vec<AIAnalysisResult> {
        let mut results = Vec::new();

        // 首先进行本地模式匹配
        let local_results = self.local_pattern_match(logs);
        
        // 如果启用了 AI，使用 AI 进行更深入的分析
        if config.enabled {
            let ai_results = self.ai_analysis(logs, options, config).await;
            results.extend(local_results);
            results.extend(ai_results);
        } else {
            results = local_results;
        }

        // 根据选项过滤结果
        results.sort_by(|a, b| {
            let severity_order = |s: &str| match s {
                "critical" => 0,
                "error" => 1,
                "warning" => 2,
                _ => 3,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });

        results
    }

    /// 本地模式匹配分析
    fn local_pattern_match(&self, logs: &[String]) -> Vec<AIAnalysisResult> {
        let mut results = Vec::new();

        for pattern in &self.patterns {
            let re = match regex::Regex::new(&pattern.pattern) {
                Ok(re) => re,
                Err(_) => continue,
            };

            let matched_logs: Vec<String> = logs
                .iter()
                .filter(|log| re.is_match(log))
                .cloned()
                .collect();

            if !matched_logs.is_empty() {
                results.push(AIAnalysisResult {
                    analysis_type: "pattern_match".to_string(),
                    severity: pattern.severity.clone(),
                    title: pattern.name.replace("_", " ").to_title_case(),
                    description: pattern.description.clone(),
                    suggestions: if pattern.solutions.is_empty() {
                        vec!["请查看详细日志以获取更多信息".to_string()]
                    } else {
                        pattern.solutions.clone()
                    },
                    related_logs: matched_logs,
                    confidence: 0.9,
                });
            }
        }

        results
    }

    /// AI 深度分析
    async fn ai_analysis(
        &self,
        logs: &[String],
        options: &LogAnalysisOptions,
        config: &AIConfig,
    ) -> Vec<AIAnalysisResult> {
        let provider = get_provider(&config.provider);

        // 截取最近的日志
        let max_lines = options.max_lines.unwrap_or(100);
        let logs_to_analyze: Vec<&str> = logs.iter().rev().take(max_lines).map(|s| s.as_str()).collect();
        let logs_text = logs_to_analyze.join("\n");

        // 构建分析提示词
        let system_prompt = build_analysis_system_prompt(options);
        let user_message = format!(
            "请分析以下 Minecraft 服务器日志，识别问题并提供解决方案：\n\n{}\n\n请以 JSON 格式返回分析结果。",
            logs_text
        );

        let messages = vec![
            AIMessage {
                role: "system".to_string(),
                content: system_prompt,
                timestamp: chrono::Utc::now().timestamp(),
            },
            AIMessage {
                role: "user".to_string(),
                content: user_message,
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        let response = provider.chat(&messages, config).await;

        if response.success {
            if let Some(content) = response.content {
                return parse_ai_analysis_response(&content);
            }
        }

        Vec::new()
    }

    /// 获取特定问题的解决方案
    pub async fn get_solution(
        &self,
        problem_description: &str,
        config: &AIConfig,
    ) -> Option<String> {
        if !config.enabled {
            return None;
        }

        let provider = get_provider(&config.provider);

        let messages = vec![
            AIMessage {
                role: "system".to_string(),
                content: "你是一个 Minecraft 服务器专家，擅长诊断和解决服务器问题。请提供简洁明了的解决方案。".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
            AIMessage {
                role: "user".to_string(),
                content: format!("问题描述：{}\n\n请提供详细的解决方案。", problem_description),
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        let response = provider.chat(&messages, config).await;
        response.content
    }

    /// 解释日志行
    pub async fn explain_log_line(
        &self,
        log_line: &str,
        config: &AIConfig,
    ) -> Option<String> {
        if !config.enabled {
            return None;
        }

        let provider = get_provider(&config.provider);

        let messages = vec![
            AIMessage {
                role: "system".to_string(),
                content: "你是一个 Minecraft 服务器日志分析专家。请用简洁的语言解释日志内容。".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
            AIMessage {
                role: "user".to_string(),
                content: format!("请解释这行 Minecraft 服务器日志的含义：\n{}", log_line),
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        let response = provider.chat(&messages, config).await;
        response.content
    }
}

impl Default for AIConsoleService {
    fn default() -> Self {
        Self::new()
    }
}

/// 构建分析系统提示词
fn build_analysis_system_prompt(options: &LogAnalysisOptions) -> String {
    let analysis_focus = match options.analysis_type {
        AnalysisType::ErrorDetection => "重点识别错误和异常",
        AnalysisType::PerformanceAnalysis => "重点分析性能问题",
        AnalysisType::SecurityCheck => "重点检查安全隐患",
        AnalysisType::PluginConflict => "重点检测插件冲突",
        AnalysisType::FullAnalysis => "进行全面综合分析",
    };

    format!(
        r#"你是一个专业的 Minecraft 服务器日志分析专家。{}
        
请分析日志并返回 JSON 格式的结果，格式如下：
{{
    "results": [
        {{
            "severity": "critical|error|warning|info",
            "title": "问题标题",
            "description": "问题描述",
            "suggestions": ["建议1", "建议2"],
            "related_logs": ["相关日志行"],
            "confidence": 0.0-1.0
        }}
    ]
}}

分析要点：
1. 识别所有错误和警告
2. 分析问题根本原因
3. 提供可操作的解决方案
4. 标注置信度
"#,
        analysis_focus
    )
}

/// 解析 AI 分析响应
fn parse_ai_analysis_response(content: &str) -> Vec<AIAnalysisResult> {
    // 尝试提取 JSON
    let json_start = content.find('{');
    let json_end = content.rfind('}');
    
    if let (Some(start), Some(end)) = (json_start, json_end) {
        let json_str = &content[start..=end];
        
        #[derive(Deserialize)]
        struct AnalysisResponse {
            results: Vec<AIAnalysisResultRaw>,
        }
        
        #[derive(Deserialize)]
        struct AIAnalysisResultRaw {
            severity: String,
            title: String,
            description: String,
            suggestions: Vec<String>,
            related_logs: Option<Vec<String>>,
            confidence: Option<f32>,
        }
        
        if let Ok(response) = serde_json::from_str::<AnalysisResponse>(json_str) {
            return response
                .results
                .into_iter()
                .map(|r| AIAnalysisResult {
                    analysis_type: "ai_analysis".to_string(),
                    severity: r.severity,
                    title: r.title,
                    description: r.description,
                    suggestions: r.suggestions,
                    related_logs: r.related_logs.unwrap_or_default(),
                    confidence: r.confidence.unwrap_or(0.7),
                })
                .collect();
        }
    }

    Vec::new()
}

/// 字符串转标题大小写
trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for str {
    fn to_title_case(&self) -> String {
        self.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_service_creation() {
        let service = AIConsoleService::new();
        assert!(!service.patterns.is_empty());
    }

    #[test]
    fn test_local_pattern_match() {
        let service = AIConsoleService::new();
        let logs = vec![
            "[ERROR] java.lang.OutOfMemoryError: Java heap space".to_string(),
            "[INFO] Server started".to_string(),
        ];

        let results = service.local_pattern_match(&logs);
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.severity == "critical"));
    }

    #[test]
    fn test_title_case() {
        assert_eq!("out_of_memory".to_title_case(), "Out Of Memory");
        assert_eq!("server_crash".to_title_case(), "Server Crash");
    }

    #[test]
    fn test_predefined_patterns() {
        let patterns = get_predefined_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns.iter().any(|p| p.name == "out_of_memory"));
    }
}
