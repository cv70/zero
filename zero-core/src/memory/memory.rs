use crate::error::MemoryError;
use async_trait::async_trait;
use std::collections::HashMap;

/// Memory 条目
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

/// 全局共享 Memory Trait
#[async_trait]
pub trait GlobalSharedMemory: Send + Sync {
    /// 存储记忆
    async fn store(&self, namespace: &str, key: &str, value: &str) -> Result<(), MemoryError>;

    /// 检索记忆
    async fn retrieve(&self, namespace: &str, key: &str) -> Result<Option<String>, MemoryError>;

    /// 搜索记忆
    async fn search(&self, namespace: &str, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, MemoryError>;

    /// 删除记忆
    async fn delete(&self, namespace: &str, key: &str) -> Result<(), MemoryError>;

    /// 列出所有 keys
    async fn list_keys(&self, namespace: &str) -> Result<Vec<String>, MemoryError>;
}
