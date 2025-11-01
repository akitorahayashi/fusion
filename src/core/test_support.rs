use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Harness to isolate filesystem and environment side effects per test.
pub(crate) struct TestProject {
    root: TempDir,
    original_root: Option<OsString>,
    original_config_dir: Option<OsString>,
}

impl TestProject {
    /// Create a new temporary project root and point `FUSION_PROJECT_ROOT` at it.
    pub fn new() -> Self {
        let root = TempDir::new().expect("failed to create temp project root");
        let original_root = env::var_os("FUSION_PROJECT_ROOT");
        let original_config_dir = env::var_os("FUSION_CONFIG_DIR");
        unsafe {
            // SAFETY: tests set a process-wide isolation variable before spawning threads.
            env::set_var("FUSION_PROJECT_ROOT", root.path());
            env::set_var("FUSION_CONFIG_DIR", root.path().join(".config/fusion"));
        }
        Self { root, original_root, original_config_dir }
    }

    /// Path to the temporary project root.
    pub fn root(&self) -> &Path {
        self.root.path()
    }

    /// Path to the config-backed runtime directory for the test project.
    pub fn pid_dir(&self) -> PathBuf {
        self.root().join(".config/fusion")
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

        match &self.original_config_dir {
            Some(value) => unsafe {
                // SAFETY: restoration of config dir happens after tests finish using it.
                env::set_var("FUSION_CONFIG_DIR", value);
            },
            None => unsafe {
                // SAFETY: restoration of config dir happens after tests finish using it.
                env::remove_var("FUSION_CONFIG_DIR");
            },
        }
    }
}
