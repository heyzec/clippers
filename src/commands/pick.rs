use crate::r#impl::clipboard::create_clipboard;
use crate::r#impl::storage::Storage;
use std::io::{self, Read};

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // Extract ID, ignoring parts after '|'
    let id_str = input.trim()
        .split('|')
        .next()
        .unwrap_or("")
        .trim();

    let id: u64 = id_str.parse()
        .map_err(|_| format!("Invalid ID: '{}'", id_str))?;

    let storage = Storage::from_file(100)?;
    let entry = storage.get_entry_by_id(id)
        .ok_or_else(|| format!("Entry with ID {} not found", id))?;

    let clipboard = create_clipboard()?;
    clipboard.set_multiple_types(&entry.types)?;

    Ok(())
}
