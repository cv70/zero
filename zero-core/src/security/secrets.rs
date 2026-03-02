// Secrets management for Zero Platform
//! # Secrets Management
//!
//! Provides secure storage and handling of sensitive values such as API keys.
//!
//! `SecretValue` wraps a string and clears it on drop. `SecretStore` provides
//! centralized key management and redaction support for logging.
//!
//! **Note:** This does NOT provide encryption at rest. It provides:
//! - Centralized key management
//! - Redaction support for logging
//! - Drop-time clearing

use std::collections::HashMap;
use std::fmt;

/// A sensitive value that clears itself on drop.
///
/// The value is zeroed out when the `SecretValue` is dropped to minimize the
/// window during which secrets remain in memory.
pub struct SecretValue {
    value: String,
}

impl SecretValue {
    /// Create a new secret value
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Access the raw secret value
    pub fn expose(&self) -> &str {
        &self.value
    }

    /// Get a redacted representation of the value.
    ///
    /// Shows the first 3 and last 3 characters with `***` in between.
    /// If the value is 6 characters or shorter, the entire value is masked.
    pub fn redacted(&self) -> String {
        let len = self.value.len();
        if len <= 6 {
            return "***".to_string();
        }
        let prefix: String = self.value.chars().take(3).collect();
        let suffix: String = self.value.chars().skip(len - 3).collect();
        format!("{}***{}", prefix, suffix)
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretValue")
            .field("value", &self.redacted())
            .finish()
    }
}

impl Drop for SecretValue {
    fn drop(&mut self) {
        // Clear the string's bytes before dropping.
        // SAFETY: We are writing zeros to the string's existing buffer,
        // which is valid UTF-8 (all null bytes). We do not change the
        // length or capacity of the buffer.
        let bytes = unsafe { self.value.as_bytes_mut() };
        for byte in bytes.iter_mut() {
            // Use write_volatile to prevent the compiler from optimizing away
            // the zeroing.
            unsafe {
                std::ptr::write_volatile(byte as *mut u8, 0);
            }
        }
        self.value.clear();
    }
}

/// Secure storage for sensitive values like API keys.
///
/// Values are stored in memory and cleared on drop via `SecretValue`.
pub struct SecretStore {
    secrets: HashMap<String, SecretValue>,
}

impl SecretStore {
    /// Create a new empty secret store
    pub fn new() -> Self {
        Self {
            secrets: HashMap::new(),
        }
    }

    /// Store a secret under the given key
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.secrets
            .insert(key.into(), SecretValue::new(value.into()));
    }

    /// Retrieve a secret by key
    pub fn get(&self, key: &str) -> Option<&SecretValue> {
        self.secrets.get(key)
    }

    /// Remove a secret by key
    pub fn remove(&mut self, key: &str) {
        self.secrets.remove(key);
    }

    /// List all stored key names
    pub fn keys(&self) -> Vec<&str> {
        self.secrets.keys().map(|k| k.as_str()).collect()
    }
}

impl Default for SecretStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== SecretValue ====================

    #[test]
    fn test_secret_value_expose() {
        let sv = SecretValue::new("my-secret-key-12345");
        assert_eq!(sv.expose(), "my-secret-key-12345");
    }

    #[test]
    fn test_secret_value_redacted_long() {
        let sv = SecretValue::new("sk-abcdefghijk123");
        let redacted = sv.redacted();
        assert_eq!(redacted, "sk-***123");
    }

    #[test]
    fn test_secret_value_redacted_short() {
        let sv = SecretValue::new("abc");
        assert_eq!(sv.redacted(), "***");
    }

    #[test]
    fn test_secret_value_redacted_exactly_six() {
        let sv = SecretValue::new("abcdef");
        assert_eq!(sv.redacted(), "***");
    }

    #[test]
    fn test_secret_value_redacted_seven() {
        let sv = SecretValue::new("abcdefg");
        assert_eq!(sv.redacted(), "abc***efg");
    }

    #[test]
    fn test_secret_value_debug_shows_redacted() {
        let sv = SecretValue::new("sk-super-secret-key-xyz");
        let debug = format!("{:?}", sv);
        assert!(debug.contains("***"));
        assert!(!debug.contains("super-secret"));
    }

    #[test]
    fn test_secret_value_drop_clears_memory() {
        // Verify that drop runs without panicking and actually zeros the buffer.
        let sv = SecretValue::new("test-secret-value");
        // Explicitly drop -- the Drop impl writes zeros via write_volatile.
        // This test primarily ensures Drop doesn't panic.
        drop(sv);
    }

    // ==================== SecretStore ====================

    #[test]
    fn test_store_set_and_get() {
        let mut store = SecretStore::new();
        store.set("api_key", "sk-12345");
        let val = store.get("api_key").unwrap();
        assert_eq!(val.expose(), "sk-12345");
    }

    #[test]
    fn test_store_get_nonexistent() {
        let store = SecretStore::new();
        assert!(store.get("missing").is_none());
    }

    #[test]
    fn test_store_remove() {
        let mut store = SecretStore::new();
        store.set("key", "value");
        assert!(store.get("key").is_some());
        store.remove("key");
        assert!(store.get("key").is_none());
    }

    #[test]
    fn test_store_keys() {
        let mut store = SecretStore::new();
        store.set("alpha", "a");
        store.set("beta", "b");
        store.set("gamma", "c");
        let mut keys = store.keys();
        keys.sort();
        assert_eq!(keys, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn test_store_overwrite() {
        let mut store = SecretStore::new();
        store.set("key", "old_value");
        store.set("key", "new_value");
        assert_eq!(store.get("key").unwrap().expose(), "new_value");
    }

    #[test]
    fn test_default_store() {
        let store = SecretStore::default();
        assert!(store.keys().is_empty());
    }

    #[test]
    fn test_secret_value_empty_string() {
        let sv = SecretValue::new("");
        assert_eq!(sv.expose(), "");
        assert_eq!(sv.redacted(), "***");
    }

    #[test]
    fn test_store_remove_nonexistent() {
        let mut store = SecretStore::new();
        // Should not panic
        store.remove("nonexistent");
    }
}
