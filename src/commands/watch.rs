use crate::r#impl::clipboard::create_clipboard;
use crate::r#impl::storage::Storage;

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = create_clipboard()?;

    let mut storage = Storage::from_file(100).unwrap_or_else(|_| {
        println!("No existing clipboard history found, starting fresh.");
        Storage::new(100)
    });

    println!("Starting clipboard monitor...");

    #[cfg(target_os = "linux")]
    {
        // Discard the first detection from inital clipboard
        let _ = clipboard.wait();
    }
    {
        loop {
            clipboard.wait().map_err(|e| {
                eprintln!("Error waiting for clipboard change: {}", e);
                e
            })?;

            let new_content = clipboard.get_string().ok_or("Failed to get clipboard content")?;
            println!();
            println!("Changed: {}", new_content);
            let types = clipboard.list_types();
            println!("Types: {:?}", types);

            for content_type in &types {
                match clipboard.get_by_type(content_type) {
                    Ok(content) => {
                        let mut hasher = DefaultHasher::new();
                        content.hash(&mut hasher);
                        let hash = hasher.finish();
                        println!("Type '{}' ({:x}): {}", content_type, hash, content);
                    }
                    Err(e) => {
                        println!("Type '{}': Failed to get content - {}", content_type, e);
                    }
                }
            }
            print!("\n");

            // Construct ClipboardEntry with all available MIME types
            let mut type_content_map = HashMap::new();

            for content_type in &types {
                if let Ok(content) = clipboard.get_by_type(content_type) {
                    type_content_map.insert(content_type.clone(), content);
                }
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
