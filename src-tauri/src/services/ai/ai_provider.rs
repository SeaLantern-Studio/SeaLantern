//! AI 提供商服务
//!
//! 支持多种 AI 提供商的统一接口

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Instant;

use super::{AIConfig, AIMessage, AIResponse};

/// AI 提供商类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AIProviderType {
    OpenAI,
    Anthropic,
    Local,
    Custom,
}

/// AI 提供商 trait

#[async_trait]
pub trait AIProvider: Send + Sync {
    /// 发送聊天请求
    async fn chat(&self, messages: &[AIMessage], config: &AIConfig) -> AIResponse;

    /// 获取提供商名称
    fn provider_name(&self) -> &str;

    /// 获取支持的模型列表
    fn get_supported_models(&self) -> Vec<String>;
}

/// OpenAI 提供商
pub struct OpenAIProvider {
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new() -> Self {
        Self { client: reqwest::Client::new() }
    }
}

impl Default for OpenAIProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn chat(&self, messages: &[AIMessage], config: &AIConfig) -> AIResponse {
        let start_time = Instant::now();

        let api_key = match &config.api_key {
            Some(key) => key,
            None => {
                return AIResponse {
                    success: false,
                    content: None,
                    error: Some("API key not configured".to_string()),
                    tokens_used: None,
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                };
            }
        };

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        let url = format!("{}/chat/completions", base_url);

        let messages_json: Vec<Value> = messages
            .iter()
            .map(|m| {
                json!({
                    "role": m.role,
                    "content": m.content
                })
            })
            .collect();

        let body = json!({
            "model": config.model,
            "messages": messages_json,
            "max_tokens": config.max_tokens,
            "temperature": config.temperature,
        });

        match self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Value>().await {
                        Ok(json) => {
                            let content = json["choices"][0]["message"]["content"]
                                .as_str()
                                .map(|s| s.to_string());
                            let tokens_used =
                                json["usage"]["total_tokens"].as_u64().map(|t| t as u32);

                            AIResponse {
                                success: true,
                                content,
                                error: None,
                                tokens_used,
                                response_time_ms: start_time.elapsed().as_millis() as u64,
                            }
                        }
                        Err(e) => AIResponse {
                            success: false,
                            content: None,
                            error: Some(format!("Failed to parse response: {}", e)),
                            tokens_used: None,
                            response_time_ms: start_time.elapsed().as_millis() as u64,
                        },
                    }
                } else {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    AIResponse {
                        success: false,
                        content: None,
                        error: Some(format!("API error: {}", error_text)),
                        tokens_used: None,
                        response_time_ms: start_time.elapsed().as_millis() as u64,
                    }
                }
            }
            Err(e) => AIResponse {
                success: false,
                content: None,
                error: Some(format!("Request failed: {}", e)),
                tokens_used: None,
                response_time_ms: start_time.elapsed().as_millis() as u64,
            },
        }
    }

    fn provider_name(&self) -> &str {
        "OpenAI"
    }

    fn get_supported_models(&self) -> Vec<String> {
        vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
            "gpt-3.5-turbo".to_string(),
        ]
    }
}

/// Anthropic 提供商
pub struct AnthropicProvider {
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new() -> Self {
        Self { client: reqwest::Client::new() }
    }
}

impl Default for AnthropicProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    async fn chat(&self, messages: &[AIMessage], config: &AIConfig) -> AIResponse {
        let start_time = Instant::now();

        let api_key = match &config.api_key {
            Some(key) => key,
            None => {
                return AIResponse {
                    success: false,
                    content: None,
                    error: Some("API key not configured".to_string()),
                    tokens_used: None,
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                };
            }
        };

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com/v1");
        let url = format!("{}/messages", base_url);

        // 将消息转换为 Anthropic 格式
        let mut system_prompt = String::new();
        let mut anthropic_messages: Vec<Value> = Vec::new();

        for m in messages {
            if m.role == "system" {
                system_prompt = m.content.clone();
            } else {
                anthropic_messages.push(json!({
                    "role": m.role,
                    "content": m.content
                }));
            }
        }

        let body = json!({
            "model": config.model,
            "max_tokens": config.max_tokens,
            "system": system_prompt,
            "messages": anthropic_messages,
        });

        match self
            .client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Value>().await {
                        Ok(json) => {
                            let content =
                                json["content"][0]["text"].as_str().map(|s| s.to_string());
                            let tokens_used = json["usage"]["input_tokens"]
                                .as_u64()
                                .and_then(|input| {
                                    json["usage"]["output_tokens"]
                                        .as_u64()
                                        .map(|output| input + output)
                                })
                                .map(|t| t as u32);

                            AIResponse {
                                success: true,
                                content,
                                error: None,
                                tokens_used,
                                response_time_ms: start_time.elapsed().as_millis() as u64,
                            }
                        }
                        Err(e) => AIResponse {
                            success: false,
                            content: None,
                            error: Some(format!("Failed to parse response: {}", e)),
                            tokens_used: None,
                            response_time_ms: start_time.elapsed().as_millis() as u64,
                        },
                    }
                } else {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    AIResponse {
                        success: false,
                        content: None,
                        error: Some(format!("API error: {}", error_text)),
                        tokens_used: None,
                        response_time_ms: start_time.elapsed().as_millis() as u64,
                    }
                }
            }
            Err(e) => AIResponse {
                success: false,
                content: None,
                error: Some(format!("Request failed: {}", e)),
                tokens_used: None,
                response_time_ms: start_time.elapsed().as_millis() as u64,
            },
        }
    }

    fn provider_name(&self) -> &str {
        "Anthropic"
    }

    fn get_supported_models(&self) -> Vec<String> {
        vec![
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
        ]
    }
}

/// 本地模型提供商 (Ollama 兼容)
pub struct LocalProvider {
    client: reqwest::Client,
}

impl LocalProvider {
    pub fn new() -> Self {
        Self { client: reqwest::Client::new() }
    }
}

impl Default for LocalProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AIProvider for LocalProvider {
    async fn chat(&self, messages: &[AIMessage], config: &AIConfig) -> AIResponse {
        let start_time = Instant::now();

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("http://localhost:11434");
        let url = format!("{}/api/chat", base_url);

        let messages_json: Vec<Value> = messages
            .iter()
            .map(|m| {
                json!({
                    "role": m.role,
                    "content": m.content
                })
            })
            .collect();

        let body = json!({
            "model": config.model,
            "messages": messages_json,
            "stream": false,
            "options": {
                "num_predict": config.max_tokens,
                "temperature": config.temperature,
            }
        });

        match self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Value>().await {
                        Ok(json) => {
                            let content =
                                json["message"]["content"].as_str().map(|s| s.to_string());
                            let tokens_used = json["eval_count"].as_u64().map(|t| t as u32);

                            AIResponse {
                                success: true,
                                content,
                                error: None,
                                tokens_used,
                                response_time_ms: start_time.elapsed().as_millis() as u64,
                            }
                        }
                        Err(e) => AIResponse {
                            success: false,
                            content: None,
                            error: Some(format!("Failed to parse response: {}", e)),
                            tokens_used: None,
                            response_time_ms: start_time.elapsed().as_millis() as u64,
                        },
                    }
                } else {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    AIResponse {
                        success: false,
                        content: None,
                        error: Some(format!("API error: {}", error_text)),
                        tokens_used: None,
                        response_time_ms: start_time.elapsed().as_millis() as u64,
                    }
                }
            }
            Err(e) => AIResponse {
                success: false,
                content: None,
                error: Some(format!("Request failed: {}", e)),
                tokens_used: None,
                response_time_ms: start_time.elapsed().as_millis() as u64,
            },
        }
    }

    fn provider_name(&self) -> &str {
        "Local"
    }

    fn get_supported_models(&self) -> Vec<String> {
        vec![
            "llama3.2".to_string(),
            "llama3.1".to_string(),
            "mistral".to_string(),
            "qwen2.5".to_string(),
            "deepseek-coder".to_string(),
        ]
    }
}

/// 获取提供商实例
pub fn get_provider(provider_type: &str) -> Box<dyn AIProvider> {
    match provider_type.to_lowercase().as_str() {
        "openai" => Box::new(OpenAIProvider::new()),
        "anthropic" => Box::new(AnthropicProvider::new()),
        "local" | "ollama" => Box::new(LocalProvider::new()),
        _ => Box::new(OpenAIProvider::new()), // 默认使用 OpenAI
    }
}

/// 获取所有支持的提供商
pub fn get_supported_providers() -> HashMap<String, Vec<String>> {
    let mut providers = HashMap::new();

    providers.insert("openai".to_string(), OpenAIProvider::new().get_supported_models());
    providers.insert("anthropic".to_string(), AnthropicProvider::new().get_supported_models());
    providers.insert("local".to_string(), LocalProvider::new().get_supported_models());

    providers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let openai = OpenAIProvider::new();
        assert_eq!(openai.provider_name(), "OpenAI");
        assert!(!openai.get_supported_models().is_empty());

        let anthropic = AnthropicProvider::new();
        assert_eq!(anthropic.provider_name(), "Anthropic");

        let local = LocalProvider::new();
        assert_eq!(local.provider_name(), "Local");
    }

    #[test]
    fn test_get_provider() {
        let provider = get_provider("openai");
        assert_eq!(provider.provider_name(), "OpenAI");

        let provider = get_provider("anthropic");
        assert_eq!(provider.provider_name(), "Anthropic");

        let provider = get_provider("local");
        assert_eq!(provider.provider_name(), "Local");
    }

    #[test]
    fn test_supported_providers() {
        let providers = get_supported_providers();
        assert!(providers.contains_key("openai"));
        assert!(providers.contains_key("anthropic"));
        assert!(providers.contains_key("local"));
    }
}
