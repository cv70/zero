use crate::security::validate_json;
use crate::security::SecurityError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct StorageInput {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Debug)]
pub struct StorageOutput {
    pub stored: bool,
}

pub fn put(input_json: &str) -> Result<StorageOutput, SecurityError> {
    let mut input: StorageInput = validate_json(input_json)?;
    if input.key.trim().is_empty() {
        return Err(SecurityError::InvalidInput("empty key".to_string()));
    }
    input.key = input.key.trim().to_string();
    // In real implementation: persist to storage with input.key/input.value
    Ok(StorageOutput { stored: true })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_put() {
        let json = r#"{"key":"k1","value":"v"}"#;
        let res = put(json);
        assert!(res.is_ok());
    }

    #[test]
    fn empty_key() {
        let json = r#"{"key":"","value":"v"}"#;
        let res = put(json);
        assert!(res.is_err());
    }
}
