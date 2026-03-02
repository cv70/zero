use std::path::PathBuf;

use crate::config::{Config, ConfigError};

/// Error type for initialization failures.
#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("Home directory not found")]
    HomeDirNotFound,
    #[error("Failed to create work directory: {0}")]
    WorkDirCreateFailed(#[source] std::io::Error),
    #[error("Config load failed: {0}")]
    ConfigLoadFailed(#[from] ConfigError),
}

/// Initialized zero runtime context.
///
/// Holds the loaded `Config` and the resolved work directory (`~/.zero`).
pub struct ZeroInit {
    pub config: Config,
    pub work_dir: PathBuf,
}

impl ZeroInit {
    /// Load the zero runtime context.
    ///
    /// 1. Resolves `work_dir` to `$HOME/.zero`.
    /// 2. Creates `work_dir` if it doesn't exist.
    /// 3. Loads `work_dir/config.yaml` if present, otherwise uses `Config::default()`.
    /// 4. Uses only the file values (or defaults if the file is absent).
    pub fn load() -> Result<Self, InitError> {
        let home = std::env::var("HOME")
            .ok()
            .filter(|h| !h.is_empty())
            .map(PathBuf::from)
            .ok_or(InitError::HomeDirNotFound)?;

        let work_dir = home.join(".zero");

        if !work_dir.exists() {
            std::fs::create_dir_all(&work_dir).map_err(InitError::WorkDirCreateFailed)?;
        }

        let config_path = work_dir.join("config.yaml");
        let config = if config_path.exists() {
            Config::from_yaml_file(config_path.to_str().unwrap_or_default())?
        } else {
            Config::default()
        };

        Ok(Self { config, work_dir })
    }
}

impl Default for ZeroInit {
    fn default() -> Self {
        Self {
            config: Config::default(),
            work_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn load_creates_work_dir_and_uses_default_config() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path().to_path_buf();
        // SAFETY: test runs single-threaded (--test-threads=1)
        unsafe { std::env::set_var("HOME", &home) };

        let init = ZeroInit::load().unwrap();
        assert!(init.work_dir.exists());
        assert_eq!(init.work_dir, home.join(".zero"));
        assert_eq!(init.config.name, Some("Zero Agent".to_string()));
    }

    #[test]
    fn load_reads_config_yaml_when_present() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path().to_path_buf();
        // SAFETY: test runs single-threaded (--test-threads=1)
        unsafe { std::env::set_var("HOME", &home) };

        let zero_dir = home.join(".zero");
        fs::create_dir_all(&zero_dir).unwrap();
        fs::write(
            zero_dir.join("config.yaml"),
            r#"
name: "CustomAgent"
version: "0.2.0"
"#,
        )
        .unwrap();

        let init = ZeroInit::load().unwrap();
        assert_eq!(init.config.name, Some("CustomAgent".to_string()));
        assert_eq!(init.config.version, Some("0.2.0".to_string()));
    }

    #[test]
    fn default_fallback_uses_current_dir() {
        let init = ZeroInit::default();
        assert_eq!(init.config.name, Some("Zero Agent".to_string()));
        // work_dir is current dir, should exist
        assert!(init.work_dir.exists());
    }
}
