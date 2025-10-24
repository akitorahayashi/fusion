use crate::core::{Execute, add_item::AddItem, delete_item::DeleteItem, list_items::ListItems};
use crate::error::AppError;
use crate::storage::FilesystemStorage;

/// Add a new item to storage using the default filesystem backend.
pub fn add(id: &str, content: &str) -> Result<(), AppError> {
    let storage = FilesystemStorage::new_default()?;
    let command = AddItem { id, content };

    command.execute(&storage)?;
    println!("✅ Added item '{id}'");
    Ok(())
}

/// List all stored item identifiers.
pub fn list() -> Result<Vec<String>, AppError> {
    let storage = FilesystemStorage::new_default()?;
    let command = ListItems;
    let items = command.execute(&storage)?;

    println!("📦 Stored items:");
    if items.is_empty() {
        println!("(none)");
    } else {
        for id in &items {
            println!("- {id}");
        }
    }

    Ok(items)
}

/// Delete an item from storage.
pub fn delete(id: &str) -> Result<(), AppError> {
    let storage = FilesystemStorage::new_default()?;
    let command = DeleteItem { id };

    command.execute(&storage)?;
    println!("🗑️  Deleted item '{id}'");
    Ok(())
}
