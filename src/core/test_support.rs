use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Harness to isolate filesystem and environment side effects per test.
pub(crate) struct TestProject {
    root: TempDir,
    original_root: Option<OsString>,
}

impl TestProject {
    /// Create a new temporary project root and point `FUSION_PROJECT_ROOT` at it.
    pub fn new() -> Self {
        let root = TempDir::new().expect("failed to create temp project root");
        let original_root = env::var_os("FUSION_PROJECT_ROOT");
        unsafe {
            // SAFETY: tests set a process-wide isolation variable before spawning threads.
            env::set_var("FUSION_PROJECT_ROOT", root.path());
        }
        Self { root, original_root }
    }

    /// Path to the temporary project root.
    pub fn root(&self) -> &Path {
        self.root.path()
    }

    /// Path to the `.tmp` directory relative to the temporary project root.
    pub fn pid_dir(&self) -> PathBuf {
        self.root().join(".tmp")
    }

    /// Create a custom `.env` file for tests.
    pub fn write_env_file(&self, contents: &str) {
        fs::write(self.root().join(".env"), contents).expect("failed to write .env file");
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        match &self.original_root {
            Some(value) => unsafe {
                // SAFETY: restoring the original project root is serialized at drop time.
                env::set_var("FUSION_PROJECT_ROOT", value);
            },
            None => unsafe {
                // SAFETY: restoring the original project root is serialized at drop time.
                env::remove_var("FUSION_PROJECT_ROOT");
            },
        }
    }
}
