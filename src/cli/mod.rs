pub mod llm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceType {
    Ollama,
    Mlx,
}

pub use llm::{
    RunOverrides, ServiceConfigCommand, handle_config, handle_down, handle_logs,
    handle_logs_single, handle_ps, handle_ps_single, handle_run, handle_up,
};
