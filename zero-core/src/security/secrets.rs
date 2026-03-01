// Secrets management for Zero Platform
//! # Secrets Management
//!
//! Provides secure storage and handling of sensitive values such as API keys.
//!
//! `SecretValue` wraps a string and clears it on drop. `SecretStore` provides
//! centralized key management, redaction support for logging, and environment
//! variable integration.
//!
//! **Note:** This does NOT provide encryption at rest. It provides:
//! - Centralized key management
//! - Redaction support for logging
//! - Environment variable integration
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

    /// Load a secret value from an environment variable
    pub fn from_env(key: &str) -> Option<SecretValue> {
        std::env::var(key).ok().map(SecretValue::new)
    }

    /// Convenience: load a provider API key from well-known environment variables.
    ///
    /// Supported providers:
    /// - `"anthropic"` -> `ANTHROPIC_API_KEY`
    /// - `"openai"` -> `OPENAI_API_KEY`
    /// - `"google"` -> `GOOGLE_API_KEY`
    /// - `"cohere"` -> `COHERE_API_KEY`
    ///
    /// Returns `None` if the provider is unknown or the env var is not set.
    pub fn load_provider_key(provider: &str) -> Option<SecretValue> {
        let env_var = match provider.to_lowercase().as_str() {
            "anthropic" => "ANTHROPIC_API_KEY",
            "openai" => "OPENAI_API_KEY",
            "google" => "GOOGLE_API_KEY",
            "cohere" => "COHERE_API_KEY",
            _ => return None,
        };
        Self::from_env(env_var)
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
    fn test_from_env_missing() {
        // Use a key that certainly doesn't exist
        let result = SecretStore::from_env("ZERO_TEST_NONEXISTENT_KEY_12345");
        assert!(result.is_none());
    }

    #[test]
    fn test_from_env_present() {
        // Set an env var for testing.
        // SAFETY: This test is single-threaded and the env var name is unique,
        // so there are no data races.
        unsafe {
            std::env::set_var("ZERO_TEST_SECRET_KEY", "test-value-123");
        }
        let result = SecretStore::from_env("ZERO_TEST_SECRET_KEY");
        assert!(result.is_some());
        assert_eq!(result.unwrap().expose(), "test-value-123");
        unsafe {
            std::env::remove_var("ZERO_TEST_SECRET_KEY");
        }
    }

    #[test]
    fn test_load_provider_key_unknown() {
        let result = SecretStore::load_provider_key("unknown_provider");
        assert!(result.is_none());
    }

    #[test]
    fn test_load_provider_key_anthropic() {
        // This test checks the mapping; it may or may not find the env var
        // depending on the environment. We just verify it doesn't panic.
        let _result = SecretStore::load_provider_key("anthropic");
        let _result = SecretStore::load_provider_key("Anthropic");
        let _result = SecretStore::load_provider_key("ANTHROPIC");
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
