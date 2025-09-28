use crate::r#impl::pasteboard::NSPasteboard;
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
    
    let id: usize = id_str.parse()
        .map_err(|_| format!("Invalid ID: '{}'", id_str))?;
    
    let storage = Storage::from_file(100)?;
    let entries = storage.get_entries();
    
    let entry = entries.get(id)
        .ok_or_else(|| format!("Entry with ID {} not found", id))?;
    
    let clipboard = NSPasteboard::new()?;
    clipboard.set_by_type("public.utf8-plain-text", &entry.content)?;
    
    Ok(())
}
