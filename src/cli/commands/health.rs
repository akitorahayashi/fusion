use super::shared::{load_config, service_for_runtime};
use crate::cli::ServiceType;
use crate::core::health;
use crate::error::AppError;

/// Allow a slightly longer timeout for inference (considering model load time)
const HEALTH_TIMEOUT_SECS: u64 = 30;

pub fn handle_health_single(service_type: ServiceType) -> Result<(), AppError> {
    let cfg = load_config()?;

    let service = service_for_runtime(&cfg, service_type)?;
    let model_name = match service_type {
        ServiceType::Ollama => cfg.ollama_server.model.clone(),
        ServiceType::Mlx => cfg.mlx_server.model.clone(),
    };

    let prompt = "Briefly introduce yourself in one sentence.";

    println!("🩺 Checking {} health (inference test)...", service.name);
    println!("   Model: {}", model_name);
    println!("   Prompt: \"{}\"", prompt);

    let response = health::query_inference(&service, &model_name, prompt, HEALTH_TIMEOUT_SECS)?;

    println!("✅ {}: Healthy", service.name);
    println!("📝 Response: {}", response.trim());

    Ok(())
}
