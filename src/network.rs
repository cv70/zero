use crate::security::validate_json;
use crate::security::SecurityError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct NetworkInput {
    pub address: String,
    pub port: u16,
    pub payload: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct NetworkOutput {
    pub ok: bool,
    pub message: String,
}

pub fn process(input_json: &str) -> Result<NetworkOutput, SecurityError> {
    let mut input: NetworkInput = validate_json(input_json)?;
    if input.address.trim().is_empty() {
        return Err(SecurityError::InvalidInput("address empty".to_string()));
    }
    if input.port == 0 {
        return Err(SecurityError::InvalidInput("port must be > 0".to_string()));
    }
    // Basic normalization / sanitization
    input.address = input.address.trim().to_string();
    Ok(NetworkOutput {
        ok: true,
        message: format!("connected to {}:{}", input.address, input.port),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_input() {
        let json = r#"{"address":"127.0.0.1","port":1}"#;
        let res = process(json);
        assert!(res.is_ok());
    }

    #[test]
    fn invalid_input_port_zero() {
        let json = r#"{"address":"127.0.0.1","port":0}"#;
        let res = process(json);
        assert!(res.is_err());
    }
}
