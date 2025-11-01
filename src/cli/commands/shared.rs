use crate::cli::ServiceType;
use crate::core::config::{self, Config};
use crate::core::services::{self, ManagedService};
use crate::error::AppError;

pub(super) fn load_config() -> Result<Config, AppError> {
    config::load_config()
}

pub(super) fn service_for_up(cfg: &Config, service_type: ServiceType) -> ManagedService {
    match service_type {
        ServiceType::Ollama => services::create_ollama_service(&cfg.ollama_server),
        ServiceType::Mlx => services::create_mlx_service(&cfg.mlx_server),
    }
}

pub(super) fn service_for_runtime(
    cfg: &Config,
    service_type: ServiceType,
) -> Result<ManagedService, AppError> {
    match service_type {
        ServiceType::Ollama => services::load_ollama_service(&cfg.ollama_server),
        ServiceType::Mlx => services::load_mlx_service(&cfg.mlx_server),
    }
}
