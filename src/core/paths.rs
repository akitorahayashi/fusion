use crate::error::AppError;
use std::path::PathBuf;
use std::{env, fs};

/// Resolve the project root directory for the CLI.
pub fn project_root() -> PathBuf {
    env::var_os("FUSION_PROJECT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("failed to resolve current directory"))
}

/// Return the directory used for PID and log files.
pub fn pid_dir() -> PathBuf {
    project_root().join(".tmp")
}

/// Ensure the PID directory exists on disk.
pub fn ensure_pid_dir() -> std::io::Result<PathBuf> {
    let dir = pid_dir();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Resolve the directory containing the persistent `config.toml` file.
pub fn user_config_dir() -> Result<PathBuf, AppError> {
    if let Some(override_dir) = env::var_os("FUSION_CONFIG_DIR") {
        return Ok(PathBuf::from(override_dir));
    }

    dirs::home_dir()
        .map(|dir| dir.join(".config").join("fusion"))
        .ok_or_else(|| AppError::config_error("Could not determine home directory"))
}

/// Resolve the absolute path to the user's persistent configuration file.
pub fn user_config_file() -> Result<PathBuf, AppError> {
    Ok(user_config_dir()?.join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_support::TestProject;
    use serial_test::serial;

    #[test]
    #[serial]
    fn project_root_respects_override() {
        let project = TestProject::new();
        assert_eq!(project_root(), project.root().to_path_buf());
    }

    #[test]
    #[serial]
    fn ensure_pid_dir_creates_directory() {
        let project = TestProject::new();
        let expected = project.pid_dir();
        assert!(!expected.exists());

        let created = ensure_pid_dir().expect("pid directory should be created");
        assert_eq!(created, expected);
        assert!(expected.exists());
    }

    #[test]
    #[serial]
    fn user_config_dir_respects_override() {
        let project = TestProject::new();
        let override_path = project.root().join("config");
        unsafe {
            // SAFETY: tests run serially and restore the variable on drop.
            env::set_var("FUSION_CONFIG_DIR", &override_path);
        }

        let resolved = user_config_dir().expect("config dir should resolve");
        assert_eq!(resolved, override_path);

        unsafe {
            // SAFETY: tests run serially and can unset the variable afterwards.
            env::remove_var("FUSION_CONFIG_DIR");
        }
    }
}
