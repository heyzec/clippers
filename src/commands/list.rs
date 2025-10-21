use crate::r#impl::storage::Storage;

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let storage = Storage::from_file(100)?;
    let entries = storage.get_entries();

    for (id, entry) in entries.iter().enumerate() {
        // TODO: Configure separator from command line argument
        print!("{}|{}:::", id, entry.content);
    }

    Ok(())
}
