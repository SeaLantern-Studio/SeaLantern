//! AI 翻译服务
//!
//! 提供多语言翻译功能，支持聊天消息、公告等内容翻译

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{ai_provider::get_provider, AIConfig, AIMessage, AIProvider, AITranslationResult};

/// 支持的语言
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    ChineseSimplified,
    ChineseTraditional,
    English,
    Japanese,
    Korean,
    French,
    German,
    Spanish,
    Portuguese,
    Russian,
    Arabic,
    Thai,
    Vietnamese,
    Indonesian,
    Auto,
}

impl Language {
    /// 获取语言代码
    pub fn code(&self) -> &'static str {
        match self {
            Language::ChineseSimplified => "zh-CN",
            Language::ChineseTraditional => "zh-TW",
            Language::English => "en",
            Language::Japanese => "ja",
            Language::Korean => "ko",
            Language::French => "fr",
            Language::German => "de",
            Language::Spanish => "es",
            Language::Portuguese => "pt",
            Language::Russian => "ru",
            Language::Arabic => "ar",
            Language::Thai => "th",
            Language::Vietnamese => "vi",
            Language::Indonesian => "id",
            Language::Auto => "auto",
        }
    }

    /// 获取语言显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::ChineseSimplified => "简体中文",
            Language::ChineseTraditional => "繁體中文",
            Language::English => "English",
            Language::Japanese => "日本語",
            Language::Korean => "한국어",
            Language::French => "Français",
            Language::German => "Deutsch",
            Language::Spanish => "Español",
            Language::Portuguese => "Português",
            Language::Russian => "Русский",
            Language::Arabic => "العربية",
            Language::Thai => "ไทย",
            Language::Vietnamese => "Tiếng Việt",
            Language::Indonesian => "Bahasa Indonesia",
            Language::Auto => "自动检测",
        }
    }

    /// 从代码解析
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "zh-cn" | "zh" | "chs" => Some(Language::ChineseSimplified),
            "zh-tw" | "cht" => Some(Language::ChineseTraditional),
            "en" | "eng" => Some(Language::English),
            "ja" | "jpn" => Some(Language::Japanese),
            "ko" | "kor" => Some(Language::Korean),
            "fr" | "fra" => Some(Language::French),
            "de" | "deu" => Some(Language::German),
            "es" | "spa" => Some(Language::Spanish),
            "pt" | "por" => Some(Language::Portuguese),
            "ru" | "rus" => Some(Language::Russian),
            "ar" | "ara" => Some(Language::Arabic),
            "th" | "tha" => Some(Language::Thai),
            "vi" | "vie" => Some(Language::Vietnamese),
            "id" | "ind" => Some(Language::Indonesian),
            "auto" => Some(Language::Auto),
            _ => None,
        }
    }
}

/// 翻译选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationOptions {
    /// 源语言
    pub source_language: Language,
    /// 目标语言
    pub target_language: Language,
    /// 翻译上下文 (Minecraft 相关)
    pub context: Option<TranslationContext>,
    /// 是否保留格式代码
    pub preserve_formatting: bool,
}

/// 翻译上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TranslationContext {
    /// 聊天消息
    Chat,
    /// 服务器公告
    Announcement,
    /// 服务器规则
    Rules,
    /// 帮助信息
    Help,
    /// 物品描述
    ItemDescription,
    /// UI 文本
    UIText,
    /// 通用
    General,
}

/// 翻译缓存
pub struct TranslationCache {
    cache: HashMap<String, AITranslationResult>,
    max_size: usize,
}

impl TranslationCache {
    pub fn new(max_size: usize) -> Self {
        Self { cache: HashMap::new(), max_size }
    }

    /// 生成缓存键
    fn cache_key(text: &str, source: &Language, target: &Language) -> String {
        format!("{}:{}:{}", source.code(), target.code(), text)
    }

    /// 获取缓存
    pub fn get(
        &self,
        text: &str,
        source: &Language,
        target: &Language,
    ) -> Option<&AITranslationResult> {
        let key = Self::cache_key(text, source, target);
        self.cache.get(&key)
    }

    /// 设置缓存
    pub fn set(
        &mut self,
        text: &str,
        source: &Language,
        target: &Language,
        result: AITranslationResult,
    ) {
        if self.cache.len() >= self.max_size {
            // 简单的 LRU：移除第一个条目
            if let Some(first_key) = self.cache.keys().next().cloned() {
                self.cache.remove(&first_key);
            }
        }

        let key = Self::cache_key(text, source, target);
        self.cache.insert(key, result);
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

/// AI 翻译服务
pub struct AITranslatorService {
    cache: TranslationCache,
}

impl AITranslatorService {
    /// 创建新的翻译服务
    pub fn new() -> Self {
        Self { cache: TranslationCache::new(500) }
    }

    /// 翻译文本
    pub async fn translate(
        &mut self,
        text: &str,
        options: &TranslationOptions,
        config: &AIConfig,
    ) -> AITranslationResult {
        // 检查缓存
        if let Some(cached) =
            self.cache
                .get(text, &options.source_language, &options.target_language)
        {
            return cached.clone();
        }

        // AI 翻译
        let result = if config.enabled {
            self.ai_translate(text, options, config).await
        } else {
            self.local_translate(text, options)
        };

        // 缓存结果
        if result.confidence > 0.8 {
            self.cache.set(
                text,
                &options.source_language,
                &options.target_language,
                result.clone(),
            );
        }

        result
    }

    /// 批量翻译
    pub async fn translate_batch(
        &mut self,
        texts: &[String],
        options: &TranslationOptions,
        config: &AIConfig,
    ) -> Vec<AITranslationResult> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            let result = self.translate(text, options, config).await;
            results.push(result);
        }

        results
    }

    /// 检测语言
    pub async fn detect_language(&self, text: &str, config: &AIConfig) -> Option<Language> {
        if !config.enabled {
            return self.local_detect_language(text);
        }

        let provider = get_provider(&config.provider);

        let messages = vec![
            AIMessage {
                role: "system".to_string(),
                content: "你是一个语言检测专家。请检测文本的语言，只返回语言代码（如 zh-CN, en, ja 等）。".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
            AIMessage {
                role: "user".to_string(),
                content: text.to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        let response = provider.chat(&messages, config).await;

        if response.success {
            if let Some(content) = response.content {
                return Language::from_code(content.trim());
            }
        }

        self.local_detect_language(text)
    }

    /// AI 翻译
    async fn ai_translate(
        &self,
        text: &str,
        options: &TranslationOptions,
        config: &AIConfig,
    ) -> AITranslationResult {
        let provider = get_provider(&config.provider);

        let context_instruction = match &options.context {
            Some(TranslationContext::Chat) => "这是游戏聊天消息，请保持口语化和友好。",
            Some(TranslationContext::Announcement) => "这是服务器公告，请使用正式但友好的语气。",
            Some(TranslationContext::Rules) => "这是服务器规则，请使用清晰准确的语言。",
            Some(TranslationContext::Help) => "这是帮助信息，请保持简洁明了。",
            Some(TranslationContext::ItemDescription) => "这是物品描述，请保持游戏术语一致性。",
            Some(TranslationContext::UIText) => "这是界面文本，请保持简洁。",
            _ => "请进行准确的翻译。",
        };

        let preserve_instruction = if options.preserve_formatting {
            "请保留所有 Minecraft 格式代码（如 §a, &b 等）和颜色代码。"
        } else {
            ""
        };

        let system_prompt = format!(
            r#"你是一个专业的 Minecraft 游戏翻译专家。
{}
{}

源语言：{}
目标语言：{}

请只返回翻译结果，不要添加任何解释。
"#,
            context_instruction,
            preserve_instruction,
            options.source_language.display_name(),
            options.target_language.display_name()
        );

        let messages = vec![
            AIMessage {
                role: "system".to_string(),
                content: system_prompt,
                timestamp: chrono::Utc::now().timestamp(),
            },
            AIMessage {
                role: "user".to_string(),
                content: text.to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        let response = provider.chat(&messages, config).await;

        if response.success {
            if let Some(translated) = response.content {
                return AITranslationResult {
                    original_text: text.to_string(),
                    source_language: options.source_language.code().to_string(),
                    translated_text: translated.trim().to_string(),
                    target_language: options.target_language.code().to_string(),
                    confidence: 0.9,
                };
            }
        }

        // 翻译失败，返回原文
        AITranslationResult {
            original_text: text.to_string(),
            source_language: options.source_language.code().to_string(),
            translated_text: text.to_string(),
            target_language: options.target_language.code().to_string(),
            confidence: 0.0,
        }
    }

    /// 本地简单翻译（AI 未启用时的回退）
    fn local_translate(&self, text: &str, _options: &TranslationOptions) -> AITranslationResult {
        // 本地翻译字典（常见 Minecraft 术语）
        let dictionary = get_translation_dictionary();

        let mut translated = text.to_string();
        for (source, target) in &dictionary {
            translated = translated.replace(source, target);
        }

        AITranslationResult {
            original_text: text.to_string(),
            source_language: "auto".to_string(),
            translated_text: translated,
            target_language: "auto".to_string(),
            confidence: 0.3, // 低置信度表示是简单替换
        }
    }

    /// 本地语言检测
    fn local_detect_language(&self, text: &str) -> Option<Language> {
        // 简单的字符范围检测
        let has_chinese = text.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c));
        let has_japanese = text.chars().any(|c| {
            ('\u{3040}'..='\u{309F}').contains(&c) || ('\u{30A0}'..='\u{30FF}').contains(&c)
        });
        let has_korean = text.chars().any(|c| ('\u{AC00}'..='\u{D7AF}').contains(&c));
        let has_arabic = text.chars().any(|c| ('\u{0600}'..='\u{06FF}').contains(&c));
        let has_cyrillic = text.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c));

        if has_chinese {
            Some(Language::ChineseSimplified)
        } else if has_japanese {
            Some(Language::Japanese)
        } else if has_korean {
            Some(Language::Korean)
        } else if has_arabic {
            Some(Language::Arabic)
        } else if has_cyrillic {
            Some(Language::Russian)
        } else {
            Some(Language::English)
        }
    }

    /// 清空缓存
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for AITranslatorService {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取翻译字典
fn get_translation_dictionary() -> Vec<(String, String)> {
    vec![
        // 常见游戏术语
        ("player".to_string(), "玩家".to_string()),
        ("server".to_string(), "服务器".to_string()),
        ("world".to_string(), "世界".to_string()),
        ("spawn".to_string(), "出生点".to_string()),
        ("item".to_string(), "物品".to_string()),
        ("block".to_string(), "方块".to_string()),
        ("entity".to_string(), "实体".to_string()),
        ("mob".to_string(), "生物".to_string()),
        ("crafting".to_string(), "合成".to_string()),
        ("inventory".to_string(), "背包".to_string()),
        ("health".to_string(), "生命值".to_string()),
        ("damage".to_string(), "伤害".to_string()),
        ("experience".to_string(), "经验".to_string()),
        ("level".to_string(), "等级".to_string()),
        ("achievement".to_string(), "成就".to_string()),
        ("advancement".to_string(), "进度".to_string()),
        ("biome".to_string(), "生物群系".to_string()),
        ("dimension".to_string(), "维度".to_string()),
        ("enchantment".to_string(), "附魔".to_string()),
        ("potion".to_string(), "药水".to_string()),
        ("recipe".to_string(), "配方".to_string()),
        ("redstone".to_string(), "红石".to_string()),
        ("command".to_string(), "命令".to_string()),
        ("plugin".to_string(), "插件".to_string()),
        ("mod".to_string(), "模组".to_string()),
        ("whitelist".to_string(), "白名单".to_string()),
        ("blacklist".to_string(), "黑名单".to_string()),
        ("ban".to_string(), "封禁".to_string()),
        ("kick".to_string(), "踢出".to_string()),
        ("op".to_string(), "管理员".to_string()),
        ("gamemode".to_string(), "游戏模式".to_string()),
        ("survival".to_string(), "生存".to_string()),
        ("creative".to_string(), "创造".to_string()),
        ("adventure".to_string(), "冒险".to_string()),
        ("spectator".to_string(), "旁观".to_string()),
    ]
}

/// 获取支持的语言列表
pub fn get_supported_languages() -> Vec<Language> {
    vec![
        Language::ChineseSimplified,
        Language::ChineseTraditional,
        Language::English,
        Language::Japanese,
        Language::Korean,
        Language::French,
        Language::German,
        Language::Spanish,
        Language::Portuguese,
        Language::Russian,
        Language::Arabic,
        Language::Thai,
        Language::Vietnamese,
        Language::Indonesian,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_code() {
        assert_eq!(Language::ChineseSimplified.code(), "zh-CN");
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::Japanese.code(), "ja");
    }

    #[test]
    fn test_language_from_code() {
        assert_eq!(Language::from_code("zh-CN"), Some(Language::ChineseSimplified));
        assert_eq!(Language::from_code("en"), Some(Language::English));
        assert_eq!(Language::from_code("ja"), Some(Language::Japanese));
        assert_eq!(Language::from_code("unknown"), None);
    }

    #[test]
    fn test_local_detect_language() {
        let service = AITranslatorService::new();

        assert_eq!(service.local_detect_language("你好世界"), Some(Language::ChineseSimplified));
        assert_eq!(service.local_detect_language("Hello World"), Some(Language::English));
        assert_eq!(service.local_detect_language("こんにちは"), Some(Language::Japanese));
    }

    #[test]
    fn test_translation_cache() {
        let mut cache = TranslationCache::new(10);

        let result = AITranslationResult {
            original_text: "Hello".to_string(),
            source_language: "en".to_string(),
            translated_text: "你好".to_string(),
            target_language: "zh-CN".to_string(),
            confidence: 0.9,
        };

        cache.set("Hello", &Language::English, &Language::ChineseSimplified, result.clone());

        let cached = cache.get("Hello", &Language::English, &Language::ChineseSimplified);
        assert!(cached.is_some());
    }

    #[test]
    fn test_get_supported_languages() {
        let languages = get_supported_languages();
        assert!(!languages.is_empty());
        assert!(languages.contains(&Language::ChineseSimplified));
        assert!(languages.contains(&Language::English));
    }
}
