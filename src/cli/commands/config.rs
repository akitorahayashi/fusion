use crate::core::config;
use crate::core::paths;
use crate::error::AppError;
use std::env;
use std::fs;

/// Subcommands supported by `fusion <service> config`.
#[derive(Debug)]
pub enum ServiceConfigCommand {
    Show,
    Edit,
    Path,
    Reset,
}

pub fn handle_config(command: ServiceConfigCommand) -> Result<(), AppError> {
    match command {
        ServiceConfigCommand::Show => show_config(),
        ServiceConfigCommand::Edit => edit_config(),
        ServiceConfigCommand::Path => print_config_path(),
        ServiceConfigCommand::Reset => reset_config(),
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
    let config_path = paths::user_config_file()?;
    let current_dir = env::current_dir()
        .map_err(|err| AppError::config_error(format!("Failed to get current directory: {err}")))?;

    // Create a symlink in the current directory pointing to the config file
    let link_name = "fusion.toml";
    let link_path = current_dir.join(link_name);

    // Remove existing symlink if it exists
    if link_path.exists() {
        fs::remove_file(&link_path).map_err(|err| {
            AppError::config_error(format!("Failed to remove existing symlink: {err}"))
        })?;
    }

    // Create the symlink
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&config_path, &link_path)
            .map_err(|err| AppError::config_error(format!("Failed to create symlink: {err}")))?;
    }

    #[cfg(windows)]
    {
        // On Windows, we need to handle both file and directory symlinks
        if config_path.is_dir() {
            std::os::windows::fs::symlink_dir(&config_path, &link_path).map_err(|err| {
                AppError::config_error(format!("Failed to create directory symlink: {err}"))
            })?;
        } else {
            std::os::windows::fs::symlink_file(&config_path, &link_path).map_err(|err| {
                AppError::config_error(format!("Failed to create file symlink: {err}"))
            })?;
        }
    }

    println!("Created symlink: {} -> {}", link_path.display(), config_path.display());
    println!("You can now edit the config file using your preferred editor.");
    Ok(())
}

fn print_config_path() -> Result<(), AppError> {
    let path = paths::user_config_file()?;
    println!("{}", path.display());
    Ok(())
}

fn reset_config() -> Result<(), AppError> {
    let path = paths::user_config_file()?;

    // Remove the existing config file if it exists
    if path.exists() {
        fs::remove_file(&path).map_err(|err| {
            AppError::config_error(format!("Failed to remove existing config file: {err}"))
        })?;
        println!("Removed existing config file: {}", path.display());
    }

    // Recreate with defaults
    config::ensure_config_exists()?;
    println!("Created new config file with default values: {}", path.display());
    Ok(())
}
