use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs::File;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: u64,
    pub timestamp: std::time::SystemTime,
    pub types: HashMap<String, String>, // mime_type -> content
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

    pub fn add_entry(&mut self, types: HashMap<String, String>) {
        self.highest_id += 1;
        let entry = ClipboardEntry::new(self.highest_id, types);

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

impl ClipboardEntry {
    pub fn new(id: u64, types: HashMap<String, String>) -> Self {
        Self {
            id,
            timestamp: std::time::SystemTime::now(),
            types,
        }
    }

    pub fn get_content_by_type(&self, mime_type: &str) -> Option<&String> {
        self.types.get(mime_type)
    }

    pub fn get_text_content(&self) -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            self.get_content_by_type("text/plain").cloned()
        }

        #[cfg(target_os = "macos")]
        {
            self.get_content_by_type("public.utf8-plain-text").cloned()
        }
    }

    #[allow(dead_code)]
    pub fn get_available_types(&self) -> Vec<&String> {
        self.types.keys().collect()
    }
}
