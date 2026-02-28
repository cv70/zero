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
    async fn search(&self, namespace: &str, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, MemoryError> {
        // Default implementation: gather entries in the namespace and delegate to lightweight search helper
        let keys = self.list_keys(namespace).await?;
        let mut entries: Vec<MemoryEntry> = Vec::with_capacity(keys.len());
        for k in keys {
            if let Some(val) = self.retrieve(namespace, &k).await? {
                entries.push(MemoryEntry { key: k, value: val, timestamp: 0, metadata: HashMap::new() });
            }
        }
        let results = crate::memory::search::search_entries(&entries, query);
        Ok(results.into_iter().take(limit).collect())
    }

    /// 删除记忆
    async fn delete(&self, namespace: &str, key: &str) -> Result<(), MemoryError>;

    /// 列出所有 keys
    async fn list_keys(&self, namespace: &str) -> Result<Vec<String>, MemoryError>;
}


/// Search configuration for memory search
#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub limit: usize,
}

/// Memory search results
#[derive(Debug, Clone)]
pub struct SearchResults {
    pub entries: Vec<MemoryEntry>,
    pub total: usize,
}

/// Memory search implementation
#[derive(Debug, Clone)]
pub struct MemorySearcher {
    entries: Vec<MemoryEntry>,
}

impl MemorySearcher {
    /// Creates a new MemorySearcher instance
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add entries to be searched
    pub fn add_entries(&mut self, entries: Vec<MemoryEntry>) {
        self.entries = entries;
    }

    /// Search through memory entries with query
    pub fn search(&self, query: &str) -> Vec<MemoryEntry> {
        if query.is_empty() {
            return self.entries.clone();
        }

        self.entries
            .iter()
            .filter(|entry| {
                entry.key.contains(query)
                    || entry.value.contains(query)
                    || entry.metadata.values().any(|v| v.contains(query))
            })
            .cloned()
            .collect()
    }
}

impl Default for MemorySearcher {
    fn default() -> Self {
        Self::new()
    }
}
