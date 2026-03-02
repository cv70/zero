#[derive(Debug, Clone)]
pub enum DomainError {
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    InvalidState(String),
    Internal(String),
}
