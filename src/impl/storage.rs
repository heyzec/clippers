use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::File;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub content: String,
    pub timestamp: std::time::SystemTime,
}

pub struct Storage {
    entries: VecDeque<ClipboardEntry>,
    max_entries: usize,
}

impl Storage {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries,
        }
    }

    pub fn from_file(max_entries: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let home_dir = std::env::var("HOME")?;
        let file_path = format!("{}/db.json", home_dir);

        let file = File::open(file_path)?;
        let entries: VecDeque<ClipboardEntry> = serde_json::from_reader(file)?;

        let mut storage = Self::new(max_entries);
        storage.entries = entries;

        // while storage.entries.len() > storage.max_entries {
        //     storage.entries.pop_back();
        // }

        Ok(storage)
    }

    pub fn add_entry(&mut self, content: String) {
        let entry = ClipboardEntry {
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

        serde_json::to_writer_pretty(file, &self.entries)?;

        Ok(())
    }
}
