//! AI 命令助手服务
//!
//! 提供自然语言转 Minecraft 命令的功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    ai_provider::get_provider, AICommandSuggestion, AIConfig, AIMessage, AIProvider, AIResponse,
    CommandParameter,
};

/// 命令类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CommandType {
    /// 游戏模式
    GameMode,
    /// 传送
    Teleport,
    /// 物品给予
    Give,
    /// 时间控制
    Time,
    /// 天气控制
    Weather,
    /// 玩家管理
    PlayerManagement,
    /// 世界管理
    WorldManagement,
    /// 实体操作
    Entity,
    /// 计分板
    Scoreboard,
    /// 团队
    Team,
    /// Boss栏
    BossBar,
    /// 标题
    Title,
    /// 音效
    Sound,
    /// 粒子效果
    Particle,
    /// 函数
    Function,
    /// 执行条件
    Execute,
    /// 数据操作
    Data,
    /// 其他
    Other,
}

/// 命令生成选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandGenerationOptions {
    /// 命令类型
    pub command_type: Option<CommandType>,
    /// 目标选择器类型
    pub selector_type: Option<String>,
    /// 是否需要解释
    pub include_explanation: bool,
    /// 是否需要示例
    pub include_examples: bool,
    /// Minecraft 版本
    pub mc_version: Option<String>,
    /// 服务器类型 (vanilla, spigot, paper 等)
    pub server_type: Option<String>,
}

/// AI 命令助手服务
pub struct AICommandService;

impl AICommandService {
    /// 创建新的命令助手服务
    pub fn new() -> Self {
        Self
    }

    /// 从自然语言生成命令
    pub async fn generate_command(
        &self,
        natural_language: &str,
        options: &CommandGenerationOptions,
        config: &AIConfig,
    ) -> Option<AICommandSuggestion> {
        if !config.enabled {
            // 如果 AI 未启用，返回本地解析结果
            return self.local_command_parse(natural_language, options);
        }

        let provider = get_provider(&config.provider);

        let system_prompt = self.build_command_system_prompt(options);
        let user_message = format!(
            "请将以下自然语言转换为 Minecraft 命令：\n{}\n\n返回 JSON 格式结果。",
            natural_language
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
                return self.parse_command_response(&content, options);
            }
        }

        // AI 失败时回退到本地解析
        self.local_command_parse(natural_language, options)
    }

    /// 解释命令
    pub async fn explain_command(&self, command: &str, config: &AIConfig) -> Option<String> {
        if !config.enabled {
            return self.local_command_explain(command);
        }

        let provider = get_provider(&config.provider);

        let messages = vec![
            AIMessage {
                role: "system".to_string(),
                content: "你是一个 Minecraft 命令专家，擅长解释各种命令的作用和用法。".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
            AIMessage {
                role: "user".to_string(),
                content: format!("请解释以下 Minecraft 命令的作用：\n{}", command),
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        let response = provider.chat(&messages, config).await;
        response.content
    }

    /// 获取命令建议
    pub async fn get_command_suggestions(
        &self,
        partial_command: &str,
        config: &AIConfig,
    ) -> Vec<AICommandSuggestion> {
        if !config.enabled {
            return self.local_command_suggestions(partial_command);
        }

        let provider = get_provider(&config.provider);

        let messages = vec![
            AIMessage {
                role: "system".to_string(),
                content: "你是一个 Minecraft 命令助手，帮助用户补全命令。返回 JSON 数组格式。"
                    .to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
            AIMessage {
                role: "user".to_string(),
                content: format!(
                    "用户正在输入命令：{}\n请提供可能的命令补全建议。返回 JSON 数组格式。",
                    partial_command
                ),
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        let response = provider.chat(&messages, config).await;

        if response.success {
            if let Some(content) = response.content {
                return self.parse_suggestions_response(&content);
            }
        }

        self.local_command_suggestions(partial_command)
    }

    /// 构建命令系统提示词
    fn build_command_system_prompt(&self, options: &CommandGenerationOptions) -> String {
        let version_info = options.mc_version.as_deref().unwrap_or("最新版本");
        let server_info = options.server_type.as_deref().unwrap_or("原版");

        format!(
            r#"你是一个专业的 Minecraft 命令专家。
目标版本：{}
服务器类型：{}

请将自然语言转换为准确的 Minecraft 命令。返回 JSON 格式：
{{
    "command": "生成的命令",
    "description": "命令说明",
    "parameters": [
        {{
            "name": "参数名",
            "param_type": "类型",
            "required": true/false,
            "default_value": "默认值",
            "description": "说明"
        }}
    ],
    "examples": ["示例1", "示例2"],
    "related_commands": ["相关命令1", "相关命令2"]
}}

命令规则：
1. 使用正确的命令语法
2. 选择器使用 @p, @a, @e, @r, @s
3. 坐标使用 ~ 表示相对坐标，^ 表示局部坐标
4. NBT 数据格式正确
5. 考虑版本差异
"#,
            version_info, server_info
        )
    }

    /// 解析命令响应
    fn parse_command_response(
        &self,
        content: &str,
        options: &CommandGenerationOptions,
    ) -> Option<AICommandSuggestion> {
        let json_start = content.find('{');
        let json_end = content.rfind('}');

        if let (Some(start), Some(end)) = (json_start, json_end) {
            let json_str = &content[start..=end];

            #[derive(Deserialize)]
            struct CommandResponse {
                command: String,
                description: String,
                parameters: Option<Vec<CommandParameterRaw>>,
                examples: Option<Vec<String>>,
                related_commands: Option<Vec<String>>,
            }

            #[derive(Deserialize)]
            struct CommandParameterRaw {
                name: String,
                param_type: String,
                required: Option<bool>,
                default_value: Option<String>,
                description: String,
            }

            if let Ok(response) = serde_json::from_str::<CommandResponse>(json_str) {
                return Some(AICommandSuggestion {
                    command: response.command,
                    description: response.description,
                    parameters: response
                        .parameters
                        .unwrap_or_default()
                        .into_iter()
                        .map(|p| CommandParameter {
                            name: p.name,
                            param_type: p.param_type,
                            required: p.required.unwrap_or(false),
                            default_value: p.default_value,
                            description: p.description,
                        })
                        .collect(),
                    examples: response.examples.unwrap_or_default(),
                    related_commands: response.related_commands.unwrap_or_default(),
                });
            }
        }

        None
    }

    /// 解析建议响应
    fn parse_suggestions_response(&self, content: &str) -> Vec<AICommandSuggestion> {
        let json_start = content.find('[');
        let json_end = content.rfind(']');

        if let (Some(start), Some(end)) = (json_start, json_end) {
            let json_str = &content[start..=end];

            #[derive(Deserialize)]
            struct SuggestionRaw {
                command: String,
                description: String,
            }

            if let Ok(suggestions) = serde_json::from_str::<Vec<SuggestionRaw>>(json_str) {
                return suggestions
                    .into_iter()
                    .map(|s| AICommandSuggestion {
                        command: s.command,
                        description: s.description,
                        parameters: Vec::new(),
                        examples: Vec::new(),
                        related_commands: Vec::new(),
                    })
                    .collect();
            }
        }

        Vec::new()
    }

    /// 本地命令解析 (AI 未启用时的回退方案)
    fn local_command_parse(
        &self,
        natural_language: &str,
        _options: &CommandGenerationOptions,
    ) -> Option<AICommandSuggestion> {
        let patterns = get_command_patterns();
        let nl_lower = natural_language.to_lowercase();

        for pattern in patterns {
            for keyword in &pattern.keywords {
                if nl_lower.contains(keyword) {
                    // 简单的模式匹配
                    let command = self.apply_pattern(natural_language, &pattern);
                    return Some(AICommandSuggestion {
                        command,
                        description: pattern.description.clone(),
                        parameters: pattern.parameters.clone(),
                        examples: pattern.examples.clone(),
                        related_commands: pattern.related_commands.clone(),
                    });
                }
            }
        }

        None
    }

    /// 应用模式生成命令
    fn apply_pattern(&self, input: &str, pattern: &CommandPattern) -> String {
        // 简单的实现，实际应用中可以更复杂
        let mut command = pattern.template.clone();

        // 尝试提取数字
        if let Some(num) = input.split_whitespace().find_map(|w| w.parse::<i32>().ok()) {
            command = command.replace("{number}", &num.to_string());
        }

        // 尝试提取玩家名
        let words: Vec<&str> = input.split_whitespace().collect();
        for word in &words {
            if word.len() > 2 && !word.parse::<i32>().is_ok() {
                command = command.replace("{player}", word);
                break;
            }
        }

        command
    }

    /// 本地命令解释
    fn local_command_explain(&self, command: &str) -> Option<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let command_name = parts[0].trim_start_matches('/');
        let explanations = get_command_explanations();

        explanations.get(command_name).cloned()
    }

    /// 本地命令建议
    fn local_command_suggestions(&self, partial: &str) -> Vec<AICommandSuggestion> {
        let partial_lower = partial.to_lowercase();
        let commands = get_common_commands();

        commands
            .into_iter()
            .filter(|(cmd, _)| cmd.starts_with(&partial_lower))
            .map(|(cmd, desc)| AICommandSuggestion {
                command: cmd,
                description: desc,
                parameters: Vec::new(),
                examples: Vec::new(),
                related_commands: Vec::new(),
            })
            .take(5)
            .collect()
    }
}

impl Default for AICommandService {
    fn default() -> Self {
        Self::new()
    }
}

/// 命令模式
struct CommandPattern {
    keywords: Vec<String>,
    template: String,
    description: String,
    parameters: Vec<CommandParameter>,
    examples: Vec<String>,
    related_commands: Vec<String>,
}

/// 获取命令模式
fn get_command_patterns() -> Vec<CommandPattern> {
    vec![
        CommandPattern {
            keywords: vec!["传送".to_string(), "tp".to_string(), "teleport".to_string()],
            template: "/tp @p {player}".to_string(),
            description: "传送到指定玩家".to_string(),
            parameters: vec![CommandParameter {
                name: "player".to_string(),
                param_type: "player".to_string(),
                required: true,
                default_value: None,
                description: "目标玩家名".to_string(),
            }],
            examples: vec!["/tp Steve".to_string(), "/tp @p 100 64 -200".to_string()],
            related_commands: vec!["/spreadplayers".to_string()],
        },
        CommandPattern {
            keywords: vec![
                "游戏模式".to_string(),
                "创造".to_string(),
                "生存".to_string(),
                "gamemode".to_string(),
            ],
            template: "/gamemode creative @p".to_string(),
            description: "设置游戏模式".to_string(),
            parameters: vec![CommandParameter {
                name: "mode".to_string(),
                param_type: "string".to_string(),
                required: true,
                default_value: Some("survival".to_string()),
                description: "游戏模式: survival, creative, adventure, spectator".to_string(),
            }],
            examples: vec![
                "/gamemode creative".to_string(),
                "/gamemode survival Steve".to_string(),
            ],
            related_commands: vec!["/defaultgamemode".to_string()],
        },
        CommandPattern {
            keywords: vec!["给予".to_string(), "give".to_string()],
            template: "/give @p {item} {number}".to_string(),
            description: "给予玩家物品".to_string(),
            parameters: vec![
                CommandParameter {
                    name: "player".to_string(),
                    param_type: "player".to_string(),
                    required: true,
                    default_value: None,
                    description: "目标玩家".to_string(),
                },
                CommandParameter {
                    name: "item".to_string(),
                    param_type: "item".to_string(),
                    required: true,
                    default_value: None,
                    description: "物品ID".to_string(),
                },
            ],
            examples: vec!["/give @p diamond 64".to_string()],
            related_commands: vec!["/clear".to_string()],
        },
        CommandPattern {
            keywords: vec![
                "时间".to_string(),
                "白天".to_string(),
                "黑夜".to_string(),
                "time".to_string(),
            ],
            template: "/time set day".to_string(),
            description: "设置游戏时间".to_string(),
            parameters: vec![CommandParameter {
                name: "time".to_string(),
                param_type: "string".to_string(),
                required: true,
                default_value: Some("day".to_string()),
                description: "时间: day, night, noon, midnight 或数值".to_string(),
            }],
            examples: vec!["/time set day".to_string(), "/time add 1000".to_string()],
            related_commands: vec!["/gamerule doDaylightCycle".to_string()],
        },
        CommandPattern {
            keywords: vec![
                "天气".to_string(),
                "下雨".to_string(),
                "晴天".to_string(),
                "weather".to_string(),
            ],
            template: "/weather clear".to_string(),
            description: "设置天气".to_string(),
            parameters: vec![CommandParameter {
                name: "type".to_string(),
                param_type: "string".to_string(),
                required: true,
                default_value: Some("clear".to_string()),
                description: "天气类型: clear, rain, thunder".to_string(),
            }],
            examples: vec!["/weather clear 6000".to_string()],
            related_commands: vec!["/gamerule doWeatherCycle".to_string()],
        },
    ]
}

/// 获取命令解释
fn get_command_explanations() -> HashMap<String, String> {
    let mut map = HashMap::new();

    map.insert("tp".to_string(), "传送实体到指定位置或实体".to_string());
    map.insert("gamemode".to_string(), "设置玩家的游戏模式".to_string());
    map.insert("give".to_string(), "给予玩家物品".to_string());
    map.insert("time".to_string(), "查看或修改游戏时间".to_string());
    map.insert("weather".to_string(), "设置天气".to_string());
    map.insert("kill".to_string(), "杀死实体".to_string());
    map.insert("effect".to_string(), "给予或清除实体效果".to_string());
    map.insert("enchant".to_string(), "附魔玩家手持物品".to_string());
    map.insert("summon".to_string(), "召唤实体".to_string());
    map.insert("setblock".to_string(), "放置方块".to_string());
    map.insert("fill".to_string(), "填充区域".to_string());
    map.insert("clone".to_string(), "复制区域".to_string());
    map.insert("execute".to_string(), "执行命令".to_string());
    map.insert("scoreboard".to_string(), "管理计分板".to_string());
    map.insert("tell".to_string(), "发送私聊消息".to_string());
    map.insert("say".to_string(), "广播消息".to_string());
    map.insert("title".to_string(), "显示标题".to_string());
    map.insert("bossbar".to_string(), "管理Boss栏".to_string());
    map.insert("worldborder".to_string(), "管理世界边界".to_string());
    map.insert("difficulty".to_string(), "设置游戏难度".to_string());
    map.insert("defaultgamemode".to_string(), "设置默认游戏模式".to_string());
    map.insert("gamerule".to_string(), "设置游戏规则".to_string());
    map.insert("seed".to_string(), "查看世界种子".to_string());
    map.insert("locate".to_string(), "定位结构".to_string());
    map.insert("spreadplayers".to_string(), "分散玩家".to_string());
    map.insert("particle".to_string(), "创建粒子效果".to_string());
    map.insert("playsound".to_string(), "播放音效".to_string());
    map.insert("stopsound".to_string(), "停止音效".to_string());
    map.insert("data".to_string(), "操作实体/方块数据".to_string());
    map.insert("item".to_string(), "操作物品".to_string());

    map
}

/// 获取常用命令列表
fn get_common_commands() -> Vec<(String, String)> {
    vec![
        ("/gamemode".to_string(), "设置游戏模式".to_string()),
        ("/tp".to_string(), "传送玩家".to_string()),
        ("/give".to_string(), "给予物品".to_string()),
        ("/time".to_string(), "设置时间".to_string()),
        ("/weather".to_string(), "设置天气".to_string()),
        ("/kill".to_string(), "杀死实体".to_string()),
        ("/effect".to_string(), "添加效果".to_string()),
        ("/enchant".to_string(), "附魔物品".to_string()),
        ("/summon".to_string(), "召唤实体".to_string()),
        ("/setblock".to_string(), "放置方块".to_string()),
        ("/fill".to_string(), "填充区域".to_string()),
        ("/clone".to_string(), "复制区域".to_string()),
        ("/execute".to_string(), "条件执行".to_string()),
        ("/tell".to_string(), "私聊玩家".to_string()),
        ("/say".to_string(), "广播消息".to_string()),
        ("/title".to_string(), "显示标题".to_string()),
        ("/clear".to_string(), "清除物品".to_string()),
        ("/difficulty".to_string(), "设置难度".to_string()),
        ("/gamerule".to_string(), "游戏规则".to_string()),
        ("/seed".to_string(), "世界种子".to_string()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_service_creation() {
        let service = AICommandService::new();
        assert!(true);
    }

    #[test]
    fn test_local_command_explain() {
        let service = AICommandService::new();
        let explain = service.local_command_explain("/gamemode creative");
        assert!(explain.is_some());
    }

    #[test]
    fn test_local_command_suggestions() {
        let service = AICommandService::new();
        let suggestions = service.local_command_suggestions("/gam");
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_command_patterns() {
        let patterns = get_command_patterns();
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_command_explanations() {
        let explanations = get_command_explanations();
        assert!(explanations.contains_key("tp"));
        assert!(explanations.contains_key("gamemode"));
    }
}
