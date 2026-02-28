use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub id: String,
    pub payload: T,
    pub meta: HashMap<String, String>,
    pub created_at: i64,
}
