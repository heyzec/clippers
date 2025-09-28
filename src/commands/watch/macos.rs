use crate::r#impl::pasteboard::NSPasteboard;
use crate::r#impl::storage::Storage;
use std::collections::{HashMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::thread;
use std::time::Duration;

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let clipboard = NSPasteboard::new()?;

    let mut storage = Storage::from_file(100).unwrap_or_else(|e| {
        eprintln!("Error loading clipboard history: {}", e);
        Storage::new(100)
    });

    println!("Starting clipboard monitor...");

    let mut last_change_count = 0;
    loop {
        match std::panic::catch_unwind(|| clipboard.get_change_count()) {
            Ok(change_count) => {
                if last_change_count != 0 && change_count != last_change_count {
                    print!("Changed: ");
                    let types = clipboard.list_types();                    
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
                        println!("Failed to get any clipboard content");
                    }
                }
                last_change_count = change_count;
            }
            Err(_) => {
                eprintln!("Error checking clipboard change count");
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
}
