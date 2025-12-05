use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use tempfile::TempDir;

/// Integration test harness configuring an isolated Fusion workspace.
pub struct CliTestContext {
    #[allow(dead_code)]
    pub root: TempDir,
    original_root: Option<OsString>,
    original_config_dir: Option<OsString>,
}

impl CliTestContext {
    pub fn new() -> Self {
        let root = TempDir::new().expect("failed to create temp root for tests");
        let original_root = env::var_os("FUSION_PROJECT_ROOT");
        let original_config_dir = env::var_os("FUSION_CONFIG_DIR");
        unsafe {
            // SAFETY: integration tests mutate process environment serially.
            env::set_var("FUSION_PROJECT_ROOT", root.path());
            env::set_var("FUSION_CONFIG_DIR", root.path().join(".config/fusion"));
        }
        Self { root, original_root, original_config_dir }
    }

    #[allow(dead_code)]
    pub fn pid_dir(&self) -> PathBuf {
        self.root.path().join(".config/fusion")
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

        match &self.original_config_dir {
            Some(value) => unsafe {
                // SAFETY: restoration happens after tests finish using the variable.
                env::set_var("FUSION_CONFIG_DIR", value);
            },
            None => unsafe {
                // SAFETY: restoration happens after tests finish using the variable.
                env::remove_var("FUSION_CONFIG_DIR");
            },
        }
    }
}
