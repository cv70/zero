use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("internal error: {0}")]
    InternalError(String),
}

impl From<std::io::Error> for SecurityError {
    fn from(e: std::io::Error) -> Self {
        SecurityError::InternalError(e.to_string())
    }
}
impl From<serde_json::Error> for SecurityError {
    fn from(e: serde_json::Error) -> Self {
        SecurityError::InvalidInput(e.to_string())
    }
}
