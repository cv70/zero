use crate::error::MemoryError;
use crate::memory::{GlobalSharedMemory, MemoryEntry};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

/// 文件系统内存后端
pub struct FilesystemMemory {
    base_path: PathBuf,
}

impl FilesystemMemory {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn namespace_path(&self, namespace: &str) -> PathBuf {
        self.base_path.join("memory").join(namespace)
    }
}

#[async_trait]
impl GlobalSharedMemory for FilesystemMemory {
    async fn store(&self, namespace: &str, key: &str, value: &str) -> Result<(), MemoryError> {
        let path = self.namespace_path(namespace).join(format!("{}.json", key));
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| MemoryError::StoreFailed(e.to_string()))?;
        std::fs::write(&path, value).map_err(|e| MemoryError::StoreFailed(e.to_string()))?;
        Ok(())
    }

    async fn retrieve(&self, namespace: &str, key: &str) -> Result<Option<String>, MemoryError> {
        let path = self.namespace_path(namespace).join(format!("{}.json", key));
        match std::fs::read_to_string(&path) {
            Ok(v) => Ok(Some(v)),
            Err(e) if std::io::ErrorKind::NotFound == e.kind() => Ok(None),
            Err(e) => Err(MemoryError::RetrieveFailed(e.to_string())),
        }
    }

    async fn search(&self, _namespace: &str, _query: &str, _limit: usize) -> Result<Vec<MemoryEntry>, MemoryError> {
        Ok(Vec::new())
    }

    async fn delete(&self, namespace: &str, key: &str) -> Result<(), MemoryError> {
        let path = self.namespace_path(namespace).join(format!("{}.json", key));
        std::fs::remove_file(&path).map_err(|e| MemoryError::StoreFailed(e.to_string()))?;
        Ok(())
    }

    async fn list_keys(&self, namespace: &str) -> Result<Vec<String>, MemoryError> {
        let dir = self.namespace_path(namespace);
        let mut keys = Vec::new();
        let mut entries = std::fs::read_dir(&dir).map_err(|e| MemoryError::RetrieveFailed(e.to_string()))?;
        while let Some(entry) = entries.next_entry().map_err(|e| MemoryError::RetrieveFailed(e.to_string()))? {
            if let Some(name) = entry.path().file_stem() {
                keys.push(name.to_string());
            }
        }
        Ok(keys)
    }
}
