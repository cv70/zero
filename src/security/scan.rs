use super::SecurityError;
use std::process::Command;

// Run a vulnerability scan using cargo-audit (if available at runtime).
pub fn run_vulnerability_scan() -> Result<String, SecurityError> {
    let output = Command::new("cargo").arg("audit").output();
    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(String::from_utf8_lossy(&out.stdout).to_string())
            } else {
                Err(SecurityError::InternalError(
                    String::from_utf8_lossy(&out.stderr).to_string(),
                ))
            }
        }
        Err(e) => Err(SecurityError::InternalError(e.to_string())),
    }
}
