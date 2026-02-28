use crate::security::validate_json;
use crate::security::SecurityError;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct AuthInput {
    pub username: String,
    pub password: String,
}

pub fn login(input_json: &str) -> Result<String, SecurityError> {
    let mut input: AuthInput = validate_json(input_json)?;
    if input.username.trim().is_empty() {
        return Err(SecurityError::InvalidInput("username empty".to_string()));
    }
    if input.password.trim().len() < 8 {
        return Err(SecurityError::InvalidInput(
            "password too short".to_string(),
        ));
    }
    input.username = input.username.trim().to_string();
    // Return a simple token (in real apps use JWT or similar)
    Ok(format!("token-{}", Uuid::new_v4()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_login() {
        let json = r#"{"username":"bob","password":"password123"}"#;
        let res = login(json);
        assert!(res.is_ok());
    }

    #[test]
    fn short_password() {
        let json = r#"{"username":"bob","password":"short"}"#;
        let res = login(json);
        assert!(res.is_err());
    }
}
