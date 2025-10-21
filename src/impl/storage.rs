use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::File;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: u64,
    pub content: String,
    pub timestamp: std::time::SystemTime,
}

#[derive(Serialize, Deserialize)]
pub struct Storage {
    entries: VecDeque<ClipboardEntry>,
    highest_id: u64,
    #[serde(skip)]
    max_entries: usize,
}

impl Storage {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries,
            highest_id: 0,
        }
    }

    pub fn from_file(max_entries: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let home_dir = std::env::var("HOME")?;
        let file_path = format!("{}/db.json", home_dir);

        let file = File::open(file_path)?;
        let mut storage: Storage = serde_json::from_reader(file)?;

        storage.max_entries = max_entries;

        // while storage.entries.len() > storage.max_entries {
        //     storage.entries.pop_back();
        // }

        Ok(storage)
    }

    pub fn add_entry(&mut self, content: String) {
        self.highest_id += 1;
        let entry = ClipboardEntry {
            id: self.highest_id,
            content,
            timestamp: std::time::SystemTime::now(),
        };

        self.entries.push_front(entry);

        // Trim to max_entries if needed
        if self.entries.len() > self.max_entries {
            self.entries.pop_back();
        }

        let _ = self.to_file();
    }

    pub fn get_entry_by_id(&self, id: u64) -> Option<&ClipboardEntry> {
        // TODO: Consider using a hash map for faster lookup
        self.entries.iter().find(|entry| entry.id == id)
    }

    #[allow(dead_code)]
    pub fn get_entries(&self) -> &VecDeque<ClipboardEntry> {
        &self.entries
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let home_dir = std::env::var("HOME")?;
        let file_path = format!("{}/db.json", home_dir);
        let file = File::create(file_path)?;

        serde_json::to_writer_pretty(file, &self)?;

        Ok(())
    }
}
