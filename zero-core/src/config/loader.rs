// Config loader module
use super::{ConfigError, ConfigResult};
use std::env;
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

/// JSON file configuration loader
pub struct JsonConfigLoader;
impl ConfigLoader for JsonConfigLoader {
    fn load(&self, path: Option<&Path>) -> ConfigResult<serde_json::Value> {
        let path = path.ok_or_else(|| ConfigError::NotFound("path required".to_string()))?;
        let content =
            std::fs::read_to_string(path).map_err(|e| ConfigError::LoadFailed(e.to_string()))?;
        serde_json::from_str(&content).map_err(|e| ConfigError::LoadFailed(e.to_string()))
    }

    fn save(&self, _value: &serde_json::Value, _path: Option<&Path>) -> ConfigResult<()> {
        Err(ConfigError::SaveFailed("save not implemented".to_string()))
    }

    fn validate(&self, _value: &serde_json::Value) -> ConfigResult<()> {
        Ok(())
    }
}

/// Environment configuration loader
pub struct EnvConfigLoader;
impl ConfigLoader for EnvConfigLoader {
    fn load(&self, _path: Option<&Path>) -> ConfigResult<serde_json::Value> {
        let mut map = serde_json::Map::new();
        for (key, value) in env::vars() {
            map.insert(key, serde_json::json!(value));
        }
        Ok(serde_json::Value::Object(map))
    }

    fn save(&self, _value: &serde_json::Value, _path: Option<&Path>) -> ConfigResult<()> {
        Ok(())
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
