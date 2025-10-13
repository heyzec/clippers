use crate::r#impl::storage::Storage;

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let storage = Storage::from_file(100)?;
    let entries = storage.get_entries();

    for entry in entries.iter() {
        // TODO: Configure separator from command line argument
        print!("{}|{}:::", entry.id, entry.content);
    }

    Ok(())
}
