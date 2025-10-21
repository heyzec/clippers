use crate::r#impl::clipboard::create_clipboard;
use crate::r#impl::storage::Storage;
use std::collections::hash_map::HashMap;

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = create_clipboard()?;

    let mut storage = Storage::from_file(100).unwrap_or_else(|_| {
        println!("No existing clipboard history found, starting fresh.");
        Storage::new(100)
    });

    println!("Starting clipboard monitor...");

    #[cfg(target_os = "linux")]
    {
        // Discard the first detection from initial clipboard
        let _ = clipboard.wait();
    }
    {
        loop {
            clipboard.wait().map_err(|e| {
                eprintln!("Error waiting for clipboard change: {}", e);
                e
            })?;

            let types = clipboard.list_types();

            let mut type_content_map = HashMap::new();
            for content_type in &types {
                let content = clipboard
                    .get_by_type(content_type)
                    .expect("Failed to get content");
                type_content_map.insert(content_type.clone(), content);
            }

            if !type_content_map.is_empty() {
                storage.add_entry(type_content_map);
                println!("Stored clipboard entry with {} types", types.len());
            } else {
                println!("No valid content to store.");
            }
        }
    }
}
