// Permission model for Zero Platform
//! # Permissions
//!
//! Implements a permission system governing what operations the agent is
//! allowed to perform. The model distinguishes three levels: read-only,
//! standard (scoped writes), and full access.

use super::command_safety::{classify_command, CommandSafety};
use std::path::Path;

/// Permission level for tool execution
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionLevel {
    /// Read-only operations allowed (ls, cat, grep, etc.)
    ReadOnly,
    /// Standard operations (file read/write within allowed dirs, safe commands)
    Standard,
    /// Full access (all commands, all directories)
    Full,
}

/// Policy governing what operations are allowed
#[derive(Debug, Clone)]
pub struct PermissionPolicy {
    pub level: PermissionLevel,
    /// Directories the agent may write to (only relevant for Standard level)
    pub writable_dirs: Vec<String>,
    /// Commands that are always blocked
    pub blocked_commands: Vec<String>,
    /// Whether dangerous commands require explicit confirmation
    pub require_confirmation: bool,
}

/// Result of a permission check
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionCheck {
    /// Operation is allowed
    Allowed,
    /// Operation needs user confirmation (with reason)
    NeedsConfirmation(String),
    /// Operation is denied (with reason)
    Denied(String),
}

/// Guard that checks permissions before tool execution
pub struct PermissionGuard {
    policy: PermissionPolicy,
}

// ── Default blocked commands ────────────────────────────────────────────────

/// Commands that are blocked by default in the Standard policy.
const DEFAULT_BLOCKED_COMMANDS: &[&str] = &[
    "mkfs", "dd", "sudo", "su", "shutdown", "reboot", "halt", "poweroff", "init",
    "systemctl", "service",
];

// ── PermissionPolicy constructors ───────────────────────────────────────────

impl Default for PermissionPolicy {
    /// Standard level, current working directory writable, common dangerous
    /// commands blocked, confirmation required for dangerous commands.
    fn default() -> Self {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string());

        Self {
            level: PermissionLevel::Standard,
            writable_dirs: vec![cwd],
            blocked_commands: DEFAULT_BLOCKED_COMMANDS.iter().map(|s| s.to_string()).collect(),
            require_confirmation: true,
        }
    }
}

impl PermissionPolicy {
    /// Create a read-only policy.
    pub fn read_only() -> Self {
        Self {
            level: PermissionLevel::ReadOnly,
            writable_dirs: Vec::new(),
            blocked_commands: DEFAULT_BLOCKED_COMMANDS.iter().map(|s| s.to_string()).collect(),
            require_confirmation: true,
        }
    }

    /// Create a full-access policy.
    pub fn full_access() -> Self {
        Self {
            level: PermissionLevel::Full,
            writable_dirs: Vec::new(), // not relevant for Full
            blocked_commands: DEFAULT_BLOCKED_COMMANDS.iter().map(|s| s.to_string()).collect(),
            require_confirmation: false,
        }
    }
}

// ── PermissionGuard ─────────────────────────────────────────────────────────

impl PermissionGuard {
    /// Create a new guard with the given policy.
    pub fn new(policy: PermissionPolicy) -> Self {
        Self { policy }
    }

    /// Check whether executing `command` is permitted.
    pub fn check_command(&self, command: &str) -> PermissionCheck {
        let command = command.trim();
        if command.is_empty() {
            return PermissionCheck::Allowed;
        }

        // Extract the base command name (first token).
        let base_cmd = command
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string();

        // Always check the blocked-commands list.
        if self.is_blocked(&base_cmd) {
            return PermissionCheck::Denied(format!(
                "command '{}' is in the blocked commands list",
                base_cmd,
            ));
        }

        match &self.policy.level {
            PermissionLevel::ReadOnly => {
                // Use the command safety classifier; only allow Safe commands.
                match classify_command(command) {
                    CommandSafety::Safe => PermissionCheck::Allowed,
                    CommandSafety::Dangerous(reason) => {
                        PermissionCheck::Denied(format!("read-only mode: {}", reason))
                    }
                    CommandSafety::Unknown => PermissionCheck::Denied(
                        "read-only mode: command is not in the safe whitelist".to_string(),
                    ),
                }
            }
            PermissionLevel::Standard => {
                // Classify safety.
                match classify_command(command) {
                    CommandSafety::Safe => PermissionCheck::Allowed,
                    CommandSafety::Dangerous(reason) => {
                        if self.policy.require_confirmation {
                            PermissionCheck::NeedsConfirmation(reason)
                        } else {
                            PermissionCheck::Denied(reason)
                        }
                    }
                    CommandSafety::Unknown => {
                        if self.policy.require_confirmation {
                            PermissionCheck::NeedsConfirmation(
                                "command safety is unknown; please confirm".to_string(),
                            )
                        } else {
                            PermissionCheck::Allowed
                        }
                    }
                }
            }
            PermissionLevel::Full => {
                // Full access -- allow everything not explicitly blocked
                // (blocked check was already done above).
                PermissionCheck::Allowed
            }
        }
    }

    /// Check whether writing to `path` is permitted.
    pub fn check_file_write(&self, path: &str) -> PermissionCheck {
        match &self.policy.level {
            PermissionLevel::ReadOnly => {
                PermissionCheck::Denied("read-only mode: file writes are not allowed".to_string())
            }
            PermissionLevel::Standard => {
                // Normalise path for comparison.
                let normalised = normalise_path(path);

                for dir in &self.policy.writable_dirs {
                    let normalised_dir = normalise_path(dir);
                    if normalised.starts_with(&normalised_dir) {
                        return PermissionCheck::Allowed;
                    }
                }

                PermissionCheck::Denied(format!(
                    "path '{}' is outside writable directories",
                    path,
                ))
            }
            PermissionLevel::Full => PermissionCheck::Allowed,
        }
    }

    /// Check whether a tool invocation is permitted.
    ///
    /// Routes to the appropriate checker based on tool name:
    /// - `"bash"` -> `check_command` (extracts command from JSON `arguments`)
    /// - `"write_file"` / `"edit_file"` -> `check_file_write` (extracts path from JSON)
    /// - `"read_file"` -> always allowed
    pub fn check_tool(&self, tool_name: &str, arguments: &str) -> PermissionCheck {
        match tool_name {
            "bash" => {
                let command = extract_json_field(arguments, "command")
                    .unwrap_or_else(|| arguments.to_string());
                self.check_command(&command)
            }
            "write_file" | "edit_file" => {
                let path = extract_json_field(arguments, "path")
                    .or_else(|| extract_json_field(arguments, "file_path"))
                    .unwrap_or_default();
                self.check_file_write(&path)
            }
            "read_file" => PermissionCheck::Allowed,
            _ => {
                // Unknown tool: depend on permission level.
                match &self.policy.level {
                    PermissionLevel::ReadOnly => PermissionCheck::Denied(format!(
                        "read-only mode: unknown tool '{}' is not allowed",
                        tool_name,
                    )),
                    PermissionLevel::Standard => {
                        if self.policy.require_confirmation {
                            PermissionCheck::NeedsConfirmation(format!(
                                "unknown tool '{}'; please confirm",
                                tool_name,
                            ))
                        } else {
                            PermissionCheck::Allowed
                        }
                    }
                    PermissionLevel::Full => PermissionCheck::Allowed,
                }
            }
        }
    }

    // ── helpers ─────────────────────────────────────────────────────────

    /// Check whether `cmd` matches any entry in the blocked commands list.
    fn is_blocked(&self, cmd: &str) -> bool {
        self.policy.blocked_commands.iter().any(|b| b == cmd)
    }
}

/// Very small JSON field extractor.  Looks for `"field": "value"` and returns
/// the value.  This avoids pulling in serde_json for a simple extraction.
fn extract_json_field(json: &str, field: &str) -> Option<String> {
    // Try a simple pattern: "field": "value" or "field":"value"
    let patterns = [
        format!("\"{}\":\"", field),
        format!("\"{}\": \"", field),
        format!("\"{}\" : \"", field),
        format!("\"{}\" :\"", field),
    ];

    for pattern in &patterns {
        if let Some(start) = json.find(pattern.as_str()) {
            let value_start = start + pattern.len();
            let rest = &json[value_start..];
            // Find the closing quote, handling escaped quotes.
            let mut end = 0;
            let mut escape = false;
            for ch in rest.chars() {
                if escape {
                    escape = false;
                    end += ch.len_utf8();
                    continue;
                }
                if ch == '\\' {
                    escape = true;
                    end += 1;
                    continue;
                }
                if ch == '"' {
                    break;
                }
                end += ch.len_utf8();
            }
            return Some(rest[..end].to_string());
        }
    }

    None
}

/// Normalise a path for prefix comparison.  Ensures a trailing separator so
/// that `/home/user` doesn't match `/home/username`.
fn normalise_path(path: &str) -> String {
    let mut p = path.to_string();
    if !p.ends_with(std::path::MAIN_SEPARATOR) {
        p.push(std::path::MAIN_SEPARATOR);
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── PermissionPolicy constructors ───────────────────────────────────

    #[test]
    fn test_default_policy_is_standard() {
        let policy = PermissionPolicy::default();
        assert_eq!(policy.level, PermissionLevel::Standard);
        assert!(!policy.writable_dirs.is_empty());
        assert!(!policy.blocked_commands.is_empty());
        assert!(policy.require_confirmation);
    }

    #[test]
    fn test_read_only_policy() {
        let policy = PermissionPolicy::read_only();
        assert_eq!(policy.level, PermissionLevel::ReadOnly);
        assert!(policy.writable_dirs.is_empty());
    }

    #[test]
    fn test_full_access_policy() {
        let policy = PermissionPolicy::full_access();
        assert_eq!(policy.level, PermissionLevel::Full);
        assert!(!policy.require_confirmation);
    }

    // ── check_command: ReadOnly ─────────────────────────────────────────

    #[test]
    fn test_readonly_allows_safe_commands() {
        let guard = PermissionGuard::new(PermissionPolicy::read_only());
        assert_eq!(guard.check_command("ls -la"), PermissionCheck::Allowed);
        assert_eq!(guard.check_command("cat foo.txt"), PermissionCheck::Allowed);
        assert_eq!(guard.check_command("grep pattern file"), PermissionCheck::Allowed);
    }

    #[test]
    fn test_readonly_denies_unknown_commands() {
        let guard = PermissionGuard::new(PermissionPolicy::read_only());
        assert!(matches!(
            guard.check_command("python3 script.py"),
            PermissionCheck::Denied(_)
        ));
    }

    #[test]
    fn test_readonly_denies_dangerous_commands() {
        let guard = PermissionGuard::new(PermissionPolicy::read_only());
        assert!(matches!(
            guard.check_command("rm -rf /"),
            PermissionCheck::Denied(_)
        ));
    }

    // ── check_command: Standard ─────────────────────────────────────────

    #[test]
    fn test_standard_allows_safe_commands() {
        let guard = PermissionGuard::new(PermissionPolicy::default());
        assert_eq!(guard.check_command("ls"), PermissionCheck::Allowed);
        assert_eq!(guard.check_command("git status"), PermissionCheck::Allowed);
    }

    #[test]
    fn test_standard_needs_confirmation_for_unknown() {
        let guard = PermissionGuard::new(PermissionPolicy::default());
        assert!(matches!(
            guard.check_command("cargo build"),
            PermissionCheck::NeedsConfirmation(_)
        ));
    }

    #[test]
    fn test_standard_needs_confirmation_for_dangerous() {
        let guard = PermissionGuard::new(PermissionPolicy::default());
        assert!(matches!(
            guard.check_command("rm -rf /tmp/test"),
            PermissionCheck::NeedsConfirmation(_)
        ));
    }

    #[test]
    fn test_standard_blocks_blocked_commands() {
        let guard = PermissionGuard::new(PermissionPolicy::default());
        assert!(matches!(
            guard.check_command("sudo apt install foo"),
            PermissionCheck::Denied(_)
        ));
    }

    // ── check_command: Full ─────────────────────────────────────────────

    #[test]
    fn test_full_allows_everything() {
        let guard = PermissionGuard::new(PermissionPolicy::full_access());
        assert_eq!(guard.check_command("cargo build"), PermissionCheck::Allowed);
        assert_eq!(guard.check_command("rm -rf /tmp/test"), PermissionCheck::Allowed);
    }

    #[test]
    fn test_full_still_blocks_blocked_commands() {
        let guard = PermissionGuard::new(PermissionPolicy::full_access());
        assert!(matches!(
            guard.check_command("sudo rm -rf /"),
            PermissionCheck::Denied(_)
        ));
    }

    // ── check_file_write ────────────────────────────────────────────────

    #[test]
    fn test_readonly_denies_file_write() {
        let guard = PermissionGuard::new(PermissionPolicy::read_only());
        assert!(matches!(
            guard.check_file_write("/home/user/file.txt"),
            PermissionCheck::Denied(_)
        ));
    }

    #[test]
    fn test_standard_allows_write_in_writable_dir() {
        let policy = PermissionPolicy {
            level: PermissionLevel::Standard,
            writable_dirs: vec!["/home/user/project".to_string()],
            blocked_commands: Vec::new(),
            require_confirmation: true,
        };
        let guard = PermissionGuard::new(policy);
        assert_eq!(
            guard.check_file_write("/home/user/project/src/main.rs"),
            PermissionCheck::Allowed
        );
    }

    #[test]
    fn test_standard_denies_write_outside_writable_dir() {
        let policy = PermissionPolicy {
            level: PermissionLevel::Standard,
            writable_dirs: vec!["/home/user/project".to_string()],
            blocked_commands: Vec::new(),
            require_confirmation: true,
        };
        let guard = PermissionGuard::new(policy);
        assert!(matches!(
            guard.check_file_write("/etc/passwd"),
            PermissionCheck::Denied(_)
        ));
    }

    #[test]
    fn test_standard_denies_write_to_similarly_named_dir() {
        // /home/user/project should NOT match /home/user/project-other
        let policy = PermissionPolicy {
            level: PermissionLevel::Standard,
            writable_dirs: vec!["/home/user/project".to_string()],
            blocked_commands: Vec::new(),
            require_confirmation: true,
        };
        let guard = PermissionGuard::new(policy);
        assert!(matches!(
            guard.check_file_write("/home/user/project-other/file.txt"),
            PermissionCheck::Denied(_)
        ));
    }

    #[test]
    fn test_full_allows_any_file_write() {
        let guard = PermissionGuard::new(PermissionPolicy::full_access());
        assert_eq!(
            guard.check_file_write("/etc/passwd"),
            PermissionCheck::Allowed
        );
    }

    // ── check_tool ──────────────────────────────────────────────────────

    #[test]
    fn test_tool_bash_routes_to_check_command() {
        let guard = PermissionGuard::new(PermissionPolicy::read_only());
        let args = r#"{"command": "ls -la"}"#;
        assert_eq!(guard.check_tool("bash", args), PermissionCheck::Allowed);
    }

    #[test]
    fn test_tool_bash_denies_dangerous_in_readonly() {
        let guard = PermissionGuard::new(PermissionPolicy::read_only());
        let args = r#"{"command": "rm -rf /"}"#;
        assert!(matches!(
            guard.check_tool("bash", args),
            PermissionCheck::Denied(_)
        ));
    }

    #[test]
    fn test_tool_write_file_routes_to_check_file_write() {
        let policy = PermissionPolicy {
            level: PermissionLevel::Standard,
            writable_dirs: vec!["/home/user/project".to_string()],
            blocked_commands: Vec::new(),
            require_confirmation: true,
        };
        let guard = PermissionGuard::new(policy);
        let args = r#"{"path": "/home/user/project/src/main.rs", "content": "fn main() {}"}"#;
        assert_eq!(
            guard.check_tool("write_file", args),
            PermissionCheck::Allowed
        );
    }

    #[test]
    fn test_tool_edit_file_routes_to_check_file_write() {
        let guard = PermissionGuard::new(PermissionPolicy::read_only());
        let args = r#"{"file_path": "/home/user/file.rs", "old": "x", "new": "y"}"#;
        assert!(matches!(
            guard.check_tool("edit_file", args),
            PermissionCheck::Denied(_)
        ));
    }

    #[test]
    fn test_tool_read_file_always_allowed() {
        let guard = PermissionGuard::new(PermissionPolicy::read_only());
        let args = r#"{"path": "/etc/passwd"}"#;
        assert_eq!(guard.check_tool("read_file", args), PermissionCheck::Allowed);
    }

    #[test]
    fn test_tool_unknown_denied_in_readonly() {
        let guard = PermissionGuard::new(PermissionPolicy::read_only());
        assert!(matches!(
            guard.check_tool("web_search", "{}"),
            PermissionCheck::Denied(_)
        ));
    }

    #[test]
    fn test_tool_unknown_needs_confirmation_in_standard() {
        let guard = PermissionGuard::new(PermissionPolicy::default());
        assert!(matches!(
            guard.check_tool("web_search", "{}"),
            PermissionCheck::NeedsConfirmation(_)
        ));
    }

    #[test]
    fn test_tool_unknown_allowed_in_full() {
        let guard = PermissionGuard::new(PermissionPolicy::full_access());
        assert_eq!(
            guard.check_tool("web_search", "{}"),
            PermissionCheck::Allowed
        );
    }

    // ── extract_json_field ──────────────────────────────────────────────

    #[test]
    fn test_extract_json_field_basic() {
        let json = r#"{"command": "ls -la", "timeout": 30}"#;
        assert_eq!(
            extract_json_field(json, "command"),
            Some("ls -la".to_string())
        );
    }

    #[test]
    fn test_extract_json_field_with_escaped_quote() {
        let json = r#"{"path": "file\"name.txt"}"#;
        assert_eq!(
            extract_json_field(json, "path"),
            Some("file\"name.txt".to_string())
        );
    }

    #[test]
    fn test_extract_json_field_missing() {
        let json = r#"{"other": "value"}"#;
        assert_eq!(extract_json_field(json, "command"), None);
    }

    // ── empty command ───────────────────────────────────────────────────

    #[test]
    fn test_empty_command_always_allowed() {
        let guard = PermissionGuard::new(PermissionPolicy::read_only());
        assert_eq!(guard.check_command(""), PermissionCheck::Allowed);
    }
}
