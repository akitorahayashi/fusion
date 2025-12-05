mod commands;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceType {
    Ollama,
    Mlx,
}

pub use commands::{
    ServiceConfigCommand, handle_config, handle_down, handle_health_single, handle_logs,
    handle_logs_single, handle_ps, handle_ps_single, handle_up,
};

pub(crate) fn service_label(service_type: ServiceType) -> &'static str {
    match service_type {
        ServiceType::Ollama => "Ollama",
        ServiceType::Mlx => "MLX",
    }
}
