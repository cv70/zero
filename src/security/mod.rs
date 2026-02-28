pub mod errors;
pub mod monitor;
pub mod scan;
pub mod validator;

pub use errors::SecurityError;

// Simple JSON-based input validation helper
pub fn validate_json<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, SecurityError> {
    serde_json::from_str(s).map_err(|e| SecurityError::InvalidInput(e.to_string()))
}
