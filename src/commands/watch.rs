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
        // Discard the first detection from initial clipboard
        let _ = clipboard.wait();
    }
    {
        loop {
            clipboard.wait().map_err(|e| {
                eprintln!("Error waiting for clipboard change: {}", e);
                e
            })?;

            let new_content = clipboard
                .get_string()
                .ok_or("Failed to get clipboard content")?;
            println!();
            println!("Changed: {}", new_content);
            let types = clipboard.list_types();
            println!("Types: {:?}", types);
            storage.add_entry(new_content);
        }
    }
}
