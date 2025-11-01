use crate::cli::ServiceType;
use crate::core::config;
use crate::core::paths;
use crate::error::AppError;
use std::env;
use std::fs;
use std::process::Command;

/// Subcommands supported by `fusion <service> config`.
#[derive(Debug)]
pub enum ServiceConfigCommand {
    Show,
    Edit,
    Path,
    Set { key: String, value: String },
}

pub fn handle_config(
    service_type: ServiceType,
    command: ServiceConfigCommand,
) -> Result<(), AppError> {
    match command {
        ServiceConfigCommand::Show => show_config(),
        ServiceConfigCommand::Edit => edit_config(),
        ServiceConfigCommand::Path => print_config_path(),
        ServiceConfigCommand::Set { key, value } => set_config_value(service_type, key, value),
    }
}

fn show_config() -> Result<(), AppError> {
    let _ = config::load_config_document()?;
    let path = paths::user_config_file()?;
    let contents = fs::read_to_string(&path)?;
    print!("{}", contents);
    Ok(())
}

fn edit_config() -> Result<(), AppError> {
    let _ = config::load_config_document()?;
    let path = paths::user_config_file()?;
    let editor = env::var("EDITOR")
        .map_err(|_| AppError::config_error("$EDITOR is not set; cannot edit configuration"))?;

    let editor_parts: Vec<&str> = editor.split_whitespace().collect();
    if editor_parts.is_empty() {
        return Err(AppError::config_error("$EDITOR is empty"));
    }

    let mut command = Command::new(editor_parts[0]);
    for arg in &editor_parts[1..] {
        command.arg(arg);
    }
    command.arg(&path);

    let status = command
        .status()
        .map_err(|err| AppError::config_error(format!("Failed to launch editor: {err}")))?;
    if !status.success() {
        return Err(AppError::config_error("Editor exited with a non-zero status"));
    }
    Ok(())
}

fn print_config_path() -> Result<(), AppError> {
    let path = paths::user_config_file()?;
    println!("{}", path.display());
    Ok(())
}

fn set_config_value(
    _service_type: ServiceType,
    key: String,
    value: String,
) -> Result<(), AppError> {
    let mut document = config::load_config_document()?;
    let segments: Vec<String> = key
        .split('.')
        .map(|segment| segment.trim().to_string())
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.is_empty() {
        return Err(AppError::config_error("Configuration key must not be empty"));
    }
    let refs: Vec<&str> = segments.iter().map(|segment| segment.as_str()).collect();
    let inferred = config::infer_toml_edit_value(&value);
    config::set_document_value(&mut document, &refs, inferred)?;
    config::save_config_document(&document)?;

    println!("Updated configuration key '{}'", segments.join("."));
    Ok(())
}
