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
}
