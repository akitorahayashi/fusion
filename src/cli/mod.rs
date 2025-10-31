pub mod llm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceType {
    Ollama,
    Mlx,
}

pub use llm::{
    StartOptions, handle_down, handle_logs, handle_logs_single, handle_ps, handle_ps_single,
    handle_up,
};
