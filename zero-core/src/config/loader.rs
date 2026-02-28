// Config loader module
use super::{ConfigError, ConfigResult};
use crate::error::ModuleError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

    /// Get configuration as type
    fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> ConfigResult<T> {
        let value = self.load(None)?;
        serde_json::from_value(value.get(key).cloned().unwrap_or_default())
            .map_err(|e| ConfigError::LoadFailed(e.to_string()))
    }
}

/// JSON file configuration loader
pub struct JsonConfigLoader;
impl ConfigLoader for JsonConfigLoader {
    fn load(&self, path: Option<&Path>) -> ConfigResult<serde_json::Value> {
        let path = path.expect("path must exist");
        let content =
            std::fs::read_to_string(path).map_err(|e| ConfigError::LoadFailed(e.to_string()))?;
        serde_json::from_str(&content).map_err(|e| ConfigError::LoadFailed(e.to_string()))
    }

    fn save(&self, _value: &serde_json::Value, _path: Option<&Path>) -> ConfigResult<()> {
        unimplemented!("save not implemented");
    }

    fn validate(&self, _value: &serde_json::Value) -> ConfigResult<()> {
        unimplemented!("validate not implemented");
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
        Ok(serde_json::Value::Object(map).map_err(|e| ConfigError::FormatError(e))?)
    }

    fn save(&self, _value: &serde_json::Value, _path: Option<&Path>) -> ConfigResult<()> {
        Ok(())
    }

    fn validate(&self, _value: &serde_json::Value) -> ConfigResult<()> {
        Ok(())
    }
}

/// Composite configuration loader
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

impl ConfigLoader for CompositeConfigLoader {
    fn load(&self, path: Option<&Path>) -> ConfigResult<serde_json::Value> {
        for loader in &self.loaders {
            match loader.load(path) {
                Ok(value) => return Ok(value),
                Err(_) => continue,
            }
        }
        Err(ConfigError::NotFound("config not found".to_string())
            .map_err(|e| ConfigError::LoadFailed(e.to_string()))?)
    }

    fn save(&self, _value: &serde_json::Value, _path: Option<&Path>) -> ConfigResult<()> {
        Ok(())
    }

    fn validate(&self, _value: &serde_json::Value) -> ConfigResult<()> {
        Ok(())
    }
}

/// Builder for configuration
#[allow(dead_code)]
pub struct ConfigBuilder {
    config: serde_json::Value,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: serde_json::Value::Object(serde_json::Map::new())
                .map_err(|e| ConfigError::FormatError(e))
                .unwrap_or_default(),
        }
    }

    pub fn build(self) -> serde_json::Value {
        self.config
    }
}
