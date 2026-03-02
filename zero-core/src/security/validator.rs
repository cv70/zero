// Security validator implementation for Zero Platform
//! # Security Validator
//!
//! The security validator provides functions for validating input data.
//! It helps prevent security issues by checking data before processing.

use super::command_safety::CommandSafety;
use std::collections::HashSet;
use tracing::{info, warn};

use crate::security::command_safety::classify_command;

/// Validator for input data
pub struct InputValidator {
    allowed_patterns: HashSet<String>,
    max_length: usize,
}

impl InputValidator {
    /// Create a new input validator
    pub fn new() -> Self {
        Self {
            allowed_patterns: HashSet::new(),
            max_length: 1024,
        }
    }

    /// Add an allowed pattern
    pub fn add_allowed_pattern(&mut self, pattern: &str) {
        self.allowed_patterns.insert(pattern.to_string());
    }

    /// Set maximum input length
    pub fn set_max_length(&mut self, length: usize) {
        self.max_length = length;
    }

    /// Validate input string
    pub fn validate(&self, input: &str) -> Result<(), String> {
        if input.len() > self.max_length {
            return Err(format!("Input too long (max: {})", self.max_length));
        }
        Ok(())
    }

    /// Filter input to remove disallowed patterns
    pub fn filter(&self, input: &str) -> String {
        let classified = classify_command(input);
        match classified {
            CommandSafety::Dangerous(reason) => {
                warn!("Dangerous command detected: {}", reason);
                String::new()
            }
            CommandSafety::Safe => input.to_string(),
            CommandSafety::Unknown => {
                info!("Unknown command: {}", input);
                input.to_string()
            }
        }
    }
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}
