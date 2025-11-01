use crate::error::AppError;
use std::path::PathBuf;
use std::{env, fs};

/// Resolve the project root directory for the CLI.
pub fn project_root() -> PathBuf {
    env::var_os("FUSION_PROJECT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("failed to resolve current directory"))
}

pub fn ensure_pid_dir() -> Result<PathBuf, AppError> {
    let dir = pid_dir();
    fs::create_dir_all(&dir).map_err(AppError::from)?;
    Ok(dir)
}

/// Return the directory used for PID, log, and runtime config files.
pub fn pid_dir() -> PathBuf {
    match user_config_dir() {
        Ok(dir) => dir,
        Err(err) => panic!("Failed to resolve config directory: {err}"),
    }
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

/// Resolve the service-specific directory for logs, PID, and state files.
pub fn service_state_dir(service_name: &str) -> Result<PathBuf, AppError> {
    Ok(user_config_dir()?.join(service_name))
}

/// Resolve the service-specific configuration file.
pub fn service_config_file(service_name: &str) -> Result<PathBuf, AppError> {
    Ok(service_state_dir(service_name)?.join("config.toml"))
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
