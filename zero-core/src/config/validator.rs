// Config validator module
use super::ConfigResult;
use serde_json::Value;

/// Config validator trait
pub trait ConfigValidator: Send + Sync {
    /// Validate configuration value
    fn validate(&self, value: &Value) -> ConfigResult<()>;

    /// Get validation rules
    fn get_schema(&self) -> Value;

    /// Set validation rules
    fn set_schema(&mut self, schema: Value);
}

/// Simple config validator
pub struct SimpleConfigValidator {
    schema: Value,
}

impl SimpleConfigValidator {
    pub fn new(schema: Value) -> Self {
        Self { schema }
    }
}

impl ConfigValidator for SimpleConfigValidator {
    fn validate(&self, _value: &Value) -> ConfigResult<()> {
        Ok(())
    }

    fn get_schema(&self) -> Value {
        self.schema.clone()
    }

    fn set_schema(&mut self, schema: Value) {
        self.schema = schema;
    }
}

/// Composite config validator
pub struct CompositeConfigValidator {
    validators: Vec<Box<dyn ConfigValidator>>,
}

impl CompositeConfigValidator {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    pub fn add_validator(&mut self, validator: Box<dyn ConfigValidator>) {
        self.validators.push(validator);
    }
}

impl ConfigValidator for CompositeConfigValidator {
    fn validate(&self, value: &Value) -> ConfigResult<()> {
        for validator in &self.validators {
            validator.validate(value)?;
        }
        Ok(())
    }

    fn get_schema(&self) -> Value {
        Value::Object(serde_json::Map::new())
    }

    fn set_schema(&mut self, _schema: Value) {
        // Not implemented
    }
}
