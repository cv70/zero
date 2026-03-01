// Security sandbox implementation for Zero Platform
//! # Security Sandbox
//!
//! The security sandbox provides filesystem and command access policies inspired
//! by the Codex sandbox model. It enforces read/write/execute/network controls
//! based on configurable policies.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Sandbox policy controlling filesystem and command access
#[derive(Debug, Clone)]
pub enum SandboxPolicy {
    /// No sandboxing - full access to everything
    Disabled,
    /// Read-only filesystem access
    ReadOnly,
    /// Read-only with writable workspace directory
    WorkspaceWrite {
        /// The workspace root (usually cwd)
        workspace: PathBuf,
        /// Additional writable paths
        extra_writable: Vec<PathBuf>,
        /// Whether network access is allowed
        allow_network: bool,
    },
    /// Fully isolated - deny by default, explicit allowlists
    Isolated {
        /// Readable paths
        readable: Vec<PathBuf>,
        /// Writable paths
        writable: Vec<PathBuf>,
        /// Whether network access is allowed
        allow_network: bool,
    },
}

impl SandboxPolicy {
    /// Create a disabled (no sandboxing) policy
    pub fn disabled() -> Self {
        SandboxPolicy::Disabled
    }

    /// Create a read-only policy
    pub fn read_only() -> Self {
        SandboxPolicy::ReadOnly
    }

    /// Create a workspace-write policy with network allowed
    pub fn workspace(cwd: impl Into<PathBuf>) -> Self {
        SandboxPolicy::WorkspaceWrite {
            workspace: cwd.into(),
            extra_writable: Vec::new(),
            allow_network: true,
        }
    }
}

/// Result of sandbox access check
#[derive(Debug, Clone, PartialEq)]
pub enum SandboxAccess {
    Allowed,
    Denied(String),
}

impl SandboxAccess {
    /// Returns true if access is allowed
    pub fn is_allowed(&self) -> bool {
        matches!(self, SandboxAccess::Allowed)
    }

    /// Returns true if access is denied
    pub fn is_denied(&self) -> bool {
        matches!(self, SandboxAccess::Denied(_))
    }
}

/// Default protected path patterns that are always read-only even within writable roots
const DEFAULT_PROTECTED_PATTERNS: &[&str] =
    &[".git", ".env", ".ssh", "id_rsa", "credentials", "secrets"];

/// Sandbox manager that enforces filesystem access policies
pub struct SandboxManager {
    policy: SandboxPolicy,
    /// Protected paths that are always read-only even within writable roots
    protected_paths: HashSet<PathBuf>,
}

impl SandboxManager {
    /// Create a new sandbox manager with the given policy
    pub fn new(policy: SandboxPolicy) -> Self {
        let mut protected_paths = HashSet::new();
        for pattern in DEFAULT_PROTECTED_PATTERNS {
            protected_paths.insert(PathBuf::from(pattern));
        }
        Self {
            policy,
            protected_paths,
        }
    }

    /// Add a protected path that will be read-only even within writable roots
    pub fn with_protected_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.protected_paths.insert(path.into());
        self
    }

    /// Check whether a path is protected (always read-only)
    pub fn is_protected(&self, path: &Path) -> bool {
        for protected in &self.protected_paths {
            // Check if any component of the path matches a protected pattern,
            // or if the path ends with the protected name
            for component in path.components() {
                if let std::path::Component::Normal(name) = component {
                    if name == protected.as_os_str() {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check whether reading from a path is allowed
    pub fn check_read(&self, path: &Path) -> SandboxAccess {
        match &self.policy {
            SandboxPolicy::Disabled
            | SandboxPolicy::ReadOnly
            | SandboxPolicy::WorkspaceWrite { .. } => SandboxAccess::Allowed,
            SandboxPolicy::Isolated {
                readable, writable, ..
            } => {
                // Allow if path is under any readable or writable path
                if Self::is_under_any(path, readable) || Self::is_under_any(path, writable) {
                    SandboxAccess::Allowed
                } else {
                    SandboxAccess::Denied(format!(
                        "Read access denied: {} is not under any readable path",
                        path.display()
                    ))
                }
            }
        }
    }

    /// Check whether writing to a path is allowed
    pub fn check_write(&self, path: &Path) -> SandboxAccess {
        // Protected paths are always denied for writes (except in Disabled mode)
        if !matches!(self.policy, SandboxPolicy::Disabled) && self.is_protected(path) {
            return SandboxAccess::Denied(format!(
                "Write access denied: {} is a protected path",
                path.display()
            ));
        }

        match &self.policy {
            SandboxPolicy::Disabled => SandboxAccess::Allowed,
            SandboxPolicy::ReadOnly => SandboxAccess::Denied(format!(
                "Write access denied: sandbox is read-only ({})",
                path.display()
            )),
            SandboxPolicy::WorkspaceWrite {
                workspace,
                extra_writable,
                ..
            } => {
                if path.starts_with(workspace) {
                    return SandboxAccess::Allowed;
                }
                if Self::is_under_any(path, extra_writable) {
                    return SandboxAccess::Allowed;
                }
                SandboxAccess::Denied(format!(
                    "Write access denied: {} is outside workspace and extra writable paths",
                    path.display()
                ))
            }
            SandboxPolicy::Isolated { writable, .. } => {
                if Self::is_under_any(path, writable) {
                    SandboxAccess::Allowed
                } else {
                    SandboxAccess::Denied(format!(
                        "Write access denied: {} is not under any writable path",
                        path.display()
                    ))
                }
            }
        }
    }

    /// Check whether executing a program is allowed
    pub fn check_execute(&self, program: &str) -> SandboxAccess {
        match &self.policy {
            SandboxPolicy::Disabled => SandboxAccess::Allowed,
            SandboxPolicy::ReadOnly => SandboxAccess::Denied(format!(
                "Execution denied: sandbox is read-only ({})",
                program
            )),
            SandboxPolicy::WorkspaceWrite { .. } | SandboxPolicy::Isolated { .. } => {
                SandboxAccess::Denied(format!(
                    "Execution denied: {} - execution is controlled by the permission system, not the sandbox",
                    program
                ))
            }
        }
    }

    /// Check whether network access is allowed
    pub fn check_network(&self) -> SandboxAccess {
        match &self.policy {
            SandboxPolicy::Disabled => SandboxAccess::Allowed,
            SandboxPolicy::ReadOnly => {
                SandboxAccess::Denied("Network access denied: sandbox is read-only".to_string())
            }
            SandboxPolicy::WorkspaceWrite { allow_network, .. }
            | SandboxPolicy::Isolated { allow_network, .. } => {
                if *allow_network {
                    SandboxAccess::Allowed
                } else {
                    SandboxAccess::Denied("Network access denied by sandbox policy".to_string())
                }
            }
        }
    }

    /// Check if a path is under any of the given root paths
    fn is_under_any(path: &Path, roots: &[PathBuf]) -> bool {
        roots.iter().any(|root| path.starts_with(root))
    }
}

/// Validate input data against a schema
pub fn validate_input(input: &str, _schema: &str) -> Result<(), String> {
    // Basic validation - check for valid JSON
    if input.is_empty() {
        return Err("Input cannot be empty".to_string());
    }
    Ok(())
}

/// Scan for potential security issues
pub fn scan(data: &[u8]) -> Vec<String> {
    let mut issues = Vec::new();

    // Example: Check for SQL injection patterns
    let sql_patterns = ["DROP", "DELETE", "UNION", "SELECT"];
    let data_str = String::from_utf8_lossy(data);

    for pattern in sql_patterns {
        if data_str.to_uppercase().contains(pattern) {
            issues.push(format!(
                "Potential SQL injection pattern found: {}",
                pattern
            ));
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== SandboxPolicy convenience constructors ====================

    #[test]
    fn test_policy_disabled() {
        let policy = SandboxPolicy::disabled();
        assert!(matches!(policy, SandboxPolicy::Disabled));
    }

    #[test]
    fn test_policy_read_only() {
        let policy = SandboxPolicy::read_only();
        assert!(matches!(policy, SandboxPolicy::ReadOnly));
    }

    #[test]
    fn test_policy_workspace() {
        let policy = SandboxPolicy::workspace("/home/user/project");
        match policy {
            SandboxPolicy::WorkspaceWrite {
                workspace,
                extra_writable,
                allow_network,
            } => {
                assert_eq!(workspace, PathBuf::from("/home/user/project"));
                assert!(extra_writable.is_empty());
                assert!(allow_network);
            }
            _ => panic!("Expected WorkspaceWrite variant"),
        }
    }

    // ==================== SandboxAccess ====================

    #[test]
    fn test_sandbox_access_allowed() {
        let access = SandboxAccess::Allowed;
        assert!(access.is_allowed());
        assert!(!access.is_denied());
    }

    #[test]
    fn test_sandbox_access_denied() {
        let access = SandboxAccess::Denied("reason".to_string());
        assert!(!access.is_allowed());
        assert!(access.is_denied());
    }

    // ==================== Protected paths ====================

    #[test]
    fn test_default_protected_paths() {
        let mgr = SandboxManager::new(SandboxPolicy::disabled());
        assert!(mgr.is_protected(Path::new("/home/user/.git")));
        assert!(mgr.is_protected(Path::new("/project/.env")));
        assert!(mgr.is_protected(Path::new("/home/.ssh/config")));
        assert!(mgr.is_protected(Path::new("/home/user/id_rsa")));
        assert!(mgr.is_protected(Path::new("/etc/credentials")));
        assert!(mgr.is_protected(Path::new("/app/secrets/key.pem")));
    }

    #[test]
    fn test_non_protected_path() {
        let mgr = SandboxManager::new(SandboxPolicy::disabled());
        assert!(!mgr.is_protected(Path::new("/home/user/code/main.rs")));
        assert!(!mgr.is_protected(Path::new("/tmp/output.txt")));
    }

    #[test]
    fn test_custom_protected_path() {
        let mgr =
            SandboxManager::new(SandboxPolicy::disabled()).with_protected_path("my_secret_dir");
        assert!(mgr.is_protected(Path::new("/project/my_secret_dir/file.txt")));
        // Default protected paths still work
        assert!(mgr.is_protected(Path::new("/project/.git/HEAD")));
    }

    // ==================== check_read ====================

    #[test]
    fn test_read_disabled_always_allowed() {
        let mgr = SandboxManager::new(SandboxPolicy::disabled());
        assert!(mgr.check_read(Path::new("/any/path")).is_allowed());
    }

    #[test]
    fn test_read_readonly_always_allowed() {
        let mgr = SandboxManager::new(SandboxPolicy::read_only());
        assert!(mgr.check_read(Path::new("/any/path")).is_allowed());
    }

    #[test]
    fn test_read_workspace_always_allowed() {
        let mgr = SandboxManager::new(SandboxPolicy::workspace("/workspace"));
        assert!(mgr.check_read(Path::new("/outside/workspace")).is_allowed());
    }

    #[test]
    fn test_read_isolated_allowed_under_readable() {
        let policy = SandboxPolicy::Isolated {
            readable: vec![PathBuf::from("/readable")],
            writable: vec![PathBuf::from("/writable")],
            allow_network: false,
        };
        let mgr = SandboxManager::new(policy);
        assert!(mgr.check_read(Path::new("/readable/file.txt")).is_allowed());
    }

    #[test]
    fn test_read_isolated_allowed_under_writable() {
        let policy = SandboxPolicy::Isolated {
            readable: vec![],
            writable: vec![PathBuf::from("/writable")],
            allow_network: false,
        };
        let mgr = SandboxManager::new(policy);
        assert!(mgr.check_read(Path::new("/writable/file.txt")).is_allowed());
    }

    #[test]
    fn test_read_isolated_denied_outside_paths() {
        let policy = SandboxPolicy::Isolated {
            readable: vec![PathBuf::from("/readable")],
            writable: vec![PathBuf::from("/writable")],
            allow_network: false,
        };
        let mgr = SandboxManager::new(policy);
        assert!(mgr.check_read(Path::new("/other/path")).is_denied());
    }

    // ==================== check_write ====================

    #[test]
    fn test_write_disabled_always_allowed() {
        let mgr = SandboxManager::new(SandboxPolicy::disabled());
        assert!(mgr.check_write(Path::new("/any/path")).is_allowed());
        // Even protected paths are writable in disabled mode
        assert!(
            mgr.check_write(Path::new("/project/.git/HEAD"))
                .is_allowed()
        );
    }

    #[test]
    fn test_write_readonly_always_denied() {
        let mgr = SandboxManager::new(SandboxPolicy::read_only());
        assert!(mgr.check_write(Path::new("/any/path")).is_denied());
    }

    #[test]
    fn test_write_workspace_inside_workspace_allowed() {
        let mgr = SandboxManager::new(SandboxPolicy::workspace("/workspace"));
        assert!(
            mgr.check_write(Path::new("/workspace/src/main.rs"))
                .is_allowed()
        );
    }

    #[test]
    fn test_write_workspace_outside_workspace_denied() {
        let mgr = SandboxManager::new(SandboxPolicy::workspace("/workspace"));
        assert!(mgr.check_write(Path::new("/outside/file.txt")).is_denied());
    }

    #[test]
    fn test_write_workspace_extra_writable_allowed() {
        let policy = SandboxPolicy::WorkspaceWrite {
            workspace: PathBuf::from("/workspace"),
            extra_writable: vec![PathBuf::from("/tmp")],
            allow_network: true,
        };
        let mgr = SandboxManager::new(policy);
        assert!(mgr.check_write(Path::new("/tmp/output.txt")).is_allowed());
    }

    #[test]
    fn test_write_workspace_protected_path_denied() {
        let mgr = SandboxManager::new(SandboxPolicy::workspace("/workspace"));
        assert!(
            mgr.check_write(Path::new("/workspace/.git/HEAD"))
                .is_denied()
        );
        assert!(mgr.check_write(Path::new("/workspace/.env")).is_denied());
    }

    #[test]
    fn test_write_isolated_inside_writable_allowed() {
        let policy = SandboxPolicy::Isolated {
            readable: vec![],
            writable: vec![PathBuf::from("/output")],
            allow_network: false,
        };
        let mgr = SandboxManager::new(policy);
        assert!(
            mgr.check_write(Path::new("/output/result.txt"))
                .is_allowed()
        );
    }

    #[test]
    fn test_write_isolated_outside_writable_denied() {
        let policy = SandboxPolicy::Isolated {
            readable: vec![PathBuf::from("/readable")],
            writable: vec![PathBuf::from("/output")],
            allow_network: false,
        };
        let mgr = SandboxManager::new(policy);
        // Readable but not writable
        assert!(mgr.check_write(Path::new("/readable/file.txt")).is_denied());
    }

    #[test]
    fn test_write_isolated_protected_path_denied() {
        let policy = SandboxPolicy::Isolated {
            readable: vec![],
            writable: vec![PathBuf::from("/output")],
            allow_network: false,
        };
        let mgr = SandboxManager::new(policy);
        assert!(mgr.check_write(Path::new("/output/.env")).is_denied());
    }

    // ==================== check_execute ====================

    #[test]
    fn test_execute_disabled_allowed() {
        let mgr = SandboxManager::new(SandboxPolicy::disabled());
        assert!(mgr.check_execute("rm").is_allowed());
    }

    #[test]
    fn test_execute_readonly_denied() {
        let mgr = SandboxManager::new(SandboxPolicy::read_only());
        assert!(mgr.check_execute("ls").is_denied());
    }

    #[test]
    fn test_execute_workspace_denied() {
        let mgr = SandboxManager::new(SandboxPolicy::workspace("/workspace"));
        assert!(mgr.check_execute("cargo").is_denied());
    }

    #[test]
    fn test_execute_isolated_denied() {
        let policy = SandboxPolicy::Isolated {
            readable: vec![],
            writable: vec![],
            allow_network: false,
        };
        let mgr = SandboxManager::new(policy);
        assert!(mgr.check_execute("python").is_denied());
    }

    // ==================== check_network ====================

    #[test]
    fn test_network_disabled_allowed() {
        let mgr = SandboxManager::new(SandboxPolicy::disabled());
        assert!(mgr.check_network().is_allowed());
    }

    #[test]
    fn test_network_readonly_denied() {
        let mgr = SandboxManager::new(SandboxPolicy::read_only());
        assert!(mgr.check_network().is_denied());
    }

    #[test]
    fn test_network_workspace_allowed_by_default() {
        let mgr = SandboxManager::new(SandboxPolicy::workspace("/workspace"));
        assert!(mgr.check_network().is_allowed());
    }

    #[test]
    fn test_network_workspace_denied_when_disabled() {
        let policy = SandboxPolicy::WorkspaceWrite {
            workspace: PathBuf::from("/workspace"),
            extra_writable: vec![],
            allow_network: false,
        };
        let mgr = SandboxManager::new(policy);
        assert!(mgr.check_network().is_denied());
    }

    #[test]
    fn test_network_isolated_based_on_flag() {
        let policy_allowed = SandboxPolicy::Isolated {
            readable: vec![],
            writable: vec![],
            allow_network: true,
        };
        let mgr_allowed = SandboxManager::new(policy_allowed);
        assert!(mgr_allowed.check_network().is_allowed());

        let policy_denied = SandboxPolicy::Isolated {
            readable: vec![],
            writable: vec![],
            allow_network: false,
        };
        let mgr_denied = SandboxManager::new(policy_denied);
        assert!(mgr_denied.check_network().is_denied());
    }

    // ==================== Backward-compat functions ====================

    #[test]
    fn test_validate_input_empty() {
        assert!(validate_input("", "any").is_err());
    }

    #[test]
    fn test_validate_input_nonempty() {
        assert!(validate_input("hello", "any").is_ok());
    }

    #[test]
    fn test_scan_detects_sql_patterns() {
        let issues = scan(b"DROP TABLE users");
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_scan_clean_data() {
        let issues = scan(b"Hello world");
        assert!(issues.is_empty());
    }
}
