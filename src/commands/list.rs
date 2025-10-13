use crate::r#impl::storage::Storage;

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let storage = Storage::from_file(100)?;
    let entries = storage.get_entries();

    for entry in entries.iter() {
        let display_content = if let Some(text_content) = entry.get_text_content() {
            text_content.clone()
        } else {
            "[no text representation available]".to_string()
        };
        
        // TODO: Configure separator from command line argument
        print!("{}|{}:::", entry.id, display_content);
    }

    Ok(())
}
