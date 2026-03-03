//! AI 内容生成服务
//! 
//! 提供服务器公告、规则文档、活动策划等内容生成功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{AIConfig, AIContentGeneration, AIMessage, AIProvider, ai_provider::get_provider};

/// 内容生成类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    /// 服务器公告
    Announcement,
    /// 服务器规则
    ServerRules,
    /// 活动策划
    Event,
    /// 欢迎消息
    Welcome,
    /// 帮助文档
    Help,
    /// 新闻动态
    News,
    /// 广告宣传
    Advertisement,
}

/// 内容生成选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentGenerationOptions {
    /// 内容类型
    pub content_type: ContentType,
    /// 服务器名称
    pub server_name: String,
    /// 服务器特色
    pub server_features: Vec<String>,
    /// 目标受众
    pub target_audience: Option<String>,
    /// 语言
    pub language: Option<String>,
    /// 风格
    pub style: Option<ContentStyle>,
    /// 长度
    pub length: Option<ContentLength>,
    /// 额外要求
    pub extra_requirements: Option<String>,
}

/// 内容风格
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContentStyle {
    Formal,
    Casual,
    Friendly,
    Professional,
    Humorous,
    Dramatic,
}

/// 内容长度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContentLength {
    Short,
    Medium,
    Long,
    Detailed,
}

/// AI 内容生成服务
pub struct AIContentService;

impl AIContentService {
    /// 创建新的内容生成服务
    pub fn new() -> Self {
        Self
    }

    /// 生成内容
    pub async fn generate_content(
        &self,
        options: &ContentGenerationOptions,
        config: &AIConfig,
    ) -> AIContentGeneration {
        // 首先尝试本地模板
        if let Some(template_result) = self.local_template_generate(options) {
            if template_result.confidence > 0.5 {
                return template_result;
            }
        }

        // AI 生成
        if config.enabled {
            if let Some(ai_result) = self.ai_generate(options, config).await {
                return ai_result;
            }
        }

        // 回退到本地模板
        self.local_template_generate(options).unwrap_or_else(|| AIContentGeneration {
            content: "无法生成内容，请检查 AI 配置或提供更多信息。".to_string(),
            content_type: format!("{:?}", options.content_type),
            style: None,
            generated_at: chrono::Utc::now().to_rfc3339(),
            confidence: 0.0,
            suggestions: vec!["请启用 AI 功能以获得更好的生成效果".to_string()],
        })
    }

    /// 生成服务器公告
    pub async fn generate_announcement(
        &self,
        server_name: &str,
        topic: &str,
        details: Option<&str>,
        config: &AIConfig,
    ) -> Option<String> {
        if !config.enabled {
            return self.local_announcement(server_name, topic);
        }

        let provider = get_provider(&config.provider);

        let messages = vec![
            AIMessage {
                role: "system".to_string(),
                content: "你是一个专业的 Minecraft 服务器管理员，擅长撰写简洁明了的服务器公告。".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
            AIMessage {
                role: "user".to_string(),
                content: format!(
                    "服务器名称：{}\n公告主题：{}\n详细信息：{}\n\n请生成一条服务器公告。",
                    server_name,
                    topic,
                    details.unwrap_or("无")
                ),
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        let response = provider.chat(&messages, config).await;
        response.content
    }

    /// 生成服务器规则
    pub async fn generate_rules(
        &self,
        server_name: &str,
        server_type: &str,
        config: &AIConfig,
    ) -> Option<String> {
        if !config.enabled {
            return self.local_rules(server_name);
        }

        let provider = get_provider(&config.provider);

        let messages = vec![
            AIMessage {
                role: "system".to_string(),
                content: "你是一个专业的 Minecraft 服务器管理员，擅长制定清晰、公正的服务器规则。".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
            AIMessage {
                role: "user".to_string(),
                content: format!(
                    "服务器名称：{}\n服务器类型：{}\n\n请生成一套完整的服务器规则，包含基本规则、禁止事项和处罚措施。",
                    server_name, server_type
                ),
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        let response = provider.chat(&messages, config).await;
        response.content
    }

    /// 生成活动策划
    pub async fn generate_event(
        &self,
        server_name: &str,
        event_type: &str,
        duration: Option<&str>,
        config: &AIConfig,
    ) -> Option<String> {
        if !config.enabled {
            return self.local_event(server_name, event_type);
        }

        let provider = get_provider(&config.provider);

        let messages = vec![
            AIMessage {
                role: "system".to_string(),
                content: "你是一个专业的 Minecraft 活动策划师，擅长设计有趣且平衡的游戏活动。".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
            AIMessage {
                role: "user".to_string(),
                content: format!(
                    "服务器名称：{}\n活动类型：{}\n活动时长：{}\n\n请生成一个详细的活动策划方案，包含活动规则、奖励设置和注意事项。",
                    server_name,
                    event_type,
                    duration.unwrap_or("1周")
                ),
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        let response = provider.chat(&messages, config).await;
        response.content
    }

    /// 生成欢迎消息
    pub async fn generate_welcome(
        &self,
        server_name: &str,
        player_name: Option<&str>,
        config: &AIConfig,
    ) -> Option<String> {
        if !config.enabled {
            return self.local_welcome(server_name, player_name);
        }

        let provider = get_provider(&config.provider);

        let player_info = player_name.map(|n| format!("玩家名称：{}", n)).unwrap_or_default();

        let messages = vec![
            AIMessage {
                role: "system".to_string(),
                content: "你是一个友好的 Minecraft 服务器欢迎助手，擅长生成热情的欢迎消息。".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
            AIMessage {
                role: "user".to_string(),
                content: format!(
                    "服务器名称：{}\n{}\n\n请生成一条热情的欢迎消息，包含服务器简介和新手提示。",
                    server_name, player_info
                ),
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        let response = provider.chat(&messages, config).await;
        response.content
    }

    /// 生成帮助文档
    pub async fn generate_help(
        &self,
        server_name: &str,
        topic: &str,
        config: &AIConfig,
    ) -> Option<String> {
        if !config.enabled {
            return self.local_help(topic);
        }

        let provider = get_provider(&config.provider);

        let messages = vec![
            AIMessage {
                role: "system".to_string(),
                content: "你是一个专业的 Minecraft 技术支持，擅长编写清晰易懂的帮助文档。".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
            AIMessage {
                role: "user".to_string(),
                content: format!(
                    "服务器名称：{}\n帮助主题：{}\n\n请生成详细的帮助文档，包含步骤说明和注意事项。",
                    server_name, topic
                ),
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        let response = provider.chat(&messages, config).await;
        response.content
    }

    /// AI 内容生成
    async fn ai_generate(
        &self,
        options: &ContentGenerationOptions,
        config: &AIConfig,
    ) -> Option<AIContentGeneration> {
        let provider = get_provider(&config.provider);

        let system_prompt = self.build_content_system_prompt(options);
        let user_message = self.build_content_user_message(options);

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
                return Some(AIContentGeneration {
                    content,
                    content_type: format!("{:?}", options.content_type),
                    style: options.style.as_ref().map(|s| format!("{:?}", s)),
                    generated_at: chrono::Utc::now().to_rfc3339(),
                    confidence: 0.85,
                    suggestions: Vec::new(),
                });
            }
        }

        None
    }

    /// 构建内容生成系统提示词
    fn build_content_system_prompt(&self, options: &ContentGenerationOptions) -> String {
        let style_instruction = match &options.style {
            Some(ContentStyle::Formal) => "使用正式、专业的语言风格。",
            Some(ContentStyle::Casual) => "使用轻松、口语化的语言风格。",
            Some(ContentStyle::Friendly) => "使用亲切、友好的语言风格。",
            Some(ContentStyle::Professional) => "使用专业、客观的语言风格。",
            Some(ContentStyle::Humorous) => "使用幽默、风趣的语言风格。",
            Some(ContentStyle::Dramatic) => "使用戏剧性、引人入胜的语言风格。",
            None => "使用适当、得体的语言风格。",
        };

        let length_instruction = match &options.length {
            Some(ContentLength::Short) => "内容应简洁明了，不超过100字。",
            Some(ContentLength::Medium) => "内容应适中，约200-300字。",
            Some(ContentLength::Long) => "内容应详细，约500字左右。",
            Some(ContentLength::Detailed) => "内容应非常详细，可以超过500字。",
            None => "内容长度适中。",
        };

        let role = match options.content_type {
            ContentType::Announcement => "服务器公告撰写专家",
            ContentType::ServerRules => "服务器规则制定专家",
            ContentType::Event => "活动策划专家",
            ContentType::Welcome => "社区管理专家",
            ContentType::Help => "技术文档撰写专家",
            ContentType::News => "游戏新闻编辑",
            ContentType::Advertisement => "广告文案专家",
        };

        format!(
            r#"你是一个专业的 Minecraft {}。

{}
{}
请确保内容：
1. 符合 Minecraft 游戏背景
2. 语言准确、表达清晰
3. 结构合理、易于理解
"#,
            role, style_instruction, length_instruction
        )
    }

    /// 构建用户消息
    fn build_content_user_message(&self, options: &ContentGenerationOptions) -> String {
        let mut message = format!(
            "服务器名称：{}\n内容类型：{:?}",
            options.server_name, options.content_type
        );

        if !options.server_features.is_empty() {
            message.push_str(&format!("\n服务器特色：{}", options.server_features.join(", ")));
        }

        if let Some(audience) = &options.target_audience {
            message.push_str(&format!("\n目标受众：{}", audience));
        }

        if let Some(language) = &options.language {
            message.push_str(&format!("\n语言：{}", language));
        }

        if let Some(requirements) = &options.extra_requirements {
            message.push_str(&format!("\n额外要求：{}", requirements));
        }

        message.push_str("\n\n请生成内容。");

        message
    }

    /// 本地模板生成
    fn local_template_generate(&self, options: &ContentGenerationOptions) -> Option<AIContentGeneration> {
        match options.content_type {
            ContentType::Announcement => {
                Some(self.local_announcement_content(options))
            }
            ContentType::ServerRules => {
                Some(self.local_rules_content(options))
            }
            ContentType::Welcome => {
                Some(self.local_welcome_content(options))
            }
            ContentType::Event => {
                Some(self.local_event_content(options))
            }
            _ => None,
        }
    }

    /// 本地公告
    fn local_announcement(&self, server_name: &str, topic: &str) -> Option<String> {
        Some(format!(
            "【{} 公告】\n\n{}\n\n感谢各位玩家的支持！\n— {} 管理团队",
            server_name, topic, server_name
        ))
    }

    /// 本地公告内容
    fn local_announcement_content(&self, options: &ContentGenerationOptions) -> AIContentGeneration {
        AIContentGeneration {
            content: format!(
                "【{} 公告】\n\n欢迎来到 {}！\n{}\n\n感谢各位玩家的支持！",
                options.server_name,
                options.server_name,
                options.server_features.join("\n")
            ),
            content_type: "Announcement".to_string(),
            style: Some("Friendly".to_string()),
            generated_at: chrono::Utc::now().to_rfc3339(),
            confidence: 0.4,
            suggestions: vec!["启用 AI 可获得更丰富的公告内容".to_string()],
        }
    }

    /// 本地规则
    fn local_rules(&self, server_name: &str) -> Option<String> {
        Some(format!(
            r#"【{} 服务器规则】

一、基本规则
1. 尊重所有玩家，禁止辱骂、歧视行为
2. 禁止使用外挂、作弊等违规软件
3. 禁止恶意破坏其他玩家的建筑
4. 禁止恶意 PK 和骚扰行为

二、建筑规则
1. 请勿在他人领地附近未经允许建造
2. 保持建筑风格协调，避免过于突兀
3. 及时清理建筑垃圾和临时设施

三、处罚措施
- 违规者将根据情节轻重给予警告、禁言、封禁等处罚
- 严重违规者将永久封禁

祝您游戏愉快！
— {} 管理团队"#,
            server_name, server_name
        ))
    }

    /// 本地规则内容
    fn local_rules_content(&self, options: &ContentGenerationOptions) -> AIContentGeneration {
        AIContentGeneration {
            content: self.local_rules(&options.server_name).unwrap_or_default(),
            content_type: "ServerRules".to_string(),
            style: Some("Formal".to_string()),
            generated_at: chrono::Utc::now().to_rfc3339(),
            confidence: 0.4,
            suggestions: vec!["启用 AI 可获得更详细和定制化的规则".to_string()],
        }
    }

    /// 本地欢迎消息
    fn local_welcome(&self, server_name: &str, player_name: Option<&str>) -> Option<String> {
        let player_part = player_name.map(|n| format!("{}, ", n)).unwrap_or_default();
        Some(format!(
            "欢迎{}来到 {} 服务器！祝您游戏愉快！\n输入 /help 查看帮助，/rules 查看服务器规则。",
            player_part, server_name
        ))
    }

    /// 本地欢迎内容
    fn local_welcome_content(&self, options: &ContentGenerationOptions) -> AIContentGeneration {
        AIContentGeneration {
            content: format!(
                "欢迎来到 {} 服务器！\n\n{}\n\n输入 /help 查看帮助，/rules 查看服务器规则。\n祝您游戏愉快！",
                options.server_name,
                options.server_features.join("\n")
            ),
            content_type: "Welcome".to_string(),
            style: Some("Friendly".to_string()),
            generated_at: chrono::Utc::now().to_rfc3339(),
            confidence: 0.4,
            suggestions: vec!["启用 AI 可获得更个性化的欢迎消息".to_string()],
        }
    }

    /// 本地活动
    fn local_event(&self, server_name: &str, event_type: &str) -> Option<String> {
        Some(format!(
            r#"【{} - {} 活动】

活动时间：待定
活动地点：服务器主城

活动规则：
1. 所有玩家均可参与
2. 遵守服务器基本规则
3. 活动期间禁止作弊行为

奖励：
- 第一名：神秘大奖
- 参与奖：纪念物品

欢迎踊跃参与！
— {} 管理团队"#,
            server_name, event_type, server_name
        ))
    }

    /// 本地活动内容
    fn local_event_content(&self, options: &ContentGenerationOptions) -> AIContentGeneration {
        AIContentGeneration {
            content: format!(
                "【{} 特别活动】\n\n活动时间：待定\n活动内容：{}\n\n欢迎踊跃参与！",
                options.server_name,
                options.server_features.join(", ")
            ),
            content_type: "Event".to_string(),
            style: Some("Friendly".to_string()),
            generated_at: chrono::Utc::now().to_rfc3339(),
            confidence: 0.4,
            suggestions: vec!["启用 AI 可获得更详细的活动策划".to_string()],
        }
    }

    /// 本地帮助
    fn local_help(&self, topic: &str) -> Option<String> {
        let help_topics = get_help_topics();
        help_topics.get(topic).cloned()
    }
}

impl Default for AIContentService {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取帮助主题
fn get_help_topics() -> HashMap<String, String> {
    let mut topics = HashMap::new();

    topics.insert(
        "基础命令".to_string(),
        r#"【基础命令帮助】

常用命令：
- /spawn - 传送到出生点
- /home - 传送到家
- /sethome - 设置家
- /tpa <玩家> - 请求传送到玩家
- /tpahere <玩家> - 请求玩家传送到你
- /back - 返回上一个位置
- /balance - 查看余额
- /pay <玩家> <金额> - 支付金币

聊天命令：
- /msg <玩家> <消息> - 私聊
- /reply <消息> - 回复私聊
- /ignore <玩家> - 屏蔽玩家

信息查询：
- /help - 查看帮助
- /rules - 查看规则
- /list - 在线玩家列表
"#.to_string()
    );

    topics.insert(
        "领地保护".to_string(),
        r#"【领地保护帮助】

创建领地：
1. 使用木锄左键选择第一个角
2. 右键选择对角
3. 输入 /res create <领地名>

领地管理：
- /res remove <领地名> - 删除领地
- /res set <领地名> <权限> <true/false> - 设置权限
- /res pset <领地名> <玩家> <权限> <true/false> - 设置玩家权限
- /res trust <玩家> - 信任玩家

常用权限：
- build - 建造
- use - 使用
- container - 容器
- move - 移动
"#.to_string()
    );

    topics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_service_creation() {
        let service = AIContentService::new();
        assert!(true);
    }

    #[test]
    fn test_local_announcement() {
        let service = AIContentService::new();
        let result = service.local_announcement("TestServer", "服务器维护");
        assert!(result.is_some());
        assert!(result.unwrap().contains("TestServer"));
    }

    #[test]
    fn test_local_rules() {
        let service = AIContentService::new();
        let result = service.local_rules("TestServer");
        assert!(result.is_some());
        assert!(result.unwrap().contains("规则"));
    }

    #[test]
    fn test_local_welcome() {
        let service = AIContentService::new();
        let result = service.local_welcome("TestServer", Some("Steve"));
        assert!(result.is_some());
        assert!(result.unwrap().contains("Steve"));
    }

    #[test]
    fn test_get_help_topics() {
        let topics = get_help_topics();
        assert!(topics.contains_key("基础命令"));
        assert!(topics.contains_key("领地保护"));
    }
}
