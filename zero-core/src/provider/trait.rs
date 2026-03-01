use crate::error::ProviderError;
use async_trait::async_trait;

/// 模型能力枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelCapability {
    TextOnly,
    TextAndImages,
    TextAndVideo,
    Multimodal,
}

/// 媒体输入类型
#[derive(Debug, Clone)]
pub enum MediaInput {
    Image { url: String, mime_type: String },
    ImageBytes { data: Vec<u8>, mime_type: String },
    Video { url: String, mime_type: String },
    Audio { url: String, mime_type: String },
}

/// Tool metadata
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
}

/// 补全选项
#[derive(Debug, Clone)]
pub struct CompleteOpts {
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub tools: Vec<ToolMetadata>,
    pub system_prompt: Option<String>,
}

impl Default for CompleteOpts {
    fn default() -> Self {
        Self {
            model: String::new(),
            temperature: Some(0.7),
            max_tokens: None,
            tools: Vec::new(),
            system_prompt: None,
        }
    }
}

/// Tool 调用请求
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// Tool 调用结果
#[derive(Debug, Clone)]
pub struct ToolCallResult {
    pub id: String,
    pub result: Result<String, String>,
}

/// LLM Provider Trait
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Provider 名称
    fn name(&self) -> &str;

    /// 支持的模型能力
    fn capabilities(&self) -> ModelCapability;

    /// 可用模型列表
    fn available_models(&self) -> Vec<String>;

    /// 纯文本补全
    async fn complete(&self, prompt: &str, opts: CompleteOpts) -> Result<String, ProviderError>;

    /// 多模态补全（可选实现）
    async fn complete_with_media(
        &self,
        _prompt: &str,
        _media: &[MediaInput],
        _opts: CompleteOpts,
    ) -> Result<String, ProviderError> {
        Err(ProviderError::RequestFailed(
            "Multimodal not supported".into(),
        ))
    }

    /// Tool 调用补全（可选实现）
    async fn complete_with_tools(
        &self,
        _prompt: &str,
        _tools: &[ToolCall],
        _opts: CompleteOpts,
    ) -> Result<ToolCallResult, ProviderError> {
        Err(ProviderError::RequestFailed(
            "Tool calling not supported".into(),
        ))
    }
}
