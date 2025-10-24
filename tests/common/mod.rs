use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Integration test harness configuring an isolated Fusion workspace.
pub struct CliTestContext {
    root: TempDir,
    original_root: Option<OsString>,
}

impl CliTestContext {
    pub fn new() -> Self {
        let root = TempDir::new().expect("failed to create temp root for tests");
        let original_root = env::var_os("FUSION_PROJECT_ROOT");
        unsafe {
            // SAFETY: integration tests mutate process environment serially.
            env::set_var("FUSION_PROJECT_ROOT", root.path());
        }
        Self { root, original_root }
    }

    pub fn root(&self) -> &Path {
        self.root.path()
    }

    pub fn pid_dir(&self) -> PathBuf {
        self.root().join(".tmp")
    }
}

impl Drop for CliTestContext {
    fn drop(&mut self) {
        match &self.original_root {
            Some(value) => unsafe {
                // SAFETY: restoration happens after tests finish using the variable.
                env::set_var("FUSION_PROJECT_ROOT", value);
            },
            None => unsafe {
                // SAFETY: restoration happens after tests finish using the variable.
                env::remove_var("FUSION_PROJECT_ROOT");
            },
        }
    }
}
