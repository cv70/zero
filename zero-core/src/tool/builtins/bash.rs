/// Bash execution tool

use crate::tool::r#trait::{Tool, ToolMetadata, ToolContext, ToolOutput};
use crate::error::ToolError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::Command;

/// Bash tool for executing shell commands
pub struct BashTool {
    blocked_patterns: Vec<String>,
}

impl BashTool {
    /// Create a new Bash tool
    pub fn new() -> Self {
        Self {
            blocked_patterns: vec![
                "rm -rf /".to_string(),
                "sudo".to_string(),
                "shutdown".to_string(),
                "reboot".to_string(),
            ],
        }
    }

    /// Check if command is dangerous
    fn is_dangerous(&self, command: &str) -> bool {
        self.blocked_patterns
            .iter()
            .any(|pattern| command.contains(pattern))
    }
}

impl Default for BashTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Tool for BashTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "bash".to_string(),
            description: "Execute shell commands".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "Shell command to execute" }
                },
                "required": ["command"]
            }),
        }
    }

    async fn execute(&self, input: &str, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        #[derive(Deserialize)]
        struct Args {
            command: String,
        }

        let args: Args = serde_json::from_str(input)
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;

        // Security check
        if self.is_dangerous(&args.command) {
            return Err(ToolError::ExecutionFailed(
                "Dangerous command blocked".to_string(),
            ));
        }

        // Execute command
        match Command::new("sh")
            .arg("-c")
            .arg(&args.command)
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let result = format!("{}{}", stdout, stderr);
                Ok(ToolOutput::text(result.chars().take(50000).collect::<String>()))
            }
            Err(e) => Err(ToolError::ExecutionFailed(format!(
                "Command execution failed: {}",
                e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bash_echo() {
        let tool = BashTool::new();
        let input = r#"{"command": "echo hello"}"#;
        let ctx = ToolContext::new("test".to_string());
        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_bash_blocked_command() {
        let tool = BashTool::new();
        let input = r#"{"command": "rm -rf /"}"#;
        let ctx = ToolContext::new("test".to_string());
        let result = tool.execute(input, &ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bash_invalid_json() {
        let tool = BashTool::new();
        let ctx = ToolContext::new("test".to_string());
        let result = tool.execute("invalid json", &ctx).await;
        assert!(result.is_err());
    }
}
