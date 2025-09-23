use crate::r#impl::pasteboard::NSPasteboard;
use crate::r#impl::storage::Storage;
use std::thread;
use std::time::Duration;

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let clipboard = NSPasteboard::new()?;

    let mut storage = Storage::from_file(100).unwrap_or_else(|_| {
        println!("No existing clipboard history found, starting fresh.");
        Storage::new(100)
    });

    println!("Starting clipboard monitor...");

    let mut last_change_count = 0;
    loop {
        match std::panic::catch_unwind(|| clipboard.get_change_count()) {
            Ok(change_count) => {
                if last_change_count != 0 && change_count != last_change_count {
                    print!("Changed: ");
                    if let Some(new_content) = clipboard.get_string() {
                        print!("{}", new_content);
                        let types = clipboard.list_types();
                        println!(" {:?}", types);
                        storage.add_entry(new_content);
                    } else {
                        println!("Failed to get clipboard content");
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
