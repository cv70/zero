// Config loader module
use super::{ConfigError, ConfigResult};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Configuration loader trait
pub trait ConfigLoader: Send + Sync {
    /// Load configuration from source
    fn load(&self, path: Option<&Path>) -> ConfigResult<serde_json::Value>;
    /// Save configuration to source
    fn save(&self, value: &serde_json::Value, path: Option<&Path>) -> ConfigResult<()>;
    /// Validate configuration
    fn validate(&self, value: &serde_json::Value) -> ConfigResult<()>;
}

/// YAML file configuration loader
pub struct YamlConfigLoader;

impl ConfigLoader for YamlConfigLoader {
    fn load(&self, path: Option<&Path>) -> ConfigResult<serde_json::Value> {
        let path = path.ok_or_else(|| ConfigError::NotFound("path required".to_string()))?;
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::LoadFailed(format!("YAML load error: {}", e)))?;
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::LoadFailed(format!("YAML parse error: {}", e)))?;
        serde_json::to_string(&yaml_value)
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s))
            .map_err(|e| ConfigError::LoadFailed(format!("YAML to JSON error: {}", e)))
    }

    fn save(&self, _value: &serde_json::Value, _path: Option<&Path>) -> ConfigResult<()> {
        Err(ConfigError::SaveFailed("save not implemented".to_string()))
    }

    fn validate(&self, _value: &serde_json::Value) -> ConfigResult<()> {
        Ok(())
    }
}

/// Composite configuration loader that tries loaders in order
pub struct CompositeConfigLoader {
    loaders: Vec<Box<dyn ConfigLoader>>,
}

impl CompositeConfigLoader {
    pub fn new() -> Self {
        Self {
            loaders: Vec::new(),
        }
    }

    pub fn add_loader(&mut self, loader: Box<dyn ConfigLoader>) {
        self.loaders.push(loader);
    }
}

impl Default for CompositeConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigLoader for CompositeConfigLoader {
    fn load(&self, path: Option<&Path>) -> ConfigResult<serde_json::Value> {
        for loader in &self.loaders {
            match loader.load(path) {
                Ok(value) => return Ok(value),
                Err(_) => continue,
            }
        }
        Err(ConfigError::NotFound("no loader succeeded".to_string()))
    }

    fn save(&self, _value: &serde_json::Value, _path: Option<&Path>) -> ConfigResult<()> {
        Ok(())
    }

    fn validate(&self, _value: &serde_json::Value) -> ConfigResult<()> {
        Ok(())
    }
}

/// Builder for configuration values
#[allow(dead_code)]
pub struct ConfigBuilder {
    config: serde_json::Value,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    pub fn build(self) -> serde_json::Value {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Main configuration structure
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    /// Application name
    pub name: Option<String>,
    /// Application version
    pub version: Option<String>,
    /// Application description
    pub description: Option<String>,
    /// Provider configuration
    pub provider: Option<ProviderConfig>,
    /// Tool configuration
    pub tool: Option<ToolConfig>,
    /// Channel configuration
    pub channel: Option<ChannelConfig>,
    /// Additional settings
    pub settings: Option<HashMap<String, serde_yaml::Value>>,
    /// List of features
    pub features: Option<Vec<String>>,
}

impl Config {
    /// Load configuration from YAML file
    pub fn from_yaml_file(path: &str) -> ConfigResult<Self> {
        let content = fs::read_to_string(path).map_err(|e| {
            ConfigError::LoadFailed(format!("Cannot read config file '{}': {}", path, e))
        })?;

        let config: Self = serde_yaml::from_str(&content).map_err(|e| {
            ConfigError::LoadFailed(format!("Cannot parse config in '{}': {}", path, e))
        })?;

        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> ConfigResult<()> {
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: Some("Zero Agent".to_string()),
            version: Some("1.0.0".to_string()),
            description: Some("A smart AI agent for coding".to_string()),
            provider: Some(ProviderConfig::default()),
            tool: Some(ToolConfig::default()),
            channel: Some(ChannelConfig::default()),
            settings: Some(HashMap::new()),
            features: Some(vec![]),
        }
    }
}

/// Provider configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ProviderConfig {
    /// Provider name (anthropic, openai, ollama)
    pub name: Option<String>,
    /// API key for the provider
    pub api_key: Option<String>,
    /// Model name to use with the provider
    pub model: Option<String>,
    /// API endpoint URL
    pub endpoint: Option<String>,
    /// Enable streaming output
    #[serde(default)]
    pub stream: bool,
    /// System prompt
    pub system_prompt: Option<String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            name: Some("anthropic".to_string()),
            api_key: None,
            model: None,
            endpoint: None,
            stream: false,
            system_prompt: None,
        }
    }
}

/// Tool configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ToolConfig {
    /// Enable built-in tools
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// List of built-in tool paths to load
    #[serde(default)]
    pub paths: Vec<String>,
    /// Shell command timeout in seconds
    #[serde(default = "default_tool_timeout")]
    pub timeout: u64,
}

fn default_true() -> bool {
    true
}

fn default_tool_timeout() -> u64 {
    60
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            paths: vec![],
            timeout: 60,
        }
    }
}

/// Channel configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ChannelConfig {
    /// Channel type (cli, discord, slack)
    #[serde(rename = "type")]
    pub type_: String,
    /// Webhook URL for the channel
    pub webhook_url: Option<String>,
    /// Enable channel
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            type_: "cli".to_string(),
            webhook_url: None,
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.name, Some("Zero Agent".to_string()));
        assert!(config.provider.is_some());
        assert!(config.tool.is_some());
        assert!(config.channel.is_some());
    }

    #[test]
    fn test_config_from_yaml_string() {
        let yaml = r#"
name: "TestAgent"
version: "0.1.0"
description: "Test agent"

provider:
  name: "openai"
  model: "gpt-4"
  api_key: "sk-test"
  endpoint: "https://api.openai.com/v1"
  stream: true
  system_prompt: "You are a test assistant."

tool:
  enabled: true
  paths:
    - "bash"
    - "file:read"
  timeout: 30

channel:
  type: "cli"
  enabled: true

features:
  - "openai"
  - "web"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.name, Some("TestAgent".to_string()));

        let provider = config.provider.unwrap();
        assert_eq!(provider.name, Some("openai".to_string()));
        assert_eq!(provider.model, Some("gpt-4".to_string()));
        assert!(provider.stream);
        assert_eq!(
            provider.endpoint,
            Some("https://api.openai.com/v1".to_string())
        );
        assert!(provider.system_prompt.is_some());

        let tool = config.tool.unwrap();
        assert!(tool.enabled);
        assert_eq!(tool.paths.len(), 2);
        assert_eq!(tool.timeout, 30);

        let channel = config.channel.unwrap();
        assert_eq!(channel.type_, "cli");
        assert!(channel.enabled);

        assert_eq!(
            config.features,
            Some(vec!["openai".to_string(), "web".to_string()])
        );
    }
}
